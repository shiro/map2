use itertools::Itertools;
use crate::xkb::XKBTransformer;

use super::*;

fn format_err(err: NomErr<CustomError<&str>>, input: &str, pos: usize) -> Error {
    match err {
        NomErr::Error(err) => {
            if err.expected.len() > 0 {
                anyhow!(
                    "{}\n{: >pos$}^ expected one of: {}",
                    input,
                    "",
                    err.expected.iter().unique().join(", ")
                )
            } else {
                anyhow!(
                    "{}\n{: >pos$}^ failed here",
                    input,
                    "",
                )
            }
        }
        _ => anyhow!("failed to parse key mapping value")
    }
}

pub fn parse_key_action_with_mods(raw: &str, transformer: Option<&XKBTransformer>) -> Result<ParsedKeyAction> {
    let (rest, from) = single_key_action_utf_with_flags_utf(transformer)(raw)
        .map_err(|err| format_err(err, raw, 0))?;

    if !rest.is_empty() { return Err(anyhow!("expected exactly 1 key action from input '{}'", raw)); }

    Ok(from)
}

pub fn parse_key_sequence(raw: &str, transformer: Option<&XKBTransformer>) -> Result<Vec<ParsedKeyAction>> {
    let (rest, (res, last_err)) = key_sequence_utf(transformer)(raw)
        .map_err(|err| format_err(err, raw, 0))?;

    if !rest.is_empty() {
        return Err(
            format_err(NomErr::Error(last_err), raw, raw.len() - rest.len())
        );
    }

    Ok(res)
}
