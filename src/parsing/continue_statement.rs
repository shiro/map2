use super::*;

pub(super) fn continue_statement(input: &str) -> ResNew<&str, Stmt> {
    let (input, _) = tag_custom("continue")(input)
        .map_err(|_: NomErr<CustomError<_>>| make_generic_nom_err_options(input, vec!["continue".to_string()]))?;

    let (input, _) = ws0(input)?;

    let (input, _) = tag_custom(";")(input)?;

    Ok((input, (Stmt::Continue, None)))
}
