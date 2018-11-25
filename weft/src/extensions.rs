use std::{fmt, io};
use template::{RenderTarget, WeftRenderable};

/// A helper struct for the `Displayable` trait.
pub struct Displayer<'a, D: 'a>(&'a D);

/// A mechanism to render a value that implements `fmt::Display` in templates.
pub trait Displayable: Sized {
    /// Extension method for types that implement fmt::Display
    fn display(&self) -> Displayer<Self>;
}

impl<D: fmt::Display> Displayable for D {
    fn display(&self) -> Displayer<D> {
        Displayer(self)
    }
}

impl<'a> WeftRenderable for &'a str {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        target.text(self)
    }
}

impl WeftRenderable for String {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        target.text(self)
    }
}

impl<'a, D: fmt::Display> WeftRenderable for Displayer<'a, D> {
    fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
        target.text(&format!("{}", self.0))
    }
}
