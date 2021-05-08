use super::*;
use nom::combinator::not;

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

pub(super) fn expr_3(input: &str) -> ResNew<&str, Expr> {
    // TODO fold this
    let (input, expr) = alt((
        map(
            tuple((tag_custom("!"), not(tag("{")), expr_3)),
            |(_, _, (expr, last_err))| (Expr::Neg(Box::new(expr)), last_err),
        ),
        expr_4,
    ))(input)?;

    Ok((input, expr))
}

pub(super) fn expr_2(i: &str) -> ResNew<&str, Expr> {
    let (input, init) = expr_3(i)?;
    let expr = fold_many0_once_err(
        |input: &str| {
            tuple((
                ws0,
                alt((tag_custom("*"), tag_custom("/"))),
                ws0,
                expr_3
            ))(input)
        },
        init.0,
        |acc, (_, op, _, (val, _))| {
            match op {
                "*" => Expr::Mul(Box::new(acc), Box::new(val)),
                "/" => Expr::Div(Box::new(acc), Box::new(val)),
                _ => unreachable!()
            }
        },
    )(input);

    match expr {
        Err(v) => Err(v),
        Ok((next, (expr, last_err))) => Ok((next, (expr, Some(last_err)))),
    }
}

pub(super) fn expr_1(i: &str) -> ResNew<&str, Expr> {
    let (input, init) = expr_2(i)?;
    let expr = fold_many0_once_err(
        |input: &str| {
            tuple((
                ws0,
                alt((tag_custom("+"), tag_custom("-"))),
                ws0,
                expr_2,
            ))(input)
        },
        init.0,
        |acc, (_, op, _, (val, _))| {
            match op {
                "+" => Expr::Add(Box::new(acc), Box::new(val)),
                "-" => Expr::Sub(Box::new(acc), Box::new(val)),
                _ => unreachable!()
            }
        },
    )(input);

    match expr {
        Err(v) => Err(v),
        Ok((next, (expr, last_err))) => Ok((next, (expr, Some(last_err)))),
    }
}

pub(super) fn expr(input: &str) -> ResNew<&str, Expr> {
    let (input, init) = expr_1(input)?;

    let expr = fold_many0_once_err(
        |i: &str| {
            tuple((
                ws0,
                alt((
                    tag_custom("=="),
                    tag_custom("!="),
                    tag_custom("&&"),
                    tag_custom("||"),
                    tag_custom("<"),
                    tag_custom(">"))),
                ws0, expr_1))(i)
        },
        init.0,
        |acc, (_, op, _, (val, last_err))| {
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
