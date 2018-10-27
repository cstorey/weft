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
fn should_interpolate_content_in_cdata() {
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
