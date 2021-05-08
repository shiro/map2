use super::*;

pub(super) fn expr_4(input: &str) -> ResNew<&str, Expr> {
    alt((
        map(tuple((tag_custom("("), expr, tag_custom(")"))), |(_, v, _)| v),
        boolean,
        string,
        number,
        lambda,
        variable_initialization,
        variable_assignment,
        function_call,
        key_mapping_inline,
        key_mapping,
        variable,
    ))(input)
}

// pub(super) fn expr_3(input: &str) -> Res<&str, Expr> {
//     context("expr_3",
//             alt((
//                 map(tuple((tag("??"), expr_4)), |(_, v)| Expr::Neg(Box::new(v))),
//                 expr_4,
//             )))(input)
// }
//
// pub(super) fn expr_2(i: &str) -> Res<&str, Expr> {
//     let (i, init) = expr_3(i)?;
//     fold_many0_once(
//         |i: &str| {
//             context(
//                 "expr_2",
//                 tuple((ws0, alt((tag("*"), tag("/"))), ws0, expr_3)),
//             )(i)
//         },
//         init,
//         |acc, (_, op, _, val)| {
//             match op {
//                 "*" => Expr::Mul(Box::new(acc), Box::new(val)),
//                 "/" => Expr::Div(Box::new(acc), Box::new(val)),
//                 _ => unreachable!()
//             }
//         },
//     )(i)
// }
//
// pub(super) fn expr_1(i: &str) -> Res<&str, Expr> {
//     let (i, init) = expr_2(i)?;
//     fold_many0_once(
//         |i: &str| {
//             context(
//                 "expr_1",
//                 tuple((ws0, alt((tag("+"), tag("-"))), ws0, expr_2)),
//             )(i)
//         },
//         init,
//         |acc, (_, op, _, val)| {
//             match op {
//                 "+" => Expr::Add(Box::new(acc), Box::new(val)),
//                 "-" => Expr::Sub(Box::new(acc), Box::new(val)),
//                 _ => unreachable!()
//             }
//         },
//     )(i)
// }

pub(super) fn expr(input: &str) -> ResNew<&str, Expr> {
    let (input, init) = expr_4(input)?;
        // .map_err(|v| NomErr::Error(CustomError { input, expected: vec!["expression".to_string()] }))?;

    let expr = fold_many0_once_err(
        |i: &str| {
            tuple((alt((tag("=="), tag("!="), tag("&&"), tag("||"), tag("<"), tag(">"))), ws0, expr_4))(i)
        },
        init.0,
        |acc, (op, _, (val, last_err))| {
            match op {
                "==" => Expr::Eq(Box::new(acc), Box::new(val)),
                "!=" => Expr::Neq(Box::new(acc), Box::new(val)),
                "&&" => Expr::And(Box::new(acc), Box::new(val)),
                "||" => Expr::Or(Box::new(acc), Box::new(val)),
                ">" => Expr::GT(Box::new(acc), Box::new(val)),
                "<" => Expr::LT(Box::new(acc), Box::new(val)),
                _ => unreachable!()
            }
        },
    )(input);

    match expr {
        Err(v) => Err(v),
        Ok((next, (expr, last_err))) => Ok((next, (expr, Some(last_err)))),
    }
    // .map(|(next, expr)| (next, (expr, None)))
    // .map_err(|v| NomErr::Error(CustomError { input, expected: vec!["expression".to_string()] }))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operator_equal() {
        assert_eq!(expr("true == true"), Ok(("", Expr::Eq(
            Box::new(Expr::Value(ValueType::Bool(true))),
            Box::new(Expr::Value(ValueType::Bool(true))),
        ))));
        assert_eq!(expr("\"hello world\" == \"hello world\""), Ok(("", Expr::Eq(
            Box::new(Expr::Value(ValueType::String("hello world".to_string()))),
            Box::new(Expr::Value(ValueType::String("hello world".to_string()))),
        ))));
        assert_eq!(expr("\"22hello\" == true"), Ok(("", Expr::Eq(
            Box::new(Expr::Value(ValueType::String("22hello".to_string()))),
            Box::new(Expr::Value(ValueType::Bool(true))),
        ))));
    }

    #[test]
    fn test_add_sub() {
        assert_eq!(expr("33 + 33"), Ok(("", Expr::Add(
            Box::new(Expr::Value(ValueType::Number(33.0))),
            Box::new(Expr::Value(ValueType::Number(33.0))),
        ))));

        assert_eq!(expr("33 - 33"), Ok(("", Expr::Sub(
            Box::new(Expr::Value(ValueType::Number(33.0))),
            Box::new(Expr::Value(ValueType::Number(33.0))),
        ))));
    }
}
