use super::*;

pub(crate) fn parse_script<>(raw_script: &str) -> Result<Block> {
    match global_block(raw_script) {
        Ok(v) => {
            if v.0.is_empty() {
                Ok(v.1)
            } else {
                Err(anyhow!("parsing failed, remaining input:\n'{}'\n", v.0))
            }
        }
        Err(err) => Err(anyhow!("parsing failed: {}", err))
    }
}

// pub(crate) fn parse_key_sequence<>(raw: &str) -> Result<Vec<KeyActionWithMods>> {
//     match key_sequence(raw) {
//         Ok(v) => {
//             if v.0.is_empty() {
//                 Ok(v.1)
//             } else {
//                 Err(anyhow!("parsing failed, remaining input:\n'{}'\n", v.0))
//             }
//         }
//         Err(err) => Err(anyhow!("parsing failed: {}", err))
//     }
// }


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_sequence() {
        // assert_eq!(parse_key_sequence("hello{enter}world").unwrap(),
        //            vec![].tap_mut(|v| v.append_string_sequence("hello{enter}world").unwrap()));
    }
}