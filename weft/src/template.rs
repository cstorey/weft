use html5ever;
use html5ever::QualName;
use std::io;
use std::iter;

/// An internal representation of a qualified name, such as a tag or attribute.
/// Does not currently support namespaces.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QName(String);

impl From<String> for QName {
    fn from(src: String) -> Self {
        QName(src)
    }
}

impl<'a> From<&'a str> for QName {
    fn from(src: &'a str) -> Self {
        QName(src.to_string())
    }
}

impl QName {
    fn as_qual_name(&self) -> html5ever::QualName {
        html5ever::QualName::new(None, ns!(html), html5ever::LocalName::from(self.0.clone()))
    }
}

/// An attribute name and value pair.
pub struct AttrPair {
    name: QualName,
    value: String,
}

/// Something that we can use to actually render HTML to text.
///
pub trait RenderTarget {
    /// Open an element with the given name and attributes.
    fn start_element_attrs<'a, I: IntoIterator<Item = &'a AttrPair>>(
        &mut self,
        name: QName,
        attrs: I,
    ) -> Result<(), io::Error>;
    /// Write plain text content.
    fn text(&mut self, content: &str) -> Result<(), io::Error>;
    /// Close an element.
    fn end_element(&mut self, name: QName) -> Result<(), io::Error>;
}

/// This is designed to be implemented via the `weft_derive` crate,
/// but can be implemented manually for special cases.
pub trait WeftRenderable {
    /// Outputs a representation of this object to the target.
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error>;
}

impl<'a, R: WeftRenderable> WeftRenderable for &'a R {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        (**self).render_to(target)
    }
}

struct Html5Wrapper<R> {
    inner: R,
}
struct Html5Ser<'a, T: 'a>(&'a mut T);

impl<'a, T: 'a + html5ever::serialize::Serializer> RenderTarget for Html5Ser<'a, T> {
    fn start_element_attrs<'attr, I: IntoIterator<Item = &'attr AttrPair>>(
        &mut self,
        name: QName,
        attrs: I,
    ) -> Result<(), io::Error> {
        self.0.start_elem(
            name.as_qual_name(),
            attrs.into_iter().map(|a| (&a.name, &*a.value)),
        )
    }
    fn text(&mut self, content: &str) -> Result<(), io::Error> {
        self.0.write_text(content)
    }
    fn end_element(&mut self, name: QName) -> Result<(), io::Error> {
        self.0.end_elem(name.as_qual_name())
    }
}

impl<R: WeftRenderable> html5ever::serialize::Serialize for Html5Wrapper<R> {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        _: html5ever::serialize::TraversalScope,
    ) -> Result<(), io::Error>
    where
        S: html5ever::serialize::Serializer,
    {
        self.inner.render_to(&mut Html5Ser(serializer))?;
        Ok(())
    }
}

impl AttrPair {
    /// Builds an attribute from a local-name and a value convertible to a string.
    pub fn new<S: ToString>(local_name: &str, value: S) -> Self {
        let qual = QualName::new(None, ns!(), html5ever::LocalName::from(local_name));
        AttrPair {
            name: qual,
            value: value.to_string(),
        }
    }
}

/// Renders the template in `widget` to the writer `out`.
pub fn render_writer<R: WeftRenderable, W: io::Write>(widget: R, out: W) -> Result<(), io::Error> {
    let nodes = Html5Wrapper { inner: widget };
    html5ever::serialize::serialize(out, &nodes, Default::default())?;
    Ok(())
}

/// Renders the template in `widget` to a new String.
pub fn render_to_string<R: WeftRenderable>(widget: R) -> Result<String, io::Error> {
    let mut out = Vec::new();
    render_writer(widget, &mut out)?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}
