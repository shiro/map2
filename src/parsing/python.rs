use super::*;

pub fn parse_key_action_with_mods_py(raw: &str) -> Result<ParsedKeyAction> {
    let from = key_action_with_flags(raw).expect("failed to parse key");
    if !from.0.is_empty() { return Err(anyhow!("failed to parse key")); }
    let from = from.1;
    Ok(from.0)
}

pub fn parse_key_sequence_py(raw: &str) -> Result<Vec<ParsedKeyAction>> {
    // let raw = format!("\"{}\"", raw);
    let from = key_sequence(raw).expect("failed to parse key sequence");
    if !from.0.is_empty() { return Err(anyhow!("failed to parse key sequence")); }
    let from = from.1;
    Ok(from.0)
}
