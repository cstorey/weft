extern crate regex;
extern crate weft;
extern crate weft_derive;

use regex::*;
use std::io;
use std::iter;
use weft::*;

#[test]
fn should_render_trivial_example() {
    struct TrivialExample;
    // This simulates what a template of the form `<p>Hello</p>` should compile to.
    impl Renderable for TrivialExample {
        fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
            target.start_element("p".into())?;
            target.text("Hello".into())?;
            target.end_element("p".into())?;
            Ok(())
        }
    }

    let s = render_to_string(TrivialExample).expect("render_to_string");
    let expected = "<p>Hello</p>";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    );
}

#[test]
fn should_render_attrs() {
    struct TrivialExample;
    // This simulates what a template of the form `<p>Hello</p>` should compile to.
    impl Renderable for TrivialExample {
        fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
            target.start_element_attrs(
                "p".into(),
                iter::once(&AttrPair::new("class", "some-classes")),
            )?;
            target.text("Hello".into())?;
            target.end_element("p".into())?;
            Ok(())
        }
    }

    let s = render_to_string(TrivialExample).expect("render_to_string");

    let matcher = Regex::new("\\s+class=[\"']some-classes[\"']").expect("Regex::new");
    assert!(
        matcher.find(&s).is_some(),
        "String {:?} matches {:?}",
        s,
        matcher
    )
}

#[test]
fn render_supports_builtins() {
    struct TrivialExample;
    // This simulates what a template of the form `<p>Hello</p>` should compile to.
    impl Renderable for TrivialExample {
        fn render_to<T: RenderTarget>(&self, target: &mut T) -> Result<(), io::Error> {
            "Hello world!".render_to(target)?;
            Ok(())
        }
    }

    let s = render_to_string(TrivialExample).expect("render_to_string");
    let expected = "Hello world!";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    );
}
