use super::*;

pub(super) fn if_stmt(input: &str) -> ResNew<&str, Stmt> {
    let (input, _) = tag_custom("if")(input)?;

    let (input, v) = tuple((
        ws0,
        tag_custom("("),
        ws0,
        expr,
        ws0,
        tag_custom(")"),
        ws0,
        block,
        many0_err(tuple((
            ws0,
            tag_custom("else"),
            ws1,
            tag_custom("if"),
            ws0,
            tag_custom("("),
            ws0,
            expr,
            ws0,
            tag_custom(")"),
            ws0,
            block,
        ))),
    ))(input)?;
    let mut last_err = Some(v.8.1);
    let mut pairs: Vec<(Expr, Block)> = v.8.0.into_iter().map(|v| (v.7.0, v.11.0)).collect();
    let first_pair = (v.3.0, v.7.0);
    pairs.insert(0, first_pair);

    let mut input = input;
    let mut else_block = None;
    if let Ok((i, _)) = tuple::<_, _, CustomError<&str>, _>((ws0, tag_custom("else")))(input) {
        let is_followed_by_if = tuple::<_, _, CustomError<&str>, _>((ws1, tag_custom("if")))(i).is_ok();

        if !is_followed_by_if {
            let (i, res) = tuple((ws0, block))(i)?;
            else_block = Some(res.1.0);
            last_err = None;
            input = i;
        }
    };

    let stmt = Stmt::If(pairs, else_block);
    Ok((input, (stmt, last_err)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_if() {
        assert_eq!(if_stmt("if(true){ a::b; }"), nom_ok( Stmt::If(vec![
            (nom_eval(expr("true")), nom_eval(block("{a::b;}"))),
        ], None,
        )));
        assert_eq!(stmt("if(true){ a::b; }"), nom_ok( Stmt::If(vec![
            (nom_eval(expr("true")), nom_eval(block("{a::b;}"))),
        ], None,
        )));

        assert_eq!(stmt("if(\"a\" == \"a\"){ a::b; }"), nom_ok( Stmt::If(vec![
            (nom_eval(expr("\"a\" == \"a\"")), nom_eval(block("{a::b;}"))),
        ], None,
        )));
        assert_eq!(stmt("if(foo() == \"a\"){ a::b; }"), nom_ok( Stmt::If(vec![
            (Expr::Eq(
                Box::new(Expr::FunctionCall("foo".to_string(), vec![])),
                Box::new(Expr::Value(ValueType::String("a".to_string()))),
            ),
             nom_eval(block("{a::b;}"))),
        ], None,
        )));
    }

    #[test]
    fn test_else_if() {
        assert_eq!(if_stmt("if(true){ a::b; }else if(false){ a::b; }"), nom_ok( Stmt::If(vec![
            (nom_eval(expr("true")), nom_eval(block("{a::b;}"))),
            (nom_eval(expr("false")), nom_eval(block("{a::b;}"))),
        ], None,
        )));
    }

    #[test]
    fn test_else() {
        assert_eq!(if_stmt("if(true){ a::b; }else{ a::b; }"), nom_ok( Stmt::If(vec![
            (nom_eval(expr("true")), nom_eval(block("{a::b;}"))),
        ], Some(nom_eval(block("{a::b;}"))),
        )));
    }
}