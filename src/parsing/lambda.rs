use super::*;

pub(super) fn lambda(input: &str) -> Res<&str, Expr> {
    context(
        "lambda",
        tuple((
            tag("|"),
            multispace0,
            tag("|"),
            multispace0,
            block,
        )),
    )(input).map(|(next, val)| (next, Expr::Lambda(val.4)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda() {
        assert_eq!(lambda("||{}"), Ok(("", Expr::Lambda(Block::new()))));
        assert_eq!(lambda("||{ a::b; }"), Ok(("", Expr::Lambda(
            block_body("a::b;").unwrap().1
        ))));
    }
}