use std::io;

use weft::{AttrPair, QName, RenderTarget, WeftRenderable};

struct ErasedRenderTarget<'a>(&'a mut dyn RenderTarget);

/// An erased version of [`weft::RenderTarget`], usable with a [`ErasedRenderable`]
pub trait ErasedRenderTarget {}
/// Erased version of [`weft::StartElementTarget`]
pub trait ErasedStartElementTarget {}


/// This is exactly like the [`weft::WeftRenderable`] trait, but for cases where
/// we need a trait object. Eg: for a `Vec<Box<dyn ErasedRenderable>>`.
pub trait ErasedRenderable {
    /// Outputs a representation of this object to the target.
    fn erased_render_to(&self, target: &mut dyn RenderTarget) -> Result<(), io::Error>;
}

impl StartElementTarget for Box<dyn ErasedStartElementTarget> {
    fn attribute(&mut self, pair: &AttrPair) -> IoResult<()> {
        (**self).erased_attribute(pair)
    }

    fn close(self) -> IoResult<()> {
        (**self).erased_close()
    }
}

impl<T> ErasedRenderable for T
where
    T: WeftRenderable,
{
    fn erased_render_to(&self, mut target: &mut dyn ErasedRenderTarget) -> Result<(), io::Error> {
        // self.render_to(target)
        fn assert_render_target<T: RenderTarget>(t: T) -> T { t }

        WeftRenderable::render_to(self, assert_render_target(target))
    }
}


impl WeftRenderable for dyn ErasedRenderable {
    fn render_to<T>(&self, target: &mut T) -> Result<(), io::Error>
    where
        for<'t> &'t mut T: RenderTarget {
        self.erased_render_to(target)
    }
}

impl<'a> RenderTarget for ErasedRenderTarget<'a> {
    fn start_element_attrs(&mut self, name: QName, attrs: &[&AttrPair]) -> Result<(), io::Error> {
        self.0.start_element_attrs(name, attrs)
    }
    fn text(&mut self, content: &str) -> Result<(), io::Error> {
        self.0.text(content)
    }
    fn end_element(&mut self, name: QName) -> Result<(), io::Error> {
        self.0.end_element(name)
    }
}

/// Renderer created from an anonymous function
pub struct FnRenderer<F>(F);

/// Allows easily creating a renderer from an anonymous function.
pub fn render_fn<F: Fn(&mut dyn RenderTarget) -> Result<(), io::Error>>(f: F) -> FnRenderer<F> {
    FnRenderer(f)
}

impl<F: Fn(&mut dyn RenderTarget) -> Result<(), io::Error>> WeftRenderable for FnRenderer<F> {
    fn render_to(&self, target: &mut impl RenderTarget) -> Result<(), io::Error> {
        (self.0)(target)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn assert_renderable(_: impl WeftRenderable) {}

    #[test]
    fn from_fn_should_be_renderable() {
        let child = render_fn(|_| Ok(()));
        assert_renderable(&child);
    }
}
