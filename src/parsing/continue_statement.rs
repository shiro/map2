use super::*;

pub(super) fn continue_statement(input: &str) -> ResNew<&str, Stmt> {
    let (input, _) = tag_custom("continue")(input)?;
    let (input, _) = ws0(input)?;
    let (input, _) = tag_custom(";")(input)?;

    Ok((input, (Stmt::Continue, None)))
}
