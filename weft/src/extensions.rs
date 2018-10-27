use std::io;
use template::{RenderTarget, Renderable};

impl<'a> Renderable for &'a str {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        target.text(self)
    }
}

impl<'a> Renderable for String {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        target.text(self)
    }
}
