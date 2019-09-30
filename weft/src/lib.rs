#![deny(missing_docs)]
/*!
# `weft`.
This module provides runtime support for `weft` templates.

## Example:

```rust
   use weft_derive::WeftRenderable;
   #[derive(WeftRenderable)]
   #[template(source = "<p>Hello {{ self.0 }}!</p>")]
   struct Greeting(String);

    fn main() {
        let s = weft::render_to_string(Greeting("world".into())).expect("render_to_string");
        println!("{}", s);
        // Should print `<p>Hello world!<p>`
    }
```
*/

mod extensions;
mod template;

pub use crate::template::*;
pub use weft_derive::WeftRenderable;

/// A module for things that should be in-scope by default in a template expression.
pub mod prelude {
    pub use crate::extensions::*;
}
