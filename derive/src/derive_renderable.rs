use failure::Error;
use html5ever::rcdom::{Handle, NodeData};
use html5ever::tendril::StrTendril;
use html5ever::QualName;
use proc_macro2::TokenStream as TokenStream2;

#[derive(Default, Debug)]
struct Walker {
    statements: Vec<TokenStream2>,
}

impl Walker {
    fn into_body(self) -> TokenStream2 {
        let mut body = TokenStream2::new();
        body.extend(self.statements);
        return body;
    }
}

pub fn template_fn_body(nodes: &[Handle]) -> Result<TokenStream2, Error> {
    info!("Deriving implementation");
    let mut walker = Walker::default();
    walker.children(nodes)?;
    Ok(walker.into_body())
}

impl Walker {
    fn dom(&mut self, node: &Handle) -> Result<(), Error> {
        match node.data {
            NodeData::Document => {
                self.children(&node.children.borrow())?;
            }
            NodeData::Doctype { .. } => {
                debug!(
                    "Ignoring doctype: children: {:?}",
                    node.children.borrow().len()
                );
            }
            NodeData::Element {
                ref name,
                ref attrs,
                ..
            } => {
                self.element(name, &attrs.borrow(), &node.children.borrow())?;
            }
            NodeData::Text { ref contents } => {
                self.text(&*contents.borrow())?;
            }
            NodeData::Comment { .. } => {
                debug!(
                    "Ignoring comment: children: {:?}",
                    node.children.borrow().len()
                );
            }
            NodeData::ProcessingInstruction { .. } => {
                debug!(
                    "Ignoring processing instruction: children: {:?}",
                    node.children.borrow().len()
                );
            }
        }
        Ok(())
    }

    fn children(&mut self, nodes: &[Handle]) -> Result<(), Error> {
        for child in nodes.iter() {
            self.dom(&child)?;
        }

        Ok(())
    }

    fn element(
        &mut self,
        name: &QualName,
        attrs: &[html5ever::Attribute],
        children: &[Handle],
    ) -> Result<(), Error> {
        let localname = name.local.to_string();
        trace!("Start Element {:?}: {:?}", name, attrs);

        let mut plain_attrs = Vec::new();

        let mut replace_content = None;

        for at in attrs {
            match &*at.name.local {
                "weft-replace" => replace_content = Some(at.value.to_string()),
                _ => plain_attrs.push(at),
            }
        }

        if let Some(repl) = replace_content {
            let val = repl
                .parse::<TokenStream2>()
                .map_err(|e| failure::err_msg(format!("{:?}", e)))?;
            let q = quote!(#val.render_to(target)?;);

            self.statements.push(q);
        } else {
            let attrs_quotes = plain_attrs.into_iter().map(|at| (at)).map(|at| {
                let key_name: String = at.name.local.to_string();
                let value: String = at.value.to_string();
                quote!(::std::iter::once(&::weft::AttrPair::new(#key_name, #value)))
            });

            let attrs_q = attrs_quotes.fold(
                quote!(::std::iter::empty()),
                |prefix, it| quote!(#prefix.chain(#it)),
            );
            self.statements.push(quote!(
                target.start_element_attrs(#localname.into(), #attrs_q)?;
            ));

            self.children(children)?;

            self.statements.push(quote!(
                target.end_element(#localname.into())?;
            ));
            trace!("End Element {:?}", name);
        }
        Ok(())
    }
    fn text(&mut self, contents: &StrTendril) -> Result<(), Error> {
        let cdata = contents.to_string();
        trace!("Text {:?}", cdata);
        self.statements.push(quote!(
                target.text(#cdata)?;
            ));
        Ok(())
    }
}
