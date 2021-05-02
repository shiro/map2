use super::*;

pub(crate) fn if_stmt(input: &str) -> Res<&str, Stmt> {
    context(
        "if_stmt",
        tuple((
            tag("if"),
            multispace0,
            tag("("),
            multispace0,
            expr,
            multispace0,
            tag(")"),
            multispace0,
            block,
            many0(tuple((
                tag("else"),
                multispace0,
                tag("if"),
                multispace0,
                tag("("),
                multispace0,
                expr,
                multispace0,
                tag(")"),
                multispace0,
                block,
            ))),
            opt(tuple((
                multispace0,
                tag("else"),
                multispace0,
                block,
            ))),
        )),
    )(input).map(|(next, v)| {
        let mut pairs: Vec<(Expr, Block)> = v.9.into_iter().map(|v| (v.6, v.10)).collect();
        let first_pair = (v.4, v.8);
        pairs.insert(0, first_pair);

        let else_block = match v.10 {
            Some(v) => Some(v.3),
            _ => None,
        };

        (next, Stmt::If(pairs, else_block))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_if() {
        assert_eq!(if_stmt("if(true){ a::b; }"), Ok(("", Stmt::If(vec![
            (expr("true").unwrap().1, block("{a::b;}").unwrap().1),
        ], None,
        ))));
        assert_eq!(stmt("if(true){ a::b; }"), Ok(("", Stmt::If(vec![
            (expr("true").unwrap().1, block("{a::b;}").unwrap().1),
        ], None,
        ))));

        assert_eq!(stmt("if(\"a\" == \"a\"){ a::b; }"), Ok(("", Stmt::If(vec![
            (expr("\"a\" == \"a\"").unwrap().1, block("{a::b;}").unwrap().1),
        ], None,
        ))));
        assert_eq!(stmt("if(foo() == \"a\"){ a::b; }"), Ok(("", Stmt::If(vec![
            (Expr::Eq(
                Box::new(Expr::FunctionCall("foo".to_string(), vec![])),
                Box::new(Expr::Value(ValueType::String("a".to_string()))),
            ),
             block("{a::b;}").unwrap().1),
        ], None,
        ))));
    }

    #[test]
    fn test_else_if() {
        assert_eq!(if_stmt("if(true){ a::b; }else if(false){ a::b; }"), Ok(("", Stmt::If(vec![
            (expr("true").unwrap().1, block("{a::b;}").unwrap().1),
            (expr("false").unwrap().1, block("{a::b;}").unwrap().1),
        ], None,
        ))));
    }

    #[test]
    fn test_else() {
        assert_eq!(if_stmt("if(true){ a::b; }else{ a::b; }"), Ok(("", Stmt::If(vec![
            (expr("true").unwrap().1, block("{a::b;}").unwrap().1),
        ], Some(block("{a::b;}").unwrap().1),
        ))));
    }
}