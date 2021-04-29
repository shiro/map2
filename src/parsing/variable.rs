use super::*;

pub(super) fn variable_initialization(input: &str) -> Res<&str, Expr> {
    context(
        "variable_declaration",
        tuple((
            tag("let"),
            multispace0,
            ident,
            multispace0,
            tag("="),
            multispace0,
            expr,
        )),
    )(input).map(|(next, parts)|
        (next, Expr::Init(parts.2, Box::new(parts.6)))
    )
}

pub(super) fn variable_assignment(input: &str) -> Res<&str, Expr> {
    context(
        "variable_assignment",
        tuple((
            ident,
            multispace0,
            tag("="),
            multispace0,
            expr,
        )),
    )(input).map(|(next, parts)|
        (next, Expr::Assign(parts.0, Box::new(parts.4)))
    )
}

pub(super) fn variable(input: &str) -> Res<&str, Expr> {
    context(
        "variable",
        ident,
    )(input).map(|(next, v)| (next, Expr::Name(v)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment() {
        assert_eq!(ident("hello2"), Ok(("", "hello2".to_string())));
        assert_eq!(variable_assignment("foo = true"),
                   Ok(("", Expr::Assign("foo".to_string(), Box::new(boolean("true").unwrap().1))))
        );

        assert!(matches!(ident("2hello"), Err(..)));
    }

    #[test]
    fn test_lambda() {
        assert_eq!(variable_initialization("let a = || {}"), Ok(("", Expr::Init(
            "a".to_string(),
            Box::new(expr("||{}").unwrap().1),
        )
        )));
    }
}
