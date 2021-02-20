use crate::inline_parse::{parse_inline, Segment, Substitutable};
use anyhow::Error;
use kuchiki::iter::Siblings;
use kuchiki::{ElementData, ExpandedName, NodeData, NodeRef};
use log::*;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use quote::TokenStreamExt;
use syn::{parse_quote, Token};

#[derive(Default, Debug)]
struct Walker;

#[derive(Default, Debug)]
struct Attribute {
    name: String,
    value: Substitutable,
}

#[derive(Default, Debug)]
struct Directives {
    replacement: Option<syn::Expr>,
    content: Option<syn::Expr>,
    conditional: Option<syn::Expr>,
    iterator: Option<IteratorDecl>,
    plain_attrs: Vec<Attribute>,
}

#[derive(Debug)]
struct IteratorDecl {
    pattern: syn::Pat,
    in_: Token![in],
    expr: syn::Expr,
}

fn render_to_fn(nodes: NodeRef) -> Result<TokenStream2, Error> {
    let walker = Walker::default();
    let impl_body = walker.children(nodes.children())?;
    Ok(quote! {
            fn render_to(&self, mut __weft_target: &mut impl ::weft::RenderTarget) -> Result<(), ::std::io::Error> {
                use ::weft::prelude::*;
                #impl_body;
                Ok(())
            }
    })
}

pub fn derive_impl(nodes: NodeRef, mut item: syn::DeriveInput) -> Result<TokenStream2, Error> {
    info!("Deriving implementation for {}", item.ident);
    let render_to_fn_impl = render_to_fn(nodes)?;

    info!("Generics before: {:#?}", item.generics);
    let bounds = item
        .generics
        .type_params()
        .map(|p| {
            let name = &p.ident;
            parse_quote!(#name : ::weft::WeftRenderable)
        })
        .collect::<Vec<syn::WherePredicate>>();

    {
        let where_clause = item
            .generics
            .where_clause
            .get_or_insert(parse_quote!(where));
        for clause in bounds {
            where_clause.predicates.push(clause);
        }
    }
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

    let ident = &item.ident;
    let res = quote! {
        impl #impl_generics ::weft::WeftRenderable for #ident #ty_generics #where_clause {
            #render_to_fn_impl
        }
    };
    debug!("Impl: {}", res);
    Ok(res)
}

impl Walker {
    fn dom(&self, node: NodeRef) -> Result<TokenStream2, Error> {
        match node.data() {
            NodeData::Document(_) => {
                let ts = self.children(node.children())?;
                trace!("Document => {}", ts);
                Ok(ts)
            }
            NodeData::Element(data) => {
                trace!("Element: {:?}", data);
                let ts = self.element(data, node.children())?;
                trace!("Element => {}", ts);
                Ok(ts)
            }
            NodeData::Text(ref contents) => {
                let ts = self.text(&*contents.borrow())?;
                trace!("Text => {}", ts);
                Ok(ts)
            }
            NodeData::Doctype { .. } => {
                debug!("Ignoring doctype: children: {:?}", node.children().count());
                Ok(TokenStream2::default())
            }
            NodeData::Comment { .. } => {
                debug!("Ignoring comment: children: {:?}", node.children().count());
                Ok(TokenStream2::default())
            }
            NodeData::ProcessingInstruction { .. } => {
                debug!(
                    "Ignoring processing instruction: children: {:?}",
                    node.children().count()
                );
                Ok(TokenStream2::default())
            }
            NodeData::DocumentFragment => {
                debug!("Ignoring document fragment: {:?}", node.children().count());
                Ok(TokenStream2::default())
            }
        }
    }

    fn children(&self, nodes: Siblings) -> Result<TokenStream2, Error> {
        let mut res = TokenStream2::new();
        for child in nodes {
            res.extend(self.dom(child)?);
        }

        Ok(res)
    }

    fn element(&self, data: &ElementData, children: Siblings) -> Result<TokenStream2, Error> {
        let localname = data.name.local.to_string();
        trace!("Start Element {:?}", data);

        let directive = Directives::parse_from_attrs(&data.attributes.borrow())?;
        let res = if let Some(repl) = directive.replacement {
            quote!(#repl.render_to(&mut __weft_target)?;)
        } else if let Some(content) = directive.content {
            let content = quote!(#content.render_to(&mut __weft_target)?;);
            self.emit_element(&localname, &*directive.plain_attrs, content)
        } else {
            let content = self.children(children)?;
            self.emit_element(&localname, &*directive.plain_attrs, content)
        };

        let res = if let Some(iter) = directive.iterator {
            quote!(for #iter { #res }; )
        } else {
            res
        };

        let res = if let Some(test) = directive.conditional {
            quote!(if #test { #res }; )
        } else {
            res
        };
        trace!("End Element {:?}", data);

        Ok(res)
    }
    fn text(&self, contents: &str) -> Result<TokenStream2, Error> {
        let mut result = TokenStream2::new();
        let cdata = contents.to_string();
        trace!("Text {:?}", cdata);
        let parsed = parse_inline(&cdata)?;
        for segment in parsed.children() {
            match segment {
                Segment::Literal(cdata) => {
                    let chunk = quote!(__weft_target.text(#cdata)?;);
                    result.extend(chunk);
                }
                Segment::Expr(expr) => {
                    let chunk = quote!(#expr.render_to(&mut __weft_target)?;);
                    result.extend(chunk);
                }
            }
        }
        Ok(result)
    }

    fn emit_element(
        &self,
        localname: &str,
        attrs: &[Attribute],
        content: TokenStream2,
    ) -> proc_macro2::TokenStream {
        let attrs_q = quote!(&[#(&#attrs),*]);
        let mut statements = TokenStream2::new();
        statements.extend(quote!(
            __weft_target.start_element_attrs(#localname.into(), #attrs_q)?;
        ));

        statements.extend(content);

        statements.extend(quote!(
            __weft_target.end_element(#localname.into())?;
        ));
        statements
    }
}

impl Directives {
    fn parse_from_attrs(attrs: &kuchiki::Attributes) -> Result<Self, Error> {
        let mut it = Self::default();
        for (name, value) in attrs.map.iter() {
            match &*name.local {
                "weft-replace" => {
                    let replacement = syn::parse_str(&value.value)?;
                    it.replacement = Some(replacement)
                }
                "weft-content" => {
                    let content = syn::parse_str(&value.value)?;
                    it.content = Some(content)
                }
                "weft-if" => {
                    let test = syn::parse_str(&value.value)?;
                    it.conditional = Some(test)
                }
                "weft-for" => {
                    let iterator = syn::parse_str(&value.value)?;
                    it.iterator = Some(iterator)
                }
                _ => it.plain_attrs.push(Attribute::parse(name, value)?),
            }
        }

        Ok(it)
    }
}

impl Attribute {
    fn parse(name: &ExpandedName, value: &kuchiki::Attribute) -> Result<Self, Error> {
        let name: String = name.local.to_string();
        let value = parse_inline(&value.value)?;

        Ok(Attribute { name, value })
    }
}
impl quote::ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let key_name: String = self.name.to_string();

        let str_iter_q = self
            .value
            .children()
            .map(|segment| match segment {
                Segment::Literal(cdata) => quote!(#cdata),
                Segment::Expr(expr) => quote!(#expr.to_string()),
            }).fold(
                quote!(::std::iter::empty::<::std::borrow::Cow<str>>()),
                |prefix, it| quote!(#prefix.chain(::std::iter::once(::std::borrow::Cow::from(#it)))),
            );

        let value = quote!(#str_iter_q.collect::<String>());
        tokens.append_all(quote!(::weft::AttrPair::new(#key_name, #value)))
    }
}

impl quote::ToTokens for IteratorDecl {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        self.pattern.to_tokens(tokens);
        self.in_.to_tokens(tokens);
        self.expr.to_tokens(tokens);
    }
}

impl syn::parse::Parse for IteratorDecl {
    fn parse(buf: &syn::parse::ParseBuffer<'_>) -> Result<Self, syn::parse::Error> {
        let pattern = buf.parse()?;
        let in_ = buf.parse()?;
        let expr = buf.parse()?;
        Ok(IteratorDecl { pattern, in_, expr })
    }
}
