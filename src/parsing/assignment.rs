use super::*;

pub(super) fn variable_assignment(input: &str) -> Res<&str, Expr> {
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
        (next, Expr::Assign(parts.2, Box::new(parts.6)))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment() {
        assert_eq!(ident("hello2"), Ok(("", "hello2".to_string())));
        assert_eq!(variable_assignment("let foo = true"),
                   Ok(("", Expr::Assign("foo".to_string(), Box::new(boolean("true").unwrap().1))))
        );

        assert!(matches!(ident("2hello"), Err(..)));
    }

    #[test]
    fn test_lambda() {
        assert_eq!(variable_assignment("let a = || {}"), Ok(("", Expr::Assign(
            "a".to_string(),
            Box::new(expr("||{}").unwrap().1),
        )
        )));
    }
}
