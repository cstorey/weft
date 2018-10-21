extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;

use proc_macro::TokenStream;
use quote::ToTokens;

#[proc_macro_derive(WeftTemplate, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    make_template(&ast).into()
}

fn make_template(item: &syn::DeriveInput) -> proc_macro2::TokenStream {
    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let x = quote! {
        impl #impl_generics ::weft::Renderable for #ident #ty_generics #where_clause {
            fn render_to<T: RenderTarget>(&self, mut target: T) -> Result<(), io::Error> {
                target.start_element("p".into())?;
                target.text("Hello".into())?;
                target.end_element("p".into())?;
                Ok(())
            }
        }
    };
    x.into_token_stream()
}
