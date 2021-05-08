use super::*;


pub(super) fn for_loop(input: &str) -> ResNew<&str, Stmt> {
    tuple((
        tag_custom("for"), ws0,
        tag_custom("("), ws0,
        expr, ws0,
        tag_custom(";"), ws0,
        expr, ws0,
        tag_custom(";"), ws0,
        expr, ws0,
        tag_custom(")"), ws0,
        block,
    ))(input)
        .map(|(next, v)| {
            let stmt = Stmt::For(v.4.0, v.8.0, v.12.0, v.16.0);
            (next, (stmt, None))
        })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_for_loop() {
        assert_eq!(
            for_loop("for(let i=0; i<20; i=i+1){}"),
            Ok(("", Stmt::For(
                Expr::Init("i".to_string(), Box::new(Expr::Value(ValueType::Number(0.0)))),
                Expr::LT(Box::new(Expr::Name("i".to_string())), Box::new(Expr::Value(ValueType::Number(20.0)))),
                expr("i=i+1").unwrap().1,
                Block::new(),
            )))
        );
    }
}
