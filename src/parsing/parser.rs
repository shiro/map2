
use super::*;

pub(crate) fn parse_script<>(raw_script: &str) -> Result<Block> {
    match global_block(raw_script) {
        Ok((v, (block, last_err))) => {
            if v.is_empty() {
                Ok(block)
            } else {
                if let Some(last_err) = last_err {
                    let err = convert_custom_error(raw_script, &last_err);
                    eprintln!("{}", &*err);
                } else {
                    eprintln!("generic error");
                }

                Err(anyhow!("parsing failed unwrapping..."))
                // Err(anyhow!("parsing failed, remaining input:\n'{}'\n", v))
            }
        }
        Err(err) => {
            if let NomErr::Error(err) = &err {
                let err = convert_custom_error(raw_script, err);
                eprintln!("{}", &*err);
            }

            Err(anyhow!("parsing failed: {}", err))
        }
    }
}

pub(crate) fn parse_key_sequence(raw: &str) -> Result<Vec<KeyAction>> {
    // TODO remove this workaround (allow seq to be parsed without quotes)
    let raw = format!("\"{}\"", raw);
    match key_sequence(&raw) {
        Ok(v) => {
            if v.0.is_empty() {
                Ok(v.1.0.to_key_actions())
            } else {
                Err(anyhow!("parsing failed, remaining input:\n'{}'\n", v.0))
            }
        }
        Err(err) => Err(anyhow!("parsing failed: {}", err))
    }
}

pub(crate) fn parse_key_action_with_mods(from: &str, to: Block) -> Result<Expr> {
    let from = key_action_with_flags(from).expect("failed to parse mapping trigger");
    if !from.0.is_empty() { return Err(anyhow!("failed to parse mapping trigger")); }
    let from = from.1;

    let expr = match from.0 {
        ParsedKeyAction::KeyClickAction(from) => { Expr::map_key_click_block(from, to) }
        ParsedKeyAction::KeyAction(from) => { Expr::map_key_block(from, to) }
    };

    Ok(expr)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_sequence() {
        assert_eq!(parse_key_sequence("hello{enter}world").unwrap(),
                   nom_eval(key_sequence("\"hello{enter}world\"")).to_key_actions(),
        );
    }
}