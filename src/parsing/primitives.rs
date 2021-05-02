use super::*;
use nom::number::complete::double;

pub(super) fn string(input: &str) -> Res<&str, Expr> {
    context(
        "string",
        tuple((tag("\""), take_until("\""), tag("\""))),
    )(input)
        .map(|(next, v)| (next, Expr::Value(ValueType::String(v.1.to_string()))))
}

pub(super) fn boolean(input: &str) -> Res<&str, Expr> {
    context(
        "boolean",
        alt((tag("true"), tag("false"))),
    )(input).map(|(next, v)|
        (next, match v {
            "true" => Expr::Value(ValueType::Bool(true)),
            "false" => Expr::Value(ValueType::Bool(false)),
            _ => unreachable!(),
        })
    )
}

pub(super) fn number(input: &str) -> Res<&str, Expr> {
    context(
        "boolean",
        double,
    )(input).map(|(next, v)|
        (next, Expr::Value(ValueType::Number(v)))
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        assert!(matches!(boolean("true"), Ok(("", Expr::Value(ValueType::Bool(true))))));
        assert!(matches!(boolean("false"), Ok(("", Expr::Value(ValueType::Bool(false))))));
        assert!(matches!(boolean("foo"), Err(..)));

        assert_eq!(string("\"hello world\""), Ok(("", Expr::Value(ValueType::String("hello world".to_string())))));
    }

    #[test]
    fn test_number(){
        assert!(matches!(number("42"), Ok(("", Expr::Value(ValueType::Number(42.0))))));
        assert!(matches!(number("-42.5"), Ok(("", Expr::Value(ValueType::Number(-42.5))))));
    }
}
