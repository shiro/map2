use itertools::Itertools;
use crate::xkb::UTFToRawInputTransformer;

use super::*;

fn format_err(err: NomErr<CustomError<&str>>) -> Error {
    match err {
        NomErr::Error(err) => anyhow!("failed to parse key, expected one of: {}", err.expected.iter().unique().join(", ")),
        _ => anyhow!("failed to parse key mapping value")
    }
}

pub fn parse_key_action_with_mods_py(raw: &str, transformer: &UTFToRawInputTransformer) -> Result<ParsedKeyAction> {

    let from = key_action_with_flags_utf(Some(transformer))(raw)
        .map_err(format_err)?;
    if !from.0.is_empty() { return Err(anyhow!("failed to parse key")); }
    let from = from.1;
    Ok(from.0)
}

pub fn parse_key_sequence_py(raw: &str) -> Result<Vec<ParsedKeyAction>> {
    let from = key_sequence(raw)
        .map_err(format_err)?;
    if !from.0.is_empty() { return Err(anyhow!("failed to parse key sequence")); }
    let from = from.1;
    Ok(from.0)
}
