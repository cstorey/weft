/*!
# `weft-derive`.
This module provides compiler support for creating `weft` templates. See the `weft` module for usage.
*/

extern crate proc_macro;
extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;
extern crate env_logger;
#[macro_use]
extern crate syn;
extern crate html5ever;
extern crate kuchiki;
extern crate regex;

mod derive_renderable;
mod inline_parse;
use derive_renderable::*;

use failure::{Error, ResultExt};
use html5ever::tendril::TendrilSink;
use kuchiki::NodeRef;
use proc_macro::TokenStream;
use quote::ToTokens;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Eq, PartialEq)]
enum TemplateSource {
    Path(PathBuf),
    Source(String),
}

#[derive(Debug, Clone)]
struct TemplateDerivation {
    template_source: TemplateSource,
    selector: String,
}

/// Derives a `WeftRenderable` instance from a given html template.
///
/// Requires the user pass an additional `#[template(...)]` attribute to
/// specify either a path (relative to the crate root) or template source.
///
/// ## Configuration
/// Options are specified as `parameter = value`, and have the following meanings:
///
/// ### Finding the template source.
/// * `path`: The path of the template relative to the crate root.
///           Must be present at compile time.
/// * `source`: The template source specified inline as a string.
///
/// One of `path` or `source` must be specified.

#[proc_macro_derive(WeftRenderable, attributes(template))]
pub fn derive_template(input: TokenStream) -> TokenStream {
    // Theoretically `rustc` provides it's own logging, but we
    // don't know for sure that we're using the same `log` crate. So, just in case?
    env_logger::try_init().unwrap_or_default();
    let ast: syn::DeriveInput = syn::parse(input).unwrap();
    match make_template(ast) {
        Ok(toks) => toks.into(),
        Err(err) => panic!("Error: {:?}", err),
    }
}

fn make_template(item: syn::DeriveInput) -> Result<proc_macro2::TokenStream, Error> {
    info!("Deriving for {}", item.ident);
    trace!("{:#?}", item);
    let config = TemplateDerivation::from_derive(&item).context("find template")?;
    let dom = config.load_relative_to(&root_dir())?;

    let impl_body = derive_impl(dom, item)?;

    Ok(impl_body.into_token_stream())
}

fn parse_path(path: &Path) -> Result<NodeRef, Error> {
    info!("Using template from {:?}", path);
    let parser = kuchiki::parse_html().from_utf8();

    let root = parser
        .from_file(&path)
        .with_context(|_| format!("Parsing template from path {:?}", &path))?;

    Ok(root)
}

fn parse_source(source: &str) -> Result<NodeRef, Error> {
    info!("Using inline template");
    let parser = kuchiki::parse_html();

    let root = parser.one(source);

    Ok(root)
}

impl TemplateDerivation {
    fn from_derive(item: &syn::DeriveInput) -> Result<TemplateDerivation, Error> {
        let attr = item
            .attrs
            .iter()
            .filter_map(|a| a.interpret_meta())
            .inspect(|a| info!("Attribute: {:#?}", a))
            .find(|a| a.name() == "template")
            .ok_or_else(|| failure::err_msg("Could not find template attribute"))?;

        let meta_list = match attr {
            syn::Meta::List(inner) => inner,
            _ => return Err(failure::err_msg("template attribute incorrectly formatted")),
        };

        let mut path = None;
        let mut source = None;
        let mut template_selector = None;
        for meta in meta_list.nested {
            if let syn::NestedMeta::Meta(ref item) = meta {
                if let syn::Meta::NameValue(ref pair) = item {
                    match pair.ident.to_string().as_ref() {
                        "path" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                path = Some(PathBuf::from(s.value()));
                            } else {
                                return Err(failure::err_msg(
                                    "template path attribute should be a string",
                                ));
                            }
                        }
                        "source" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                source = Some(s.value())
                            } else {
                                return Err(failure::err_msg(
                                    "template path attribute should be a string",
                                ));
                            }
                        }
                        "selector" => {
                            if let syn::Lit::Str(ref s) = pair.lit {
                                template_selector = Some(s.value())
                            } else {
                                return Err(failure::err_msg(
                                    "template selector should be a string",
                                ));
                            }
                        }

                        _ => warn!("Unrecognised attribute {:#?}", pair),
                    }
                }
            }
        }

        let template_source = match (path, source) {
            (Some(path), None) => {
                TemplateSource::Path(path)
            },
            (None, Some(source)) => {
                TemplateSource::Source(source)
            },
            _ => bail!("Exactly one of `source` or `path` attributes must be specfied in `#[template(...)]")
        };

        let selector = template_selector.unwrap_or_else(|| ":root".to_string());

        let res = TemplateDerivation {
            template_source,
            selector,
        };

        Ok(res)
    }

    fn load_relative_to<P: AsRef<Path>>(&self, root_dir: P) -> Result<NodeRef, Error> {
        let root = match &self.template_source {
            TemplateSource::Path(ref path) => {
                let path = PathBuf::from(root_dir.as_ref()).join(path);
                parse_path(&path)?
            }
            TemplateSource::Source(ref source) => parse_source(source)?,
        };

        let content = self
            .find_root_from(root)
            .ok_or_else(|| failure::err_msg("Could not locate root of parsed document?"))?;

        Ok(content)
    }

    fn find_root_from(&self, node: NodeRef) -> Option<NodeRef> {
        if let Ok(mut roots) = node.select(&self.selector) {
            let first = roots.next()?;
            if let Some(_) = roots.next() {
                warn!("Selector {} returns more than one match", self.selector);
                return None;
            }

            return Some(first.as_node().clone());
        }

        None
    }
}

fn root_dir() -> PathBuf {
    std::env::var("CARGO_MANIFEST_DIR")
        .unwrap_or_else(|_| {
            warn!("Environment variable $CARGO_MANIFEST_DIR not set, assuming .");
            ".".into()
        })
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn can_parse_with_path() {
        let deriv = parse_quote!(
            #[template(path = "hello.html")]
            struct X;
        );

        let conf = TemplateDerivation::from_derive(&deriv).expect("parse derive");

        assert_eq!(
            conf.template_source,
            TemplateSource::Path(PathBuf::from("hello.html"))
        );
    }

    #[test]
    fn can_parse_with_source() {
        let source = "<p>Stuff</p>";
        let deriv = parse_quote!(
            #[template(source = #source)]
            struct X;
        );

        let conf = TemplateDerivation::from_derive(&deriv).expect("parse derive");

        assert_eq!(conf.template_source, TemplateSource::Source(source.into()));
    }

    #[test]
    fn cannot_parse_with_neither_source_or_path() {
        let deriv = quote!(
            #[template()]
            struct X;
        );

        let parsed = syn::parse2(deriv.clone()).expect("parse");
        let res = TemplateDerivation::from_derive(&parsed);
        assert!(res.is_err(), "Template {} should not parse", deriv)
    }

    #[test]
    fn cannot_parse_with_both_source_or_path() {
        let deriv = quote!(
            #[template(source = "...", path = "...")]
            struct X;
        );

        let parsed = syn::parse2(deriv.clone()).expect("parse");
        let res = TemplateDerivation::from_derive(&parsed);
        assert!(res.is_err(), "Template {} should not parse", deriv)
    }

    #[test]
    #[should_panic]
    fn cannot_parse_with_multiple_root_matches() {
        env_logger::try_init().unwrap_or_default();

        let _ = derive_template(
            quote!(
                #[template(source = "<p>foo</p><p>bar</p>", selector = "p")]
                struct MultipleRoots;
            )
            .into(),
        );
    }

    #[test]
    fn will_extract_selector() {
        let deriv = parse_quote!(
            #[template(path = "hello.html", selector = "#hello-world")]
            struct X;
        );

        let conf = TemplateDerivation::from_derive(&deriv).expect("parse derive");

        assert_eq!(conf.selector, "#hello-world");
    }
}
