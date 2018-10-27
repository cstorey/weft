extern crate regex;
extern crate weft;
#[macro_use]
extern crate weft_derive;

use regex::*;
use std::io;
use weft::*;

#[derive(WeftTemplate)]
#[template(path = "tests/trivial.html")]
struct TrivialMarkup;

#[test]
fn should_derive_trivial_from_markup() {
    let s = render_to_string(TrivialMarkup).expect("render_to_string");
    println!("{}", s);

    let expected = "<div>Trivial</div>";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    )
}

#[test]
fn should_not_include_enclosing_html_tags() {
    let s = render_to_string(TrivialMarkup).expect("render_to_string");
    println!("{}", s);

    let unexpected = "<html>";
    assert!(
        !s.contains(unexpected),
        "String {:?} contains {:?}",
        s,
        unexpected
    )
}
#[derive(WeftTemplate)]
#[template(path = "tests/trivial_with_attrs.html")]
struct TrivialAttrs;

#[test]
fn should_pass_through_attributes() {
    let s = render_to_string(TrivialAttrs).expect("render_to_string");
    println!("{}", s);

    let matcher = Regex::new("class=[\"']foo[\"']").expect("Regex::new");
    assert!(
        matcher.find(&s).is_some(),
        "String {:?} matches {:?}",
        s,
        matcher
    )
}

#[derive(WeftTemplate)]
#[template(path = "tests/interpolatable.html")]
struct Interpolatable<'a> {
    name: &'a str,
}

#[test]
fn should_support_replace_directive() {
    let view = Interpolatable { name: "Bob" };

    let s = render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = "My name is Bob";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    )
}
#[derive(WeftTemplate)]
#[template(path = "tests/content.html")]
struct WithContent {
    child: String,
}

#[test]
fn should_support_content() {
    let view = WithContent {
        child: "Hello".into(),
    };

    let s = render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = "<p>Hello</p>";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    )
}
#[derive(WeftTemplate)]
#[template(path = "tests/conditional.html")]
struct Conditional {
    enabled: bool,
}
#[test]
fn should_strip_when_conditional_false() {
    let view = Conditional { enabled: false };

    let s = render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let unexpected = "I am enabled";
    assert!(
        !s.contains(unexpected),
        "String {:?} should not contain {:?}",
        s,
        unexpected
    )
}

#[test]
fn should_include_when_conditional_true() {
    let view = Conditional { enabled: true };

    let s = render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = "I am enabled";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}

#[derive(WeftTemplate)]
#[template(path = "tests/for-in.html")]
struct ForIn<'a> {
    items: Vec<&'a str>,
}
#[test]
fn should_support_iteration() {
    let view = ForIn {
        items: vec!["one", "two", "three"],
    };

    let s = render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = ["<p>one</p>", "<p>two</p>", "<p>three</p>"];
    assert!(
        expected.iter().all(|it| s.contains(it)),
        "String {:?} should contain all of {:?}",
        s,
        expected
    )
}

#[cfg(never)]
mod todo {
    #[derive(WeftTemplate)]
    #[template(path = "tests/content.html")]
    struct WithPolyContent<C> {
        // Need a way to declare which type variables need to have `Renderable`
        // constraints when we declare the struct Renderable impl.
        child: C,
    }
}
