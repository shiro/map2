use super::*;

pub(super) fn string(input: &str) -> Res<&str, Expr> {
    context(
        "string",
        tuple((tag("\""), take_until("\""), tag("\""))),
    )(input)
        .map(|(next, v)| (next, Expr::String(v.1.to_string())))
}

pub(super) fn boolean(input: &str) -> Res<&str, Expr> {
    context(
        "value",
        alt((tag("true"), tag("false"))),
    )(input).map(|(next, v)|
        (next, match v {
            "true" => Expr::Boolean(true),
            "false" => Expr::Boolean(false),
            _ => unreachable!(),
        })
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        assert!(matches!(boolean("true"), Ok(("", Expr::Boolean(true)))));
        assert!(matches!(boolean("false"), Ok(("", Expr::Boolean(false)))));
        assert!(matches!(boolean("foo"), Err(..)));

        assert_eq!(string("\"hello world\""), Ok(("", Expr::String("hello world".to_string()))));
    }
}
