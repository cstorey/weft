use html5ever;
use std::io;
use std::iter;

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
        html5ever::QualName::new(
            None, ns!(), html5ever::LocalName::from(self.0.clone()),
        )
    }
}

pub trait RenderTarget {
    fn start_element(&mut self, name: QName) -> Result<(), io::Error>;
    fn text(&mut self, content: &str) -> Result<(), io::Error>;
    fn end_element(&mut self, name: QName) -> Result<(), io::Error>;
}
pub trait Renderable {
    fn render_to<T: RenderTarget>(&self, target: T) -> Result<(), io::Error>;
}

struct Html5Wrapper<R> {
    inner: R,
}
struct Html5Ser<'a, T: 'a>(&'a mut T);

impl<'a, T: 'a + html5ever::serialize::Serializer> RenderTarget for Html5Ser<'a, T> {
    fn start_element(&mut self, name: QName) -> Result<(), io::Error> {
        self.0.start_elem(name.as_qual_name(), iter::empty())
    }
    fn text(&mut self, content: &str) -> Result<(), io::Error> {
        self.0.write_text(content)
    }
    fn end_element(&mut self, name: QName) -> Result<(), io::Error> {
        self.0.end_elem(name.as_qual_name())
    }
}

impl<R: Renderable> html5ever::serialize::Serialize for Html5Wrapper<R> {
    fn serialize<S>(
        &self,
        serializer: &mut S,
        _: html5ever::serialize::TraversalScope,
    ) -> Result<(), io::Error>
    where
        S: html5ever::serialize::Serializer,
    {
        self.inner.render_to(Html5Ser(serializer))?;
        Ok(())
    }
}

pub fn render_writer<R: Renderable, W: io::Write>(widget: R, out: W) -> Result<(), io::Error> {
    let nodes = Html5Wrapper { inner: widget };
    html5ever::serialize::serialize(out, &nodes, Default::default())?;
    Ok(())
}

pub fn render_to_string<R: Renderable>(widget: R) -> Result<String, io::Error> {
    let mut out = Vec::new();
    render_writer(widget, &mut out)?;
    Ok(String::from_utf8_lossy(&out).into_owned())
}

