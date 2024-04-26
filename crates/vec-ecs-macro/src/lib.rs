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
    func: syn::Expr,
}

impl Parse for CompIter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fields = Punctuated::parse_separated_nonempty(input)?;
        input.parse::<Token![;]>()?;
        let func = input.parse::<Expr>()?;

        Ok(Self { fields, func })
    }
}

#[proc_macro]
pub fn comp_iter(input: TokenStream) -> TokenStream {
    let CompIter { fields, func } = parse_macro_input!(input as CompIter);

    let mut owners_field_iter = fields.iter();
    let owners_first_field = owners_field_iter.next().unwrap();

    let mut comps_field_iter = fields.iter();
    let comps_first_field = comps_field_iter.next().unwrap();

    let mut comps_func_iter = fields.iter().map(|field| {
        let is_mut = if let Expr::Reference(r) = &field {
            r.mutability.is_some()
        } else {
            false
        };
        if is_mut {
            format_ident!("{}", "get_mut_comp_ind")
        } else {
            format_ident!("{}", "get_comp_ind")
        }
    });
    let comps_first_func = comps_func_iter.next().unwrap();

    let expanded = quote! {
        let mut inter = (#owners_first_field).owners().clone();
        #(
            inter.intersect_with((#owners_field_iter).owners());
        )*

        inter.into_ones().for_each(|comp_ind| {
            let (id1, comp1) = (#comps_first_field). #comps_first_func (comp_ind);

            (#func)(
                id1,
                comp1,
                #(
                    {
                        let (id, comp) = (#comps_field_iter). #comps_func_iter (comp_ind);
                        assert_eq!(id1, id);
                        comp
                    },
                )*
            )
        });

    };

    TokenStream::from(expanded)
}
