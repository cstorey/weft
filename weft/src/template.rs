use std::{borrow::Cow, io};

use v_htmlescape::escape;

/// An internal representation of a qualified name, such as a tag or attribute.
/// Does not currently support namespaces.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct QName<'a>(Cow<'a, str>);

impl From<String> for QName<'static> {
    fn from(src: String) -> Self {
        QName(src.into())
    }
}

impl<'a> From<&'a str> for QName<'a> {
    fn from(src: &'a str) -> Self {
        QName(src.into())
    }
}

impl<'a> QName<'a> {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

/// An attribute name and value pair.
#[derive(Debug)]
pub struct AttrPair<'n, 'v> {
    name: QName<'n>,
    value: Cow<'v, str>,
}

/// Something that we can use to actually render HTML to text.
///
pub trait RenderTarget {
    /// Open an element with the given name and attributes.
    fn start_element_attrs(&mut self, name: QName, attrs: &[&AttrPair]) -> Result<(), io::Error>;
    /// Write plain text content.
    fn text(&mut self, content: &str) -> Result<(), io::Error>;
    /// Close an element.
    fn end_element(&mut self, name: QName) -> Result<(), io::Error>;
}

/// This is designed to be implemented via the `weft_derive` crate,
/// but can be implemented manually for special cases.
pub trait WeftRenderable {
    /// Outputs a representation of this object to the target.
    fn render_to(&self, target: &mut impl RenderTarget) -> Result<(), io::Error>;
}

impl<'a, T: RenderTarget> RenderTarget for &'a mut T {
    fn start_element_attrs(&mut self, name: QName, attrs: &[&AttrPair]) -> Result<(), io::Error> {
        (**self).start_element_attrs(name, attrs)
    }
    fn text(&mut self, content: &str) -> Result<(), io::Error> {
        (**self).text(content)
    }
    fn end_element(&mut self, name: QName) -> Result<(), io::Error> {
        (**self).end_element(name)
    }
}

impl<'a, R: WeftRenderable> WeftRenderable for &'a R {
    fn render_to(&self, target: &mut impl RenderTarget) -> Result<(), io::Error> {
        (**self).render_to(target)
    }
}

struct Html5Ser<T>(T);

impl<'a, T: 'a + io::Write> RenderTarget for Html5Ser<T> {
    fn start_element_attrs(&mut self, name: QName, attrs: &[&AttrPair]) -> Result<(), io::Error> {
        self.0.write_all(b"<")?;
        self.0.write_all(name.0.as_bytes())?;

        for attr in attrs {
            // TODO: Escaping!
            self.0.write_all(b" ")?;
            self.0.write_all(attr.name.as_bytes())?;
            self.0.write_all(b"=")?;
            write!(self.0, "\"{}\"", escape(&attr.value))?;
        }
        self.0.write_all(b">")?;
        Ok(())
    }
    fn text(&mut self, content: &str) -> Result<(), io::Error> {
        write!(self.0, "{}", escape(content))?;
        Ok(())
    }
    fn end_element(&mut self, name: QName) -> Result<(), io::Error> {
        self.0.write_all(b"</")?;
        self.0.write_all(name.0.as_bytes())?;
        self.0.write_all(b">")?;
        Ok(())
    }
}

impl<'n, 'v> AttrPair<'n, 'v> {
    /// Builds an attribute from a local-name and a value convertible to a string.
    pub fn new(name: QName<'n>, value: Cow<'v, str>) -> Self {
        AttrPair { name, value }
    }
}

/// Renders the template in `widget` to the writer `out`.
pub fn render_writer<R: WeftRenderable, W: io::Write>(widget: R, out: W) -> Result<(), io::Error> {
    let mut ser = Html5Ser(out);
    widget.render_to(&mut ser)?;
    Ok(())
}

/// Renders the template in `widget` to a new String.
pub fn render_to_string<R: WeftRenderable>(widget: R) -> Result<String, io::Error> {
    let mut out = Vec::new();
    render_writer(widget, &mut out)?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}
