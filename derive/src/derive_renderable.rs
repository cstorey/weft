use failure::Error;
use html5ever::rcdom::{Handle, NodeData};
use html5ever::tendril::StrTendril;
use html5ever::QualName;
use proc_macro2::TokenStream as TokenStream2;
use syn;

#[derive(Default, Debug)]
struct Walker;

#[derive(Default, Debug)]
struct Directives<'a> {
    replacement: Option<syn::Expr>,
    content: Option<syn::Expr>,
    conditional: Option<syn::Expr>,
    iterator: Option<IteratorDecl>,
    plain_attrs: Vec<&'a html5ever::Attribute>,
}

#[derive(Debug)]
struct IteratorDecl {
    pattern: syn::Pat,
    in_: Token![in],
    expr: syn::Expr,
}

fn render_to_fn(nodes: &[Handle]) -> Result<TokenStream2, Error> {
    let walker = Walker::default();
    let impl_body = walker.children(nodes)?;
    Ok(quote! {
            fn render_to<__weft_R: ::weft::RenderTarget>(&self, __weft_target: &mut __weft_R) -> Result<(), ::std::io::Error> {
                #impl_body;
                Ok(())
            }
    })
}

pub fn derive_impl(nodes: &[Handle], mut item: syn::DeriveInput) -> Result<TokenStream2, Error> {
    info!("Deriving implementation for {}", item.ident);
    let render_to_fn_impl = render_to_fn(nodes)?;
    debug!("Fn body: {}", render_to_fn_impl);

    info!("Generics before: {:#?}", item.generics);
    let bounds = item
        .generics
        .type_params()
        .map(|p| {
            let name = &p.ident;
            parse_quote!(#name : ::weft::Renderable)
        }).collect::<Vec<syn::WherePredicate>>();

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
        impl #impl_generics ::weft::Renderable for #ident #ty_generics #where_clause {
            #render_to_fn_impl
        }
    };
    Ok(res)
}

impl Walker {
    fn dom(&self, node: &Handle) -> Result<TokenStream2, Error> {
        match node.data {
            NodeData::Document => {
                let ts = self.children(&node.children.borrow())?;
                trace!("Document => {}", ts);
                Ok(ts)
            }
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                let ts = self.element(name, &attrs.borrow(), &node.children.borrow())?;
                trace!("Element: {:?}:{:?}", name, attrs);
                trace!("Element => {}", ts);
                Ok(ts)
            }
            NodeData::Text { ref contents } => {
                let ts = self.text(&*contents.borrow())?;
                trace!("Text => {}", ts);
                Ok(ts)
            }
            NodeData::Doctype { .. } => {
                debug!(
                    "Ignoring doctype: children: {:?}",
                    node.children.borrow().len()
                );
                Ok(TokenStream2::default())
            }
            NodeData::Comment { .. } => {
                debug!(
                    "Ignoring comment: children: {:?}",
                    node.children.borrow().len()
                );
                Ok(TokenStream2::default())
            }
            NodeData::ProcessingInstruction { .. } => {
                debug!(
                    "Ignoring processing instruction: children: {:?}",
                    node.children.borrow().len()
                );
                Ok(TokenStream2::default())
            }
        }
    }

    fn children(&self, nodes: &[Handle]) -> Result<TokenStream2, Error> {
        let mut res = TokenStream2::new();
        for child in nodes.iter() {
            res.extend(self.dom(&child)?);
        }

        Ok(res)
    }

    fn element(
        &self,
        name: &QualName,
        attrs: &[html5ever::Attribute],
        children: &[Handle],
    ) -> Result<TokenStream2, Error> {
        let localname = name.local.to_string();
        trace!("Start Element {:?}: {:?}", name, attrs);

        let directive = Directives::parse_from_attrs(attrs)?;
        let res = if let Some(repl) = directive.replacement {
            quote!(#repl.render_to(__weft_target)?;)
        } else if let Some(content) = directive.content {
            let content = quote!(#content.render_to(__weft_target)?;);
            self.emit_element(&localname, &*directive.plain_attrs, content)?
        } else {
            let content = self.children(children)?;
            self.emit_element(&localname, &*directive.plain_attrs, content)?
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
        trace!("End Element {:?}", name);

        Ok(res)
    }
    fn text(&self, contents: &StrTendril) -> Result<TokenStream2, Error> {
        let cdata = contents.to_string();
        trace!("Text {:?}", cdata);
        Ok(quote!(
                __weft_target.text(#cdata)?;
            ))
    }

    fn emit_element(
        &self,
        localname: &str,
        attrs: &[&html5ever::Attribute],
        content: TokenStream2,
    ) -> Result<TokenStream2, Error> {
        let attrs_quotes = attrs.iter().map(|at| at).map(|at| {
            let key_name: String = at.name.local.to_string();
            let value: String = at.value.to_string();
            quote!(::std::iter::once(&::weft::AttrPair::new(#key_name, #value)))
        });

        let attrs_q = attrs_quotes.fold(
            quote!(::std::iter::empty()),
            |prefix, it| quote!(#prefix.chain(#it)),
        );
        let mut statements = TokenStream2::new();
        statements.extend(quote!(
                __weft_target.start_element_attrs(#localname.into(), #attrs_q)?;
            ));

        statements.extend(content);

        statements.extend(quote!(
                __weft_target.end_element(#localname.into())?;
            ));
        Ok(statements)
    }
}

impl<'a> Directives<'a> {
    fn parse_from_attrs(attrs: &'a [html5ever::Attribute]) -> Result<Self, Error> {
        let mut it = Self::default();
        for at in attrs {
            match &*at.name.local {
                "weft-replace" => {
                    let replacement = syn::parse_str(at.value.as_ref())
                        .map_err(|e| failure::err_msg(format!("{:?}", e)))?;
                    it.replacement = Some(replacement)
                }
                "weft-content" => {
                    let content = syn::parse_str(at.value.as_ref())
                        .map_err(|e| failure::err_msg(format!("{:?}", e)))?;
                    it.content = Some(content)
                }
                "weft-if" => {
                    let test = syn::parse_str(at.value.as_ref())
                        .map_err(|e| failure::err_msg(format!("{:?}", e)))?;
                    it.conditional = Some(test)
                }
                "weft-for" => {
                    let iterator = syn::parse_str(at.value.as_ref())
                        .map_err(|e| failure::err_msg(format!("{:?}", e)))?;
                    it.iterator = Some(iterator)
                }
                _ => it.plain_attrs.push(&at),
            }
        }

        Ok(it)
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
    fn parse(buf: &syn::parse::ParseBuffer) -> Result<Self, syn::parse::Error> {
        let pattern = buf.parse()?;
        let in_ = buf.parse()?;
        let expr = buf.parse()?;
        Ok(IteratorDecl { pattern, in_, expr })
    }
}
