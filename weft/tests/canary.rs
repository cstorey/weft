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

    let s = render_to_string(TrivialExample).expect("render_writer");
    let expected = "<p>Hello</p>";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    );
}
