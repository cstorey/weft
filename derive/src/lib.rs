extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate syn;
extern crate failure;
#[macro_use]
extern crate html5ever;

use std::path::PathBuf;

use failure::{Error, ResultExt};
use html5ever::parse_fragment;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::TendrilSink;
use html5ever::QualName;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::ToTokens;

#[proc_macro_derive(WeftTemplate, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    match make_template(&ast) {
        Ok(toks) => toks.into(),
        Err(err) => panic!("Error: {:?}", err),
    }
}

fn make_template(item: &syn::DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    let template = find_template(item).expect("find template");
    let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".into());
    let path = PathBuf::from(root_dir).join(template);
    let dom = parse_fragment(
        RcDom::default(),
        Default::default(),
        QualName::new(None, ns!(html), local_name!("body")),
        Vec::new(),
    ).from_utf8()
    .from_file(&path)
    .with_context(|_| format!("Parsing template from path {:?}", &path))?;

    let impl_body = template_fn_body(dom)?;

    println!("Fn body: {}", impl_body);

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let x = quote! {
        impl #impl_generics ::weft::Renderable for #ident #ty_generics #where_clause {
            fn render_to<T: RenderTarget>(&self, mut target: T) -> Result<(), io::Error> {
                #impl_body;
                /*
                */
                Ok(())
            }
        }
    };
    Ok(x.into_token_stream())
}

fn find_template(item: &syn::DeriveInput) -> Result<String, Error> {
    let attr = item
        .attrs
        .iter()
        .filter_map(|a| a.interpret_meta())
        .find(|a| a.name() == "template")
        .ok_or_else(|| failure::err_msg("Could not find template attribute"))?;

    let meta_list = match attr {
        syn::Meta::List(inner) => inner,
        _ => return Err(failure::err_msg("template attribute incorrectly formatted")),
    };

    let path = {
        let mut path = None;
        for meta in meta_list.nested {
            if let syn::NestedMeta::Meta(ref item) = meta {
                if let syn::Meta::NameValue(ref pair) = item {
                    match pair.ident.to_string().as_ref() {
                        "path" => if let syn::Lit::Str(ref s) = pair.lit {
                            path = Some(s.value());
                        } else {
                            return Err(failure::err_msg(
                                "template path attribute should be a string",
                            ));
                        },
                        _ => (),
                    }
                }
            }
        }
        path.ok_or_else(|| failure::err_msg("Missing path attribute"))?
    };

    Ok(path)
}

fn template_fn_body(dom: RcDom) -> Result<TokenStream2, Error> {
    let _ = quote!(target.start_element("p".into())?);
    let _ = quote!(target.text("Hello".into())?);
    let _ = quote!(target.end_element("p".into())?);

    let mut statements = Vec::<TokenStream2>::new();

    walk_dom(&mut statements, &dom.document)?;

    let mut body = TokenStream2::new();
    body.extend(statements);

    Ok(body)
}

fn walk_dom(statements: &mut Vec<TokenStream2>, node: &Handle) -> Result<(), Error> {
    match node.data {
        NodeData::Document => {
            walk_children(statements, node)?;
        }
        NodeData::Doctype { .. } => {
            println!(
                "Ignoring doctype: children: {:?}",
                node.children.borrow().len()
            );
        }
        NodeData::Element { ref name, .. } => {
            let localname = name.local.to_string();
            statements.push(quote!(
                target.start_element(#localname.into())?;
            ));

            walk_children(statements, &node)?;

            statements.push(quote!(
                target.end_element(#localname.into())?;
            ));
        }
        NodeData::Text { ref contents } => {
            let cdata = contents.borrow().to_string();
            statements.push(quote!(
                target.text(#cdata)?;
            ))
        }
        NodeData::Comment { .. } => {
            println!(
                "Ignoring comment: children: {:?}",
                node.children.borrow().len()
            );
        }
        NodeData::ProcessingInstruction { .. } => {
            println!(
                "Ignoring processing instruction: children: {:?}",
                node.children.borrow().len()
            );
        }
    }
    Ok(())
}

fn walk_children(statements: &mut Vec<TokenStream2>, node: &Handle) -> Result<(), Error> {
    for child in node.children.borrow().iter() {
        walk_dom(statements, &child)?;
    }

    Ok(())
}
