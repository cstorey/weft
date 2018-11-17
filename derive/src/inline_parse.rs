use failure::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Literal(String),
    Expr(String),
}

pub fn parse_inline(input: &str) -> Result<Vec<Segment>, Error> {
    let re = regex::Regex::new(r"\{\{([^}]|}[^}])*\}\}")?;
    trace!("Scanning: {:?}", input);
    let mut last_match = 0;
    let mut out = Vec::new();
    for it in re.find_iter(input) {
        trace!("Got: {:?}", it);

        let previous = &input[last_match..it.start()];
        if previous.len() > 0 {
            out.push(Segment::Literal(previous.into()));
        }

        let m = it.as_str();
        let range = 2..(m.len() - 2);
        out.push(Segment::Expr(m[range].trim().into()));

        last_match = it.end();
    }
    let remainder = &input[last_match..];
    if remainder.len() > 0 {
        out.push(Segment::Literal(remainder.into()));
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trivial_expr() {
        let segments = parse_inline("{{ foo }}").expect("parse_inline");
        assert_eq!(&segments, &[Segment::Expr("foo".into())])
    }

    #[test]
    fn test_trivial_literal() {
        let segments = parse_inline("Hi!").expect("parse_inline");
        let expected = vec![Segment::Literal("Hi!".into())];
        println!("Expected: {:?}", expected);
        println!("Got: {:?}", segments);
        assert_eq!(&segments, &expected)
    }

    #[test]
    fn test_mixed_1() {
        let segments = parse_inline("A {{ foo }}").expect("parse_inline");
        assert_eq!(
            &segments,
            &[Segment::Literal("A ".into()), Segment::Expr("foo".into())]
        )
    }

    #[test]
    fn test_mixed_2() {
        let segments = parse_inline("I {{ verb }} with {{ noun }}.").expect("parse_inline");
        assert_eq!(
            &segments,
            &[
                Segment::Literal("I ".into()),
                Segment::Expr("verb".into()),
                Segment::Literal(" with ".into()),
                Segment::Expr("noun".into()),
                Segment::Literal(".".into())
            ]
        )
    }

}
