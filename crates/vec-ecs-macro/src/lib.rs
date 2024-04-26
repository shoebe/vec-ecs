use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse_macro_input, punctuated::Punctuated, spanned::Spanned, Expr, ExprReference,
    Ident, Token,
};

/*
iter_comps!(&mut world.pos, &mut world.vel, &mut world.yomama; |id, (pos, (vel, yomama))| {
    dbg!(id);
});

*/

struct CompIter {
    borrows: Punctuated<Expr, Token![,]>,
    optional_borrows: Option<Punctuated<Expr, Token![,]>>,
}

impl Parse for CompIter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let borrows = Punctuated::parse_separated_nonempty(input)?;

        let optional_borrows = if input.parse::<Token![;]>().is_ok() {
            let is_opt = if let Ok(ident) = input.parse::<Ident>() {
                ident == "optional"
            } else {
                false
            };
            if is_opt {
                input.parse::<Token![:]>()?;
                Some(Punctuated::parse_separated_nonempty(input)?)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            borrows,
            optional_borrows,
        })
    }
}

#[proc_macro]
pub fn comp_iter(input: TokenStream) -> TokenStream {
    let CompIter {
        borrows,
        optional_borrows,
    } = parse_macro_input!(input as CompIter);

    let intersections = {
        let mut others = borrows.iter();
        let first = others.next().unwrap();

        quote! {
            let mut inter = (#first).owners().clone();
            #(
                inter.intersect_with((#others).owners());
            )*
        }
    };

    let helper_names: Vec<_> = borrows
        .iter()
        .enumerate()
        .map(|(num, _)| format_ident!("helper_{num}"))
        .collect();

    let opt_helper_names: Vec<_> = optional_borrows
        .iter()
        .flatten()
        .enumerate()
        .map(|(num, _)| format_ident!("opt_helper_{num}"))
        .collect();

    let comps_helper_iter_init = {
        let creation_funcs = borrows.iter().map(|field| {
            let is_mut = if let Expr::Reference(r) = &field {
                r.mutability.is_some()
            } else {
                false
            };
            if is_mut {
                quote!((#field).iter_helper_mut())
            } else {
                quote!((#field).iter_helper())
            }
        });
        let opt_creation_funcs = optional_borrows.iter().flatten().map(|field| {
            let is_mut = if let Expr::Reference(r) = &field {
                r.mutability.is_some()
            } else {
                false
            };
            if is_mut {
                quote!((#field).optional_iter_helper_mut())
            } else {
                quote!((#field).optional_iter_helper())
            }
        });

        quote! {
            #(
                let mut #helper_names = #creation_funcs;
            )*
            #(
                let mut #opt_helper_names = #opt_creation_funcs;
            )*
        }
    };

    let helper_generics: Vec<_> = (0..helper_names.len())
        .map(|num| format_ident!("T{num}"))
        .collect();

    let opt_helper_generics: Vec<_> = (0..opt_helper_names.len())
        .map(|num| format_ident!("OptT{num}"))
        .collect();

    let helper_types = borrows.iter().enumerate().map(|(count, field)| {
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

    let opt_helper_types = optional_borrows
        .iter()
        .flatten()
        .enumerate()
        .map(|(count, field)| {
            let is_mut = if let Expr::Reference(r) = &field {
                r.mutability.is_some()
            } else {
                false
            };
            let ident = format_ident!("OptT{count}");
            if is_mut {
                quote!(vec_ecs::OptionalCombIterHelperMut<'a, #ident>)
            } else {
                quote!(vec_ecs::OptionalCombIterHelper<'a, #ident>)
            }
        });

    let helper_generic_borrows = borrows.iter().enumerate().map(|(count, field)| {
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

    let opt_helper_generic_borrows =
        optional_borrows
            .iter()
            .flatten()
            .enumerate()
            .map(|(count, field)| {
                let is_mut = if let Expr::Reference(r) = &field {
                    r.mutability.is_some()
                } else {
                    false
                };
                let ident = format_ident!("OptT{count}");
                if is_mut {
                    quote!(Option<&'a mut #ident>)
                } else {
                    quote!(Option<&'a #ident>)
                }
            });

    let mut helper_names_no_first = helper_names.iter();
    helper_names_no_first.next().unwrap(); // this is helper_0

    let expanded = quote! {
        {
            #intersections

            #comps_helper_iter_init

            pub struct MultipleComponentsIter<'a, #(#helper_generics, )* #(#opt_helper_generics, )*> {
                ones: fixedbitset::IntoOnes,
                #(
                    #helper_names: #helper_types,
                )*
                #(
                    #opt_helper_names: #opt_helper_types,
                )*
            }
            impl<'a, #(#helper_generics, )* #(#opt_helper_generics, )*> Iterator for MultipleComponentsIter<'a, #(#helper_generics, )* #(#opt_helper_generics, )*> {
                type Item = (EntityHandle, #(#helper_generic_borrows, )* #(#opt_helper_generic_borrows, )*);

                fn next(&mut self) -> Option<Self::Item> {
                    self.ones.next().map(|entity_ind| {
                        let (id1, comp1) = self.helper_0.comp_at(entity_ind);
                        (
                            id1,
                            comp1,
                            #(
                                {
                                    let (id, comp) = self. #helper_names_no_first .comp_at(entity_ind);
                                    assert_eq!(id1, id);
                                    comp
                                },
                            )*
                            #(
                                self. #opt_helper_names .comp_at(entity_ind).map(|(id, comp)| {
                                    assert_eq!(id1, id);
                                    comp
                                }),
                            )*
                        )
                    })
                }
            }
            MultipleComponentsIter {
                ones: inter.into_ones(),
                #(
                    #helper_names,
                )*
                #(
                    #opt_helper_names,
                )*
            }
        }
    };

    TokenStream::from(expanded)
}
