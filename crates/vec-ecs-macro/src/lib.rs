use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Expr, ExprReference,
    Token,
};

/*
iter_comps!(&mut world.pos, &mut world.vel, &mut world.yomama; |id, (pos, (vel, yomama))| {
    dbg!(id);
});

*/

struct CompIter {
    fields: Punctuated<Expr, Token![,]>,
}

impl Parse for CompIter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fields = Punctuated::parse_separated_nonempty(input)?;

        Ok(Self { fields })
    }
}

#[proc_macro]
pub fn comp_iter(input: TokenStream) -> TokenStream {
    let CompIter { fields } = parse_macro_input!(input as CompIter);

    let mut owners_field_iter = fields.iter();
    let owners_first_field = owners_field_iter.next().unwrap();

    let comps_helper_iter = fields.iter();
    let comps_helper_names: Vec<_> = (0..fields.len())
        .map(|num| format_ident!("helper_{num}"))
        .collect();

    let comps_helper_creation_funcs = fields.iter().map(|field| {
        let is_mut = if let Expr::Reference(r) = &field {
            r.mutability.is_some()
        } else {
            false
        };
        if is_mut {
            format_ident!("{}", "iter_helper_mut")
        } else {
            format_ident!("{}", "iter_helper")
        }
    });

    let comps_helper_iter_generics: Vec<_> = (0..fields.len())
        .map(|num| format_ident!("T{num}"))
        .collect();

    let comps_helper_iter_types = fields.iter().enumerate().map(|(count, field)| {
        let is_mut = if let Expr::Reference(r) = &field {
            r.mutability.is_some()
        } else {
            false
        };
        let ident = format_ident!("T{count}");
        if is_mut {
            quote!(vec_ecs::CompIterHelperMut<'a, #ident>)
        } else {
            quote!(vec_ecs::CompIterHelper<'a, #ident>)
        }
    });

    let comps_helper_iter_generic_borrows = fields.iter().enumerate().map(|(count, field)| {
        let is_mut = if let Expr::Reference(r) = &field {
            r.mutability.is_some()
        } else {
            false
        };
        let ident = format_ident!("T{count}");
        if is_mut {
            quote!(&'a mut  #ident)
        } else {
            quote!(&'a #ident)
        }
    });

    let mut comps_helper_no_first = comps_helper_names.iter();
    comps_helper_no_first.next().unwrap(); // this is helper_0

    let expanded = quote! {
        {
            let mut inter = (#owners_first_field).owners().clone();
            #(
                inter.intersect_with((#owners_field_iter).owners());
            )*

            #(
                let mut #comps_helper_names = (#comps_helper_iter) . #comps_helper_creation_funcs ();
            )*

            pub struct MultipleComponentsIter<'a, #(#comps_helper_iter_generics, )*> {
                ones: fixedbitset::IntoOnes,
                #(
                    #comps_helper_names: #comps_helper_iter_types,
                )*
            }
            impl<'a, #(#comps_helper_iter_generics, )*> Iterator for MultipleComponentsIter<'a, #(#comps_helper_iter_generics, )*> {
                type Item = (EntityHandle, #(#comps_helper_iter_generic_borrows, )*);

                fn next(&mut self) -> Option<Self::Item> {
                    self.ones.next().map(|entity_ind| {
                        let (id1, comp1) = self.helper_0.comp_at(entity_ind);
                        (
                            id1,
                            comp1,
                            #(
                                {
                                    let (id, comp) = self. #comps_helper_no_first .comp_at(entity_ind);
                                    assert_eq!(id1, id);
                                    comp
                                },
                            )*
                        )
                    })
                }
            }
            MultipleComponentsIter {
                ones: inter.into_ones(),
                #(
                    #comps_helper_names,
                )*
            }
        }
    };

    TokenStream::from(expanded)
}
