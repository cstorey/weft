extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
extern crate failure;
extern crate syn;
#[macro_use]
extern crate html5ever;

use std::path::{Path, PathBuf};

use failure::{Error, ResultExt};
use html5ever::parse_fragment;
use html5ever::rcdom::{Handle, NodeData, RcDom};
use html5ever::tendril::{StrTendril, TendrilSink};
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
    let template = find_template(item).context("find template")?;
    let root_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or(".".into());
    let path = PathBuf::from(root_dir).join(template);

    let dom = parse(&path)?;

    let impl_body = template_fn_body(&dom)?;

    // eprintln!("Fn body: {}", impl_body);

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let x = quote! {
        impl #impl_generics ::weft::Renderable for #ident #ty_generics #where_clause {
            fn render_to<T: RenderTarget>(&self, mut target: T) -> Result<(), io::Error> {
                #impl_body;
                Ok(())
            }
        }
    };
    Ok(x.into_token_stream())
}

fn parse(path: &Path) -> Result<Vec<Handle>, Error> {
    let root_name = QualName::new(None, ns!(html), local_name!("html"));
    let dom = parse_fragment(RcDom::default(), Default::default(), root_name, Vec::new())
        .from_utf8()
        .from_file(&path)
        .with_context(|_| format!("Parsing template from path {:?}", &path))?;

    let content = find_root_from(dom.document)
        .ok_or_else(|| failure::err_msg("Could not locate root of parsed document?"))?;

    Ok(content)
}

fn find_root_from(node: Handle) -> Option<Vec<Handle>> {
    let root_name = QualName::new(None, ns!(html), local_name!("html"));
    match node.data {
        NodeData::Element { ref name, .. } => {
            if name == &root_name {
                return Some(node.children.borrow().clone());
            }
        }
        _ => {}
    }
    let children = node.children.borrow();
    for child in children.iter() {
        if let Some(it) = find_root_from(child.clone()) {
            return Some(it);
        }
    }

    None
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

fn template_fn_body(nodes: &[Handle]) -> Result<TokenStream2, Error> {
    let mut statements = Vec::<TokenStream2>::new();

    walk_children(&mut statements, nodes)?;

    let mut body = TokenStream2::new();
    body.extend(statements);

    Ok(body)
}

fn walk_dom(statements: &mut Vec<TokenStream2>, node: &Handle) -> Result<(), Error> {
    match node.data {
        NodeData::Document => {
            walk_children(statements, &node.children.borrow())?;
        }
        NodeData::Doctype { .. } => {
            eprintln!(
                "Ignoring doctype: children: {:?}",
                node.children.borrow().len()
            );
        }
        NodeData::Element { ref name, .. } => {
            walk_element(name, &node.children.borrow(), statements)?;
        }
        NodeData::Text { ref contents } => {
            walk_text(&*contents.borrow(), statements)?;
        }
        NodeData::Comment { .. } => {
            eprintln!(
                "Ignoring comment: children: {:?}",
                node.children.borrow().len()
            );
        }
        NodeData::ProcessingInstruction { .. } => {
            eprintln!(
                "Ignoring processing instruction: children: {:?}",
                node.children.borrow().len()
            );
        }
    }
    Ok(())
}

fn walk_children(statements: &mut Vec<TokenStream2>, nodes: &[Handle]) -> Result<(), Error> {
    for child in nodes.iter() {
        walk_dom(statements, &child)?;
    }

    Ok(())
}

fn walk_element(
    name: &QualName,
    children: &[Handle],
    statements: &mut Vec<TokenStream2>,
) -> Result<(), Error> {
    let localname = name.local.to_string();
    // eprintln!("Start Element {:?}", name);
    statements.push(quote!(
                target.start_element(#localname.into())?;
            ));

    walk_children(statements, children)?;

    statements.push(quote!(
                target.end_element(#localname.into())?;
            ));
    // eprintln!("End Element {:?}", name);

    Ok(())
}
fn walk_text(contents: &StrTendril, statements: &mut Vec<TokenStream2>) -> Result<(), Error> {
    let cdata = contents.to_string();
    // eprintln!("Text {:?}", cdata);
    statements.push(quote!(
                target.text(#cdata)?;
            ));
    Ok(())
}
