use heck::ToPascalCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{parse_macro_input, token, DeriveInput, Ident, LitStr};

#[proc_macro_derive(World, attributes(world))]
pub fn world_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    //panic!("{:#?}", input);

    let name = &input.ident;
    let st = match input.data {
        syn::Data::Struct(st) => st,
        syn::Data::Enum(_) | syn::Data::Union(_) => todo!(),
    };
    let mut fields_borrow_without = Vec::new();
    let mut handles_field = None;

    for field in st.fields.iter() {
        for attr in field.attrs.iter() {
            if attr.path().is_ident("world") {
                attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("handles") {
                        handles_field = Some(field);
                        Ok(())
                    } else if meta.path.is_ident("split_off") {
                        // this parses the `split_off`
                        let value = meta.value()?; // this parses the `=`
                        let s: Ident = value.parse()?; // this parses `"World"`
                        fields_borrow_without.push((field, s));
                        Ok(())
                    } else {
                        Err(meta.error("unsupported attribute"))
                    }
                })
                .unwrap();
            }
        }
    }
    let handles_field = handles_field.unwrap();

    let struct_defs = fields_borrow_without.iter().map(|(field, struct_name)| {
        //let field_name_caps = field.ident.as_ref().unwrap().to_string().to_pascal_case();
        //let struct_name = format_ident!("{name}No{field_name_caps}");

        let field_types = st
            .fields
            .iter()
            .filter(|field2| *field2 != *field)
            .filter(|field2| *field2 != handles_field)
            .map(|field| &field.ty);

        let field_names: Vec<_> = st
            .fields
            .iter()
            .filter(|field2| *field2 != *field)
            .filter(|field2| *field2 != handles_field)
            .map(|field| field.ident.as_ref().unwrap())
            .collect();

        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        let func_name = format_ident!("split_{field_name}");

        quote! {
            #[derive(Debug)]
            pub struct #struct_name <'a> {
                #(
                    pub #field_names: &'a mut #field_types,
                )*
            }

            impl #name {
                pub fn #func_name <'a>(&'a mut self) -> (&'a mut #field_type, #struct_name <'a>) {
                    (
                        &mut self. #field_name,
                        #struct_name {
                            #(
                                #field_names: &mut self. #field_names,
                            )*
                        }
                    )
                }
            }

            impl<'a, 'b: 'a> vec_ecs::WorldBorrow<'a> for #struct_name <'b> {}
        }
    });

    let handles_name = handles_field.ident.as_ref().unwrap();

    let field_names_other_than_handles = st
        .fields
        .iter()
        .filter(|field| *field != handles_field)
        .map(|field| field.ident.as_ref().unwrap());

    let expanded = quote! {
        #(
            #struct_defs
        )*

        impl vec_ecs::World for #name {
            fn new_entity(&mut self) -> vec_ecs::EntityHandle {
                self. #handles_name .next_handle()
            }
            fn delete_entity(&mut self, entity: vec_ecs::EntityHandle) {
                self. #handles_name .entity_deleted();
                #(
                    self. #field_names_other_than_handles . remove(entity);
                )*
            }
        }

        impl<'a> vec_ecs::WorldBorrow<'a> for #name {}
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
            impl<'a, 'b: 'a> vec_ecs::EntityBorrow<'a, #world_borrow_name <'b>> for #name_borrow <'a> {
                fn borrow(handle: vec_ecs::EntityHandle, world: &'a mut #world_borrow_name <'b>) -> Self {
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
                impl<'a> vec_ecs::EntityBorrow<'a, #world_insert_name> for #name_borrow <'a> {
                    fn borrow(handle: vec_ecs::EntityHandle, world: &'a mut #world_insert_name) -> Self {
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
        impl vec_ecs::Entity for #name {
            type WorldInsert = #world_insert_name;
            fn insert_into_world(self, id: vec_ecs::EntityHandle, world: &mut Self::WorldInsert) {
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
