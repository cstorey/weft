extern crate regex;
extern crate weft;
#[macro_use]
extern crate weft_derive;

use regex::*;

#[derive(WeftRenderable)]
#[template(path = "tests/trivial.html")]
struct TrivialMarkup;

#[test]
fn should_derive_trivial_from_markup() {
    let s = weft::render_to_string(TrivialMarkup).expect("render_to_string");
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
fn should_allow_inline_source() {
    #[derive(WeftRenderable)]
    #[template(source = "I am inline")]
    struct InlineMarkup;

    let s = weft::render_to_string(InlineMarkup).expect("render_to_string");
    println!("{}", s);

    let expected = "I am inline";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}

#[test]
fn should_not_include_enclosing_html_tags() {
    let s = weft::render_to_string(TrivialMarkup).expect("render_to_string");
    println!("{}", s);

    let unexpected = "<html>";
    assert!(
        !s.contains(unexpected),
        "String {:?} contains {:?}",
        s,
        unexpected
    )
}
#[derive(WeftRenderable)]
#[template(path = "tests/trivial_with_attrs.html")]
struct TrivialAttrs;

#[test]
fn should_pass_through_attributes() {
    let s = weft::render_to_string(TrivialAttrs).expect("render_to_string");
    println!("{}", s);

    let matcher = Regex::new("class=[\"']foo[\"']").expect("Regex::new");
    assert!(
        matcher.find(&s).is_some(),
        "String {:?} matches {:?}",
        s,
        matcher
    )
}

#[derive(WeftRenderable)]
#[template(path = "tests/interpolatable.html")]
struct Interpolatable<'a> {
    name: &'a str,
}

#[test]
fn should_support_replace_directive() {
    let view = Interpolatable { name: "Bob" };

    let s = weft::render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = "My name is Bob";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    )
}
#[derive(WeftRenderable)]
#[template(path = "tests/content.html")]
struct WithContent {
    child: String,
}

#[test]
fn should_support_content() {
    let view = WithContent {
        child: "Hello".into(),
    };

    let s = weft::render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = "<p>Hello</p>";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    )
}
#[derive(WeftRenderable)]
#[template(path = "tests/conditional.html")]
struct Conditional {
    enabled: bool,
}
#[test]
fn should_strip_when_conditional_false() {
    let view = Conditional { enabled: false };

    let s = weft::render_to_string(view).expect("render_to_string");
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

    let s = weft::render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = "I am enabled";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}

#[derive(WeftRenderable)]
#[template(path = "tests/for-in.html")]
struct ForIn<'a> {
    items: Vec<&'a str>,
}
#[test]
fn should_support_iteration() {
    let view = ForIn {
        items: vec!["one", "two", "three"],
    };

    let s = weft::render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = ["<p>one</p>", "<p>two</p>", "<p>three</p>"];
    assert!(
        expected.iter().all(|it| s.contains(it)),
        "String {:?} should contain all of {:?}",
        s,
        expected
    )
}

#[derive(WeftRenderable)]
#[template(path = "tests/content.html",)]
struct WithPolyContent<C> {
    // Need a way to declare which type variables need to have `WeftRenderable`
    // constraints when we declare the struct WeftRenderable impl.
    child: C,
}

#[test]
fn should_support_polymorphism() {
    let view = WithPolyContent { child: "hello" };

    let s = weft::render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = "<p>hello</p>";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}

#[test]
fn should_support_bare_attributes() {
    // We ostensibly don't differentiate between a bare attribute and
    // and empty value.
    #[derive(WeftRenderable)]
    #[template(source = "<p some-thing></p>")]
    struct BareAttr;

    let s = weft::render_to_string(BareAttr).expect("render_to_string");
    println!("{}", s);

    let expected = " some-thing=";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}

#[test]
fn should_support_inline_expr_in_cdata() {
    #[derive(WeftRenderable)]
    #[template(source = "<p>Hello {{ self.0 }}!</p>")]
    struct Greeting(String);

    let s = weft::render_to_string(Greeting("world".into())).expect("render_to_string");
    println!("{}", s);

    let expected = "<p>Hello world!</p>";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}

#[test]
fn should_support_inline_expr_in_attrs() {
    #[derive(WeftRenderable)]
    #[template(source = "<span abbr=\"{{self.0}}\">Longer thing</span>")]
    struct Abbr(String);

    let s = weft::render_to_string(Abbr("Long".into())).expect("render_to_string");
    println!("{}", s);

    let expected = "<span abbr=\"Long\">";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}

#[test]
fn should_correctly_parse_inline_with_entities() {
    #[derive(WeftRenderable)]
    #[template(source = "<p>Hello &#123;&#123; self.0 &#125;&#125;!</p>")]
    struct Greeting(String);

    let s = weft::render_to_string(Greeting("world".into())).expect("render_to_string");
    println!("{}", s);

    let expected = "<p>Hello world!</p>";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}

#[cfg(never)]
#[test]
#[ignore]
fn should_allow_disabling_inline_exprs() {
    #[derive(WeftRenderable)]
    #[template(source = "<p weft-inline-disable>Hello {{ self.0 }}!</p>")]
    struct Greeting2(String);

    let s = weft::render_to_string(Greeting2("world".into())).expect("render_to_string");
    println!("{}", s);

    let expected = "<p>Hello {{ self.0 }}!</p>";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}

#[test]
fn should_import_displayable() {
    #[derive(WeftRenderable)]
    #[template(source = "<p>{{ self.0.display() }}</p>")]
    struct Displayer(u64);

    let s = weft::render_to_string(Displayer(42)).expect("render_to_string");
    println!("{}", s);

    let expected = ">42<";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}