use std::io;

use weft::{AttrPair, QName, RenderTarget, WeftRenderable};

struct ErasedRenderTarget<'a>(&'a mut dyn RenderTarget);

/// This is exactly like the [`weft::WeftRenderable`] trait, but for cases where
/// we need a trait object. Eg: for a `Vec<Box<dyn ErasedRenderable>>`.
pub trait ErasedRenderable {
    /// Outputs a representation of this object to the target.
    fn erased_render_to(&self, target: &mut dyn RenderTarget) -> Result<(), io::Error>;
}

impl<T> ErasedRenderable for T
where
    T: WeftRenderable,
{
    fn erased_render_to(&self, target: &mut dyn RenderTarget) -> Result<(), io::Error> {
        self.render_to(&mut ErasedRenderTarget(target))
    }
}

impl WeftRenderable for dyn ErasedRenderable {
    fn render_to(&self, target: &mut impl RenderTarget) -> Result<(), io::Error> {
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
