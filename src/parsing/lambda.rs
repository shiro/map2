use super::*;

pub(super) fn lambda(input: &str) -> Res<&str, Expr> {
    context(
        "lambda",
        tuple((
            tag("|"),
            ws0,
            opt((tuple((
                ident,
                ws0,
                many0(tuple((
                    tag(","),
                    ws0,
                    ident,
                    ws0,
                ))),
            )))),
            tag("|"),
            ws0,
            block,
        )),
    )(input).map(|(next, val)| {
        let params = match val.2 {
            Some(v) => {
                let first = v.0;
                let mut params: Vec<String> = v.2.into_iter().map(|v| v.2.to_string()).collect();
                params.insert(0, first);
                params
            }
            None => vec![],
        };

        (next, Expr::Lambda(params, val.5))
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda() {
        assert_eq!(lambda("||{}"), Ok(("", Expr::Lambda(vec![], Block::new()))));
        assert_eq!(lambda("|a|{ a::b; }"), Ok(("", Expr::Lambda(
            vec!["a".to_string()],
            block_body("a::b;").unwrap().1,
        ))));
    }
}