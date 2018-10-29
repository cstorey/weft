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
    replacement: Option<TokenStream2>,
    content: Option<TokenStream2>,
    conditional: Option<TokenStream2>,
    iterator: Option<TokenStream2>,
    plain_attrs: Vec<&'a html5ever::Attribute>,
}

fn render_to_fn(nodes: &[Handle]) -> Result<TokenStream2, Error> {
    let walker = Walker::default();
    let impl_body = walker.children(nodes)?;
    Ok(quote! {
            fn render_to<__weft_R: ::weft::RenderTarget>(&self, target: &mut __weft_R) -> Result<(), ::std::io::Error> {
                #impl_body;
                Ok(())
            }
    })
}

pub fn derive_impl(nodes: &[Handle], item: &syn::DeriveInput) -> Result<TokenStream2, Error> {
    info!("Deriving implementation for {}", item.ident);
    let render_to_fn_impl = render_to_fn(nodes)?;
    debug!("Fn body: {}", render_to_fn_impl);

    let ident = &item.ident;
    let (impl_generics, ty_generics, where_clause) = item.generics.split_for_impl();

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
            quote!(#repl.render_to(target)?;)
        } else if let Some(content) = directive.content {
            let content = quote!(#content.render_to(target)?;);
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
                target.text(#cdata)?;
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
                target.start_element_attrs(#localname.into(), #attrs_q)?;
            ));

        statements.extend(content);

        statements.extend(quote!(
                target.end_element(#localname.into())?;
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
                    let replacement = at
                        .value
                        .as_ref()
                        .parse::<TokenStream2>()
                        .map_err(|e| failure::err_msg(format!("{:?}", e)))?;
                    it.replacement = Some(replacement)
                }
                "weft-content" => {
                    let content = at
                        .value
                        .as_ref()
                        .parse::<TokenStream2>()
                        .map_err(|e| failure::err_msg(format!("{:?}", e)))?;
                    it.content = Some(content)
                }
                "weft-if" => {
                    let test = at
                        .value
                        .as_ref()
                        .parse::<TokenStream2>()
                        .map_err(|e| failure::err_msg(format!("{:?}", e)))?;
                    it.conditional = Some(test)
                }
                "weft-for" => {
                    let iterator = at
                        .value
                        .as_ref()
                        .parse::<TokenStream2>()
                        .map_err(|e| failure::err_msg(format!("{:?}", e)))?;
                    it.iterator = Some(iterator)
                }
                _ => it.plain_attrs.push(&at),
            }
        }

        Ok(it)
    }
}
