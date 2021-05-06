use super::*;


pub(crate) fn for_loop(input: &str) -> Res<&str, Stmt> {
    context(
        "for_loop",
        tuple((
            tag("for"), ws0,
            tag("("), ws0,
            expr, ws0,
            tag(";"), ws0,
            expr, ws0,
            tag(";"), ws0,
            expr, ws0,
            tag(")"), ws0,
            block,
        )),
    )(input).map(|(next, v)| { (next, Stmt::For(v.4, v.8, v.12, v.16)) })
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
