use super::*;

pub(super) fn return_statement(input: &str) -> ResNew<&str, Stmt> {
    let (input, _) = tag_custom("return")(input)?;

    tuple((ws1, expr, ws0, tag_custom(";")))(input)
        .map(|(next, (_, (expr, last_err), _, _))| (next, (Stmt::Return(expr), last_err)))
}
