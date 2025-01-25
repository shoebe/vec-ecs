use heck::ToSnekCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, DeriveInput, Ident};

#[proc_macro_derive(World, attributes(world))]
pub fn world_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    //panic!("{:#?}", input);

    let name = &input.ident;
    let st = match input.data {
        syn::Data::Struct(st) => st,
        syn::Data::Enum(_) | syn::Data::Union(_) => todo!(),
    };

    let mut borrow_names = Vec::new();

    for attr in input.attrs.iter() {
        if attr.path().is_ident("world") {
            let e = attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("borrow") {
                    // this parses the `borrow`
                    let value = meta.value()?; // this parses the `=`
                    let s: Ident = value.parse()?; // this parses borrow_name
                    borrow_names.push(s);
                    Ok(())
                } else {
                    Err(meta.error("unsupported attribute"))
                }
            });
            if let Err(e) = e {
                return e.to_compile_error().into();
            }
        }
    }

    let mut fields_borrow_without = Vec::new();
    let mut handles_field = None;

    for field in st.fields.iter() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident("world") {
                let e = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("handles") {
                        handles_field = Some(field);
                        Ok(())
                    } else if meta.path.is_ident("not_in") {
                        // this parses the `split_off`
                        let value = meta.value()?; // this parses the `=`
                        let s: Ident = value.parse()?; // this parses `borrow_name`
                        fields_borrow_without.push((s, field));
                        Ok(())
                    } else {
                        Err(meta.error("unsupported attribute"))
                    }
                });
                if let Err(e) = e {
                    return e.to_compile_error().into();
                }
            }
        }
    }
    let handles_field = handles_field.expect(
        "need a #[world(handles)] attribute label on a struct field of type EntityHandleCounter",
    );

    let mut struct_defs = Vec::new();

    let handles_name = handles_field.ident.as_ref().unwrap();
    let handle_ty = &handles_field.ty;

    for borrow_name in borrow_names.iter() {
        //let field_name_caps = field.ident.as_ref().unwrap().to_string().to_pascal_case();
        //let struct_name = format_ident!("{name}No{field_name_caps}");

        let fields_to_ignore: Vec<_> = fields_borrow_without
            .iter()
            .filter_map(|(b_name, field)| {
                if b_name == borrow_name {
                    Some(field)
                } else {
                    None
                }
            })
            .collect();

        if fields_to_ignore.is_empty() {
            return quote_spanned! {
                borrow_name.span() => compile_error!("borrow has no fields that are ignored");
            }
            .into();
        }

        let fields: Vec<_> = st
            .fields
            .iter()
            .filter(|field2| field2.ident != handles_field.ident)
            .filter(|field2| !fields_to_ignore.iter().any(|f| f.ident == field2.ident))
            .collect();

        let field_types = fields.iter().map(|field| &field.ty);

        let field_names: Vec<_> = fields
            .iter()
            .map(|field| field.ident.as_ref().unwrap())
            .collect();

        let ignored_field_names = fields_to_ignore.iter().map(|f| f.ident.as_ref().unwrap());
        let ignored_field_types = fields_to_ignore.iter().map(|f| &f.ty);

        let borrow_name_snake = borrow_name.to_string().to_snek_case();
        let func_name = format_ident!("split_{borrow_name_snake}");

        let q = quote! {
            #[derive(Debug)]
            pub struct #borrow_name <'a> {
                #handles_name: &'a mut #handle_ty,
                #(
                    pub #field_names: &'a mut #field_types,
                )*
            }

            impl #name {
                pub fn #func_name <'a>(&'a mut self) -> (( #( &'a mut #ignored_field_types),* ), #borrow_name <'a>) {
                    (
                        ( #(&mut self. #ignored_field_names),* ),
                        #borrow_name {
                            #handles_name: &mut self. #handles_name,
                            #(
                                #field_names: &mut self. #field_names,
                            )*
                        }
                    )
                }
            }

            impl<'a, 'b: 'a> vec_ecs::WorldBorrowTrait<'a> for #borrow_name <'b> {
                fn new_entity(&mut self) -> vec_ecs::EntityHandle {
                    self. #handles_name .next_handle()
                }
            }
        };
        struct_defs.push(q);
    }

    let field_names_other_than_handles: Vec<_> = st
        .fields
        .iter()
        .filter(|field| field.ident != handles_field.ident)
        .map(|field| field.ident.as_ref().unwrap())
        .collect();

    let expanded = quote! {
        #(
            #struct_defs
        )*

        impl vec_ecs::WorldTrait for #name {
            fn new_entity(&mut self) -> vec_ecs::EntityHandle {
                self. #handles_name .next_handle()
            }
            fn delete_entity(&mut self, handle: vec_ecs::EntityHandle) {
                self. #handles_name .entity_deleted(handle);
                #(
                    self. #field_names_other_than_handles . remove(handle);
                )*
            }
            fn is_empty(&self) -> bool {
                #(self. #field_names_other_than_handles . is_empty())&&*
            }
        }

        impl<'a> vec_ecs::WorldBorrowTrait<'a> for #name {
            fn new_entity(&mut self) -> vec_ecs::EntityHandle {
                self. #handles_name .next_handle()
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}

#[proc_macro_derive(Entity, attributes(entity))]
pub fn entity_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;
    let name_borrow = format_ident!("{name}Borrow");

    let mut world_insert_name = None;
    let mut world_borrow_names = Vec::new();
    for attr in input.attrs.iter() {
        if attr.path().is_ident("entity") {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("insert") {
                    // this parses the `insert`
                    let value = meta.value()?; // this parses the `=`
                    let s: Ident = value.parse()?; // this parses `"World"`
                    world_insert_name = Some(s);
                    Ok(())
                } else if meta.path.is_ident("borrow") {
                    // this parses the `borrow`
                    let value = meta.value()?; // this parses the `=`
                    let s: Ident = value.parse()?; // this parses `"World"`
                    world_borrow_names.push(s);
                    Ok(())
                } else {
                    Err(meta.error("unsupported attribute"))
                }
            })
            .unwrap();
        }
    }
    let world_insert_name = world_insert_name.unwrap();

    let st = match input.data {
        syn::Data::Struct(st) => st,
        syn::Data::Enum(_) | syn::Data::Union(_) => todo!(),
    };
    let field_names: Vec<_> = st
        .fields
        .iter()
        .map(|f| f.ident.as_ref().unwrap())
        .collect();
    let field_types: Vec<_> = st.fields.iter().map(|f| &f.ty).collect();

    let world_borrow_impls = world_borrow_names.iter().map(|world_borrow_name| {
        quote! {
            impl<'a, 'b: 'a> vec_ecs::EntityBorrowFromWorldTrait<'a, #world_borrow_name <'b>> for #name_borrow <'a> {
                fn borrow_from_world(handle: vec_ecs::EntityHandle, world: &'a mut #world_borrow_name <'b>) -> Self {
                    Self {
                        #(
                            #field_names: world. #field_names .get_mut(handle).unwrap(),
                        )*
                    }
                }
            }
        }
    }).chain(
        std::iter::once(
            quote! {
                impl<'a> vec_ecs::EntityBorrowFromWorldTrait<'a, #world_insert_name> for #name_borrow <'a> {
                    fn borrow_from_world(handle: vec_ecs::EntityHandle, world: &'a mut #world_insert_name) -> Self {
                        Self {
                            #(
                                #field_names: world. #field_names .get_mut(handle).unwrap(),
                            )*
                        }
                    }
                }
            }
        )
    );

    let expanded = quote! {
        impl vec_ecs::EntityInsertIntoWorldTrait<#world_insert_name> for #name {
            fn insert_into_world(self, id: vec_ecs::EntityHandle, world: &mut #world_insert_name) {
                #(
                    world. #field_names .insert(id, self. #field_names);
                )*
            }
        }

        #[derive(Debug)]
        struct #name_borrow <'a> {
            #(
                #field_names: &'a mut #field_types,
            )*
        }

        #(
            #world_borrow_impls
        )*

    };
    proc_macro::TokenStream::from(expanded)
}
