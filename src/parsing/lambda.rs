use super::*;

pub(super) fn lambda(input: &str) -> ResNew<&str, Expr> {
    let (input, _) = tag_custom("|")(input)
        .map_err(|_: NomErr<CustomError<_>>| make_generic_nom_err_options(input, vec!["lambda".to_string()]))?;

    tuple((
        ws0,
        opt((tuple((
            ident,
            ws0,
            many0(tuple((
                tag_custom(","),
                ws0,
                ident,
                ws0,
            ))),
        )))),
        tag_custom("|"),
        ws0,
        block,
    )
    )(input)
        .map(|(next, val)| {
            let params = match val.1 {
                Some(v) => {
                    let first = v.0.0;
                    let mut params: Vec<String> = v.2.into_iter().map(|v| v.2.0.to_string()).collect();
                    params.insert(0, first);
                    params
                }
                None => vec![],
            };

            let expr = Expr::Lambda(params, val.4.0);
            (next, (expr, val.4.1))
        })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lambda() {
        assert_eq!(lambda("||{}"), nom_ok(Expr::Lambda(vec![], Block::new())));
        assert_eq!(lambda("|a|{ a::b; }"), nom_ok(Expr::Lambda(
            vec!["a".to_string()],
            nom_eval(block_body("a::b;")),
        )));
    }
}