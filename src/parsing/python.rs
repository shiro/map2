use itertools::Itertools;
use crate::xkb::XKBTransformer;

use super::*;

fn format_err(err: NomErr<CustomError<&str>>) -> Error {
    match err {
        NomErr::Error(err) => anyhow!("failed to parse key, expected one of: {}", err.expected.iter().unique().join(", ")),
        _ => anyhow!("failed to parse key mapping value")
    }
}

pub fn parse_key_action_with_mods_py(raw: &str, transformer: Option<&XKBTransformer>) -> Result<ParsedKeyAction> {
    let from = key_action_with_flags_utf(transformer)(raw)
        .map_err(format_err)?;

    if !from.0.is_empty() { return Err(anyhow!("expected exactly 1 key action")); }

    let from = from.1;
    Ok(from)
}

pub fn parse_key_sequence_py(raw: &str, transformer: Option<&XKBTransformer>) -> Result<Vec<ParsedKeyAction>> {
    let (rest, res) = key_sequence_utf(transformer)(raw)
        .map_err(format_err)?;

    if !rest.is_empty() { return Err(anyhow!("failed to parse key sequence")); }

    Ok(res)
}
