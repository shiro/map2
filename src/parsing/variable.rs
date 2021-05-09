use super::*;
use nom::combinator::consumed;

pub(super) fn variable_initialization(input: &str) -> ResNew<&str, Expr> {
    let (input, _) = tag_custom("let")(input)
        .map_err(|_: NomErr<CustomError<_>>| make_generic_nom_err_options(input, vec!["variable initialization".to_string()]))?;

    tuple((
        remaining(tuple((
            ws1,
            ident,
            ws0,
            tag_custom("="),
        ))),
        ws0,
        expr,
    ))(input)
        .and_then(|(next, parts)| {
            let (input_before_expr, ident, expr) = (parts.0.0, parts.0.1.1, parts.2);
            let ((name, _), (expr, last_err)) = (ident, expr);

            match expr {
                Expr::Name(_) | Expr::Value(_) | Expr::Lambda(_, _) | Expr::FunctionCall(_, _) | Expr::Eq(_, _) | Expr::Neq(_, _) |
                Expr::LT(_, _) | Expr::GT(_, _) | Expr::Add(_, _) | Expr::Sub(_, _) | Expr::Div(_, _) |
                Expr::Mul(_, _) | Expr::Neg(_) | Expr::And(_, _) | Expr::Or(_, _)
                => {}
                _ => { return Err(make_generic_nom_err_options(input_before_expr, vec!["valid initialization expression".to_string()])); }
            };

            let expr = Expr::Init(name, Box::new(expr));
            Ok((next, (expr, last_err)))
        })
}

pub(super) fn variable_assignment(input: &str) -> ResNew<&str, Expr> {
    tuple((
        ident,
        ws0,
        tag_custom("="),
        ws0,
        expr,
    ))(input).map(|(next, parts)|
        (next, (Expr::Assign(parts.0.0, Box::new(parts.4.0)), None))
    )
}

pub(super) fn variable(input: &str) -> ResNew<&str, Expr> {
    ident(input)
        .map(|(next, (name, last_err))|
            (next, (Expr::Name(name), last_err)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment() {
        assert_eq!(ident("hello2"), nom_ok( "hello2".to_string()));
        assert_eq!(variable_assignment("foo = true"),
                   nom_ok( Expr::Assign("foo".to_string(), Box::new(nom_eval(boolean("true")))))
        );

        assert!(matches!(ident("2hello"), Err(..)));
    }

    #[test]
    fn test_lambda() {
        assert_eq!(variable_initialization("let a = || {}"), nom_ok( Expr::Init(
            "a".to_string(),
            Box::new(nom_eval(expr("||{}"))),
        )
        ));
    }
}
