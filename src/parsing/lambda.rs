use super::*;


fn lambda(input: &str) -> Res<&str, ValueType> {
    context(
        "lambda",
        tuple((
            tag("|"),
            multispace0,
            tag("|"),
            multispace0,
            block,
        )),
    )(input).map(|(next, val)| (next, ValueType::Lambda(val.4)))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda() {
        assert_eq!(lambda("||{}"), Ok(("", ValueType::Lambda(Block::new()))));
        assert_eq!(lambda("||{ a::b; }"), Ok(("", ValueType::Lambda(
            block_body("a::b;").unwrap().1
        ))));
    }
}