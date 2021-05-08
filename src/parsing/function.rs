use super::*;

// pub(super) fn function_arg(input: &str) -> Res<&str, Expr> {
//     context("function_arg", expr)(input)
// }

// pub(super) fn function_call(input: &str) -> Res<&str, Expr> {
//     context(
//         "function_call",
//         tuple((
//             ident,
//             tag("("),
//             ws0,
//             opt(tuple((
//                 function_arg,
//                 ws0,
//                 many0(tuple((
//                     tag(","),
//                     ws0,
//                     function_arg,
//                     ws0,
//                 ))),
//             ))),
//             tag(")"),
//         )),
//     )(input).map(|(next, v)| {
//         match v.3 {
//             Some(arg_v) => {
//                 let mut args: Vec<Expr> = arg_v.2.into_iter().map(|x| x.2).collect();
//                 args.insert(0, arg_v.0);
//                 (next, Expr::FunctionCall(v.0, args))
//             }
//             _ => (next, Expr::FunctionCall(v.0, vec![]))
//         }
//     })
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_call() {
        assert_eq!(function_call("foobar()"), Ok(("", Expr::FunctionCall("foobar".to_string(), vec![]))));
        assert_eq!(function_call("foobar(\"hello\", true)"), Ok(("", Expr::FunctionCall("foobar".to_string(), vec![
            Expr::Value(ValueType::String("hello".to_string())),
            Expr::Value(ValueType::Bool(true)),
        ]))));
        assert_eq!(function_call("foobar(true == true)"), Ok(("", Expr::FunctionCall("foobar".to_string(), vec![
            expr("true == true").unwrap().1
        ]))));

        assert_eq!(function_call("print(variable)"), Ok(("", Expr::FunctionCall("print".to_string(), vec![
            variable("variable").unwrap().1
        ]))));
    }
}
