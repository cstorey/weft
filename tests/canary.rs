extern crate weft;

use std::io;
use weft::*;

#[test]
fn should_render_trivial_example() {
    struct TrivialExample;
    impl Renderable for TrivialExample {
        fn render_to<T: RenderTarget>(&self, mut target: T) -> Result<(), io::Error> {
            target.start_element("p".into())?;
            target.text("Hello".into())?;
            target.end_element("p".into())?;
            Ok(())
        }
    }

    let mut out = Vec::new();
    render_writer(TrivialExample, &mut out).expect("render_writer");
    let s = String::from_utf8_lossy(&out);
    let expected = "<p>Hello</p>";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    );
}
