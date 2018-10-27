extern crate weft;
#[macro_use]
extern crate weft_derive;

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
