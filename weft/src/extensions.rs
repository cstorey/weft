use std::io;
use template::{RenderTarget, WeftTemplate};

impl<'a> WeftTemplate for &'a str {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        target.text(self)
    }
}

impl<'a> WeftTemplate for String {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        target.text(self)
    }
}
