use super::*;
use unicode_xid::UnicodeXID;

use crate::parsing::{make_generic_nom_err, Res};

pub(super) fn ident(input: &str) -> ResNew<&str, String> {
    let (rest, id) = match word(input) {
        Ok((rest, id)) => (rest, id),
        Err(err) => return Err(make_generic_nom_err_options(input, vec!["identifier".to_string()])),
    };

    match id.0.as_ref() {
        // From https://doc.rust-lang.org/grammar.html#keywords
        "abstract" | "alignof" | "as" | "become" | "box" | "break" | "const" | "continue" |
        "crate" | "do" | "else" | "enum" | "extern" | "false" | "final" | "fn" | "for" |
        "if" | "impl" | "in" | "let" | "loop" | "macro" | "match" | "mod" | "move" |
        "mut" | "offsetof" | "override" | "priv" | "proc" | "pub" | "pure" | "ref" |
        "return" | "Self" | "self" | "sizeof" | "static" | "struct" | "super" | "trait" |
        "true" | "type" | "typeof" | "unsafe" | "unsized" | "use" | "virtual" | "where" |
        "while" | "yield" => Err(make_generic_nom_err_new(input)),
        _ => Ok((rest, id)),
    }
}

pub(super) fn word(input: &str) -> ResNew<&str, String> {
    let (input, _) = ws0(input)?;

    let mut chars = input.char_indices();
    match chars.next() {
        Some((_, ch)) if UnicodeXID::is_xid_start(ch) || ch == '_' => {}
        _ => return Err(make_generic_nom_err_options(input, vec!["word".to_string()])),
    }

    while let Some((i, ch)) = chars.next() {
        if !UnicodeXID::is_xid_continue(ch) {
            return Ok((&input[i..], (input[..i].into(), None)));
        }
    }

    Ok(("", (input.into(), None)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_weird_names() {
        assert_eq!(ident("_foobar"), Ok(("", "_foobar".to_string())));
        assert_eq!(ident("btn_forward"), Ok(("", "btn_forward".to_string())));
    }
}
