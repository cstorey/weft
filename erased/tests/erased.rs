use weft::WeftRenderable;


#[derive(WeftRenderable)]
#[template(path = "../weft/tests/content.html")]
struct WithBoxedContent {
    child: Box<dyn weft_erased::ErasedRenderable>,
}

#[test]
fn should_support_boxed_content() {
    let view = WithBoxedContent {
        child: Box::new("Hello".to_string()),
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
#[template(path = "../weft/tests/content.html")]
struct WithPolyContent<C> {
    // Need a way to declare which type variables need to have `WeftRenderable`
    // constraints when we declare the struct WeftRenderable impl.
    child: C,
}

#[test]
fn should_support_fn_content() {
    let child = weft_erased::render_fn(|target| target.text("Hello from a function"));
    let view = WithPolyContent { child };

    let s = weft::render_to_string(view).expect("render_to_string");
    println!("{}", s);

    let expected = "Hello from a function";
    assert!(
        s.contains(expected),
        "String {:?} should contain {:?}",
        s,
        expected
    )
}
