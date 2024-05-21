use super::*;

pub fn action_state(input: &str) -> ParseResult<&str, i32> {
    map(alt((tag_custom_no_case("down"), tag_custom_no_case("up"), tag_custom_no_case("repeat"))), |input: &str| {
        match &*input.to_lowercase() {
            "up" => 0,
            "down" => 1,
            "repeat" => 2,
            _ => unreachable!(),
        }
    })(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_state_input() {
        assert_eq!(action_state("up"), nom_ok(0));
        assert_eq!(action_state("down"), nom_ok(1));
        assert_eq!(action_state("repeat"), nom_ok(2));
    }

    #[test]
    fn action_state_case() {
        assert_eq!(action_state("UP"), nom_ok(0));
        assert_eq!(action_state("DOWN"), nom_ok(1));
        assert_eq!(action_state("REPEAT"), nom_ok(2));

        assert_eq!(action_state("DoWn"), nom_ok(1));
    }
}
