use regex::*;
use std::fmt;
use weft::*;

#[test]
fn should_render_trivial_example() {
    struct TrivialExample;
    // This simulates what a template of the form `<p>Hello</p>` should compile to.
    impl WeftRenderable for TrivialExample {
        fn render_to<T>(&self, target: &mut T) -> IoResult<()>
        where
            for<'t> &'t mut T: RenderTarget,
        {
            {
                let e = RenderTarget::start_element(&mut *target, "p".into())?;
                // let e = target.start_element("p".into())?;
                e.close()?;
            }
            target.text("Hello")?;
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
    impl WeftRenderable for TrivialExample {
        fn render_to<T>(&self, target: &mut T) -> IoResult<()>
        where
            for<'t> &'t mut T: RenderTarget,
        {
            let mut e = target.start_element("p".into())?;
            e.attribute(&AttrPair::new("class", "some-classes"))?;
            e.close()?;
            target.text("Hello")?;
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
    impl WeftRenderable for TrivialExample {
        fn render_to<T>(&self, target: &mut T) -> IoResult<()>
        where
            for<'t> &'t mut T: RenderTarget,
        {
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

#[test]
fn display_supports_renderable() {
    struct Displayable<D>(D);
    // This simulates what a template of the form `<p>Hello</p>` should compile to.
    impl<D: fmt::Display> WeftRenderable for Displayable<D> {
        fn render_to<T>(&self, target: &mut T) -> IoResult<()>
        where
            for<'t> &'t mut T: RenderTarget,
        {
            use weft::prelude::*;
            self.0.display().render_to(target)?;
            Ok(())
        }
    }

    let s = render_to_string(Displayable(42)).expect("render_to_string");
    let expected = "42";
    assert_eq!(s, expected);
}

#[test]
fn display_supports_to_string() {
    struct Displayable<D>(D);
    // This simulates what a template of the form `<p>Hello</p>` should compile to.
    impl<D: fmt::Display> WeftRenderable for Displayable<D> {
        fn render_to<T>(&self, target: &mut T) -> IoResult<()>
        where
            for<'t> &'t mut T: RenderTarget,
        {
            use weft::prelude::*;
            let mut e = target.start_element("p".into())?;
            e.attribute(&weft::AttrPair::new("x", self.0.display().to_string()))?;
            e.close()?;
            Ok(())
        }
    }

    let s = render_to_string(Displayable(23)).expect("render_to_string");
    let expected = "=\"23\"";
    assert!(
        s.contains(expected),
        "String {:?} contains {:?}",
        s,
        expected
    );
}
