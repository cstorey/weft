#![deny(missing_docs)]

//! This module provides dynamic polymorphism support for `weft` templates.

mod template;

pub use template::{ErasedRenderable,render_fn};