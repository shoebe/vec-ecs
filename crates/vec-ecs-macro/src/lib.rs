use heck::ToPascalCase;
use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Attribute,
    DeriveInput, Expr, ExprReference, Ident, Token,
};

#[proc_macro_derive(World, attributes(world))]
pub fn comp_iter(input: TokenStream) -> TokenStream {
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
                    if meta.path.is_ident("struct_borrow_without") {
                        fields_borrow_without.push(field);
                    }
                    if meta.path.is_ident("handles") {
                        handles_field = Some(field);
                    }
                    Ok(())
                });
            }
        }
    }
    let handles_field = handles_field.unwrap();

    let struct_defs = fields_borrow_without.iter().map(|field| {
        let field_name_caps = field.ident.as_ref().unwrap().to_string().to_pascal_case();
        let struct_name = format_ident!("{name}No{field_name_caps}");
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

        quote! {
            pub struct #struct_name <'a> {
                #(
                    #field_names: &'a mut #field_types,
                )*
            }

            impl<'a> #struct_name <'a>{
                pub fn split_world(world: &'a mut #name) -> (&'a mut #field_type, Self) {
                    (
                        &mut world. #field_name,
                        Self {
                            #(
                                #field_names: &mut world. #field_names,
                            )*
                        }
                    )
                }
            }
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

        impl #name {
            pub fn new_entity(&mut self) -> EntityHandle {
                self. #handles_name .next_handle()
            }
            pub fn delete_entity(&mut self, entity: EntityHandle) {
                self. #handles_name .entity_deleted();
                #(
                    self. #field_names_other_than_handles . remove(entity);
                )*
            }
        }
    };
    proc_macro::TokenStream::from(expanded)
}
