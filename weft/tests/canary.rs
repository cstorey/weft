extern crate weft;

use std::io;
use weft::*;
#[macro_use]
extern crate weft_derive;

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

#[derive(WeftTemplate)]
#[template(path = "tests/trivial.html")]
struct TrivialMarkup;

#[test]
fn should_derive_trivial_from_markup() {
    let s = render_to_string(TrivialMarkup).expect("render_writer");
    println!("{}", s);

    let expected = "<div>Trivial example</div>";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    )
}

#[test]
#[ignore]
fn should_not_include_enclosing_html_tags() {
    let s = render_to_string(TrivialMarkup).expect("render_writer");
    println!("{}", s);

    let unexpected = "<html>";
    assert!(
        !s.contains(unexpected),
        "String {:?} contains {:?}",
        s,
        unexpected
    )
}