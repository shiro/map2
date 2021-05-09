use super::*;
use nom::number::complete::double;

pub(super) fn string(input: &str) -> ResNew<&str, Expr> {
    tuple((tag_custom("\""), take_until("\""), tag_custom("\"")))(input)
        .map_err(|_: NomErr<CustomError<_>>| make_generic_nom_err_options(input, vec!["string".to_string()]))
        .map(|(next, v)|
            (next, (Expr::Value(ValueType::String(v.1.to_string())), None))
        )
}

pub(super) fn boolean(input: &str) -> ResNew<&str, Expr> {
    alt((tag_custom("true"), tag_custom("false")))
        (input)
        .map_err(|_: NomErr<CustomError<_>>| make_generic_nom_err_options(input, vec!["boolean".to_string()]))
        .map(|(next, v)|
            (next, match v {
                "true" => (Expr::Value(ValueType::Bool(true)), None),
                "false" => (Expr::Value(ValueType::Bool(false)), None),
                _ => unreachable!(),
            })
        )
}

pub(super) fn number(input: &str) -> ResNew<&str, Expr> {
    double
        (input)
        .map_err(|_: NomErr<CustomError<_>>| make_generic_nom_err_options(input, vec!["number".to_string()]))
        .map(|(next, v)|
            {
                let expr = Expr::Value(ValueType::Number(v));
                (next, (expr, None))
            }
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitives() {
        assert_eq!(boolean("true"), nom_ok(Expr::Value(ValueType::Bool(true))));
        assert_eq!(boolean("false"), nom_ok(Expr::Value(ValueType::Bool(false))));
        assert!(matches!(boolean("foo"), Err(..)));

        assert_eq!(string("\"hello world\""), nom_ok(Expr::Value(ValueType::String("hello world".to_string()))));
    }

    #[test]
    fn test_number() {
        assert_eq!(number("42"), nom_ok(Expr::Value(ValueType::Number(42.0))));
        assert_eq!(number("-42.5"), nom_ok( Expr::Value(ValueType::Number(-42.5))));
    }
}
