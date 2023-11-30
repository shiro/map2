use itertools::Itertools;
use crate::xkb::XKBTransformer;

use super::*;

fn format_err(err: NomErr<CustomError<&str>>, input: &str) -> Error {
    match err {
        NomErr::Error(err) => {
            if err.expected.len() > 0 {
                anyhow!("failed to parse input '{}', expected one of: {}", input, err.expected.iter().unique().join(", "))
            } else {
                anyhow!("failed to parse input '{}'", input)
            }
        }
        _ => anyhow!("failed to parse key mapping value")
    }
}

pub fn parse_key_action_with_mods(raw: &str, transformer: Option<&XKBTransformer>) -> Result<ParsedKeyAction> {
    let from = single_key_action_utf_with_flags_utf(transformer)(raw)
        .map_err(|err| format_err(err, raw))?;

    if !from.0.is_empty() { return Err(anyhow!("expected exactly 1 key action")); }

    let from = from.1;
    Ok(from)
}

pub fn parse_key_sequence(raw: &str, transformer: Option<&XKBTransformer>) -> Result<Vec<ParsedKeyAction>> {
    let (rest, res) = key_sequence_utf(transformer)(raw)
        .map_err(|err| format_err(err, raw))?;

    if !rest.is_empty() { return Err(anyhow!("failed to parse key sequence")); }

    Ok(res)
}
