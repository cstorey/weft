use std::io;
use template::{RenderTarget, WeftRenderable};

impl<'a> WeftRenderable for &'a str {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        target.text(self)
    }
}

impl<'a> WeftRenderable for String {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        target.text(self)
    }
}
