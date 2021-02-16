use std::io;

use v_htmlescape::escape;

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

/// An attribute name and value pair.
#[derive(Debug)]
pub struct AttrPair {
    name: String,
    value: String,
}

/// A convenience type alias, to make implementing [`RenderTarget`] a little easier.
pub type IoResult<T> = Result<T, io::Error>;

/// Something that we can use to actually render HTML to text.
pub trait RenderTarget {
    /// The type used for render the attribute list.
    type StartElement: StartElementTarget;
    /// Open an element with the given name and attributes.
    fn start_element(self, name: QName) -> IoResult<Self::StartElement>;
    /// Write plain text content.
    fn text(self, content: &str) -> IoResult<()>;
    /// Close an element.
    fn end_element(self, name: QName) -> IoResult<()>;
}

/// The helper trait used to render the attribute list when opening an element.
/// Used by [`RenderTarget`]
pub trait StartElementTarget {
    /// Emits a single attribute
    fn attribute(&mut self, pair: &AttrPair) -> IoResult<()>;
    /// This closes the start of the element.
    fn close(self) -> IoResult<()>;
}

/// This is designed to be implemented via the `weft_derive` crate,
/// but can be implemented manually for special cases.
pub trait WeftRenderable {
    /// Outputs a representation of this object to the target.
    fn render_to<T>(&self, target: &mut T) -> IoResult<()>
    where
        for<'t> &'t mut T: RenderTarget;
}

impl<'a, R: WeftRenderable> WeftRenderable for &'a R {
    fn render_to<T>(&self, target: &mut T) -> Result<(), io::Error>
    where
        for<'t> &'t mut T: RenderTarget,
    {
        (**self).render_to(target)
    }
}

struct Html5Ser<T>(T);
struct Html5Element<'a, T>(&'a mut Html5Ser<T>);

impl<'a, T: io::Write> RenderTarget for &'a mut Html5Ser<T> {
    type StartElement = Html5Element<'a, T>;

    fn start_element(self, name: QName) -> Result<Self::StartElement, io::Error> {
        self.0.write_all(b"<")?;
        self.0.write_all(name.0.as_bytes())?;

        Ok(Html5Element(self))
    }
    fn text(self, content: &str) -> Result<(), io::Error> {
        write!(self.0, "{}", escape(content))?;
        Ok(())
    }
    fn end_element(self, name: QName) -> Result<(), io::Error> {
        self.0.write_all(b"</")?;
        self.0.write_all(name.0.as_bytes())?;
        self.0.write_all(b">")?;
        Ok(())
    }
}

impl<'a, T: 'a + io::Write> StartElementTarget for Html5Element<'a, T> {
    fn attribute(&mut self, attr: &AttrPair) -> Result<(), io::Error> {
        self.0 .0.write_all(b" ")?;
        self.0 .0.write_all(attr.name.as_bytes())?;
        self.0 .0.write_all(b"=")?;
        write!(self.0 .0, "\"{}\"", escape(&attr.value))?;
        Ok(())
    }

    fn close(self) -> Result<(), io::Error> {
        self.0 .0.write_all(b">")?;
        Ok(())
    }
}

impl AttrPair {
    /// Builds an attribute from a local-name and a value convertible to a string.
    pub fn new<S: ToString>(local_name: &str, value: S) -> Self {
        AttrPair {
            name: local_name.into(),
            value: value.to_string(),
        }
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
