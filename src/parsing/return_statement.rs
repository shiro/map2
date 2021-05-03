use super::*;

pub(super) fn return_statement(input: &str) -> Res<&str, Stmt> {
    context(
        "return_statement",
        tuple((tag("return"), multispace1, expr, multispace0, tag(";"))),
    )(input).map(|(next, val)| (next, Stmt::Return(val.2)))
}
