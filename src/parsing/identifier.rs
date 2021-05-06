use super::*;
use unicode_xid::UnicodeXID;

use crate::parsing::{make_generic_nom_err, Res};

pub fn ident(input: &str) -> Res<&str, String> {
    let (rest, id) = match word(input) {
        Ok((rest, id)) => (rest, id),
        Err(err) => return Err(err),
    };

    match id.as_ref() {
        // From https://doc.rust-lang.org/grammar.html#keywords
        "abstract" | "alignof" | "as" | "become" | "box" | "break" | "const" | "continue" |
        "crate" | "do" | "else" | "enum" | "extern" | "false" | "final" | "fn" | "for" |
        "if" | "impl" | "in" | "let" | "loop" | "macro" | "match" | "mod" | "move" |
        "mut" | "offsetof" | "override" | "priv" | "proc" | "pub" | "pure" | "ref" |
        "return" | "Self" | "self" | "sizeof" | "static" | "struct" | "super" | "trait" |
        "true" | "type" | "typeof" | "unsafe" | "unsized" | "use" | "virtual" | "where" |
        "while" | "yield" => Err(make_generic_nom_err()),
        _ => Ok((rest, id)),
    }
}

pub fn word(input: &str) -> Res<&str, String> {
    let (input, _) = ws0(input)?;

    let mut chars = input.char_indices();
    match chars.next() {
        Some((_, ch)) if UnicodeXID::is_xid_start(ch) || ch == '_' => {}
        _ => return Err(make_generic_nom_err()),
    }

    while let Some((i, ch)) = chars.next() {
        if !UnicodeXID::is_xid_continue(ch) {
            return Ok((&input[i..], input[..i].into()));
        }
    }

    Ok(("", input.into()))
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
