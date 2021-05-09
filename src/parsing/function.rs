use super::*;

pub(super) fn function_arg(input: &str) -> ResNew<&str, Expr> {
    expr(input)
}

pub(super) fn function_call(input: &str) -> ResNew<&str, Expr> {
    let (input, (ident_res,_)) = tuple((ident, tag_custom("(")))(input)
        .map_err(|_: NomErr<CustomError<_>>| make_generic_nom_err_options(input, vec!["function call".to_string()]))?;

    tuple((
        ws0,
        opt(tuple((
            function_arg,
            ws0,
            many0(tuple((
                tag_custom(","),
                ws0,
                function_arg,
                ws0,
            ))),
        ))),
        tag_custom(")"),
    ))(input).map(|(next, parts)| {
        let expr = match parts.1 {
            Some(arg_v) => {
                let mut args: Vec<Expr> = arg_v.2.into_iter().map(|x| x.2.0).collect();
                args.insert(0, arg_v.0.0);
                 Expr::FunctionCall(ident_res.0, args)
            }
            _ => Expr::FunctionCall(ident_res.0, vec![])
        };
        (next, (expr,None))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_call() {
        assert_eq!(function_call("foobar()"), nom_ok( Expr::FunctionCall("foobar".to_string(), vec![])));
        assert_eq!(function_call("foobar(\"hello\", true)"), nom_ok( Expr::FunctionCall("foobar".to_string(), vec![
            Expr::Value(ValueType::String("hello".to_string())),
            Expr::Value(ValueType::Bool(true)),
        ])));
        assert_eq!(function_call("foobar(true == true)"), nom_ok( Expr::FunctionCall("foobar".to_string(), vec![
            nom_eval(expr("true == true"))
        ])));

        assert_eq!(function_call("print(variable)"), nom_ok( Expr::FunctionCall("print".to_string(), vec![
            nom_eval(variable("variable"))
        ])));
    }
}
