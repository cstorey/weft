use failure::Error;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Segment {
    Literal(String),
    Expr(syn::Expr),
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Substitutable {
    children: Vec<Segment>,
}

impl Substitutable {
    pub fn children<'a>(&'a self) -> impl 'a + Iterator<Item = Segment> {
        self.children.iter().cloned()
    }
}

pub fn parse_inline(input: &str) -> Result<Substitutable, Error> {
    let re = regex::Regex::new(r"\{\{([^}]|}[^}])*\}\}")?;
    trace!("Scanning: {:?}", input);
    let mut last_match = 0;
    let mut children = Vec::new();
    for it in re.find_iter(input) {
        trace!("Got: {:?}", it);

        let previous = &input[last_match..it.start()];
        if previous.len() > 0 {
            children.push(Segment::Literal(previous.into()));
        }

        let m = it.as_str();
        let range = 2..(m.len() - 2);
        let expr: syn::Expr =
            syn::parse_str(&m[range]).map_err(|e| failure::err_msg(format!("{:?}", e)))?;

        children.push(Segment::Expr(expr));

        last_match = it.end();
    }
    let remainder = &input[last_match..];
    if remainder.len() > 0 {
        children.push(Segment::Literal(remainder.into()));
    }

    Ok(Substitutable { children })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_trivial_expr() {
        let segments = parse_inline("{{ foo }}").expect("parse_inline");
        assert_eq!(&segments.children, &[Segment::Expr(parse_quote!(foo))])
    }

    #[test]
    fn test_trivial_literal() {
        let segments = parse_inline("Hi!").expect("parse_inline");
        let expected = vec![Segment::Literal("Hi!".into())];
        println!("Expected: {:?}", expected);
        println!("Got: {:?}", segments);
        assert_eq!(&segments.children, &expected)
    }

    #[test]
    fn test_mixed_1() {
        let segments = parse_inline("A {{ foo }}").expect("parse_inline");
        assert_eq!(
            &segments.children,
            &[
                Segment::Literal("A ".into()),
                Segment::Expr(parse_quote!(foo))
            ]
        )
    }

    #[test]
    fn test_mixed_2() {
        let segments = parse_inline("I {{ verb }} with {{ noun }}.").expect("parse_inline");
        assert_eq!(
            &segments.children,
            &[
                Segment::Literal("I ".into()),
                Segment::Expr(parse_quote!(verb)),
                Segment::Literal(" with ".into()),
                Segment::Expr(parse_quote!(noun)),
                Segment::Literal(".".into())
            ]
        )
    }
}
