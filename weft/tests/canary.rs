extern crate weft;

use std::io;
use weft::*;

#[test]
fn should_render_trivial_example() {
    struct TrivialExample;
    // This simulates what a template of the form `<p>Hello</p>` should compile to.
    impl Renderable for TrivialExample {
        fn render_to<T: RenderTarget>(&self, mut target: T) -> Result<(), io::Error> {
            target.start_element("p".into())?;
            target.text("Hello".into())?;
            target.end_element("p".into())?;
            Ok(())
        }
    }

    let s = render_to_string(TrivialExample).expect("render_writer");
    let expected = "<p>Hello</p>";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    );
}

#[test]
fn should_derive_trivial_from_markup() {
    #[derive(WeftTemplate)]
    struct TrivialMarkup;

    let s = render_to_string(TrivialMarkup).expect("render_writer");
    println!("{}", s);
}
