// IBKR's REST API gates risky orders behind confirmation prompts — e.g.
// "price exceeds the Percentage constraint", "You are submitting an order
// without market data". An `Answers` map says, ahead of time, how to reply
// to each prompt: `true` to accept and continue, `false` to decline. The
// confirmation engine (`commands::order::confirmations`) consults this map
// when it receives a `messageId`/`message` from IBKR and only auto-replies
// when a match is found.
//
// Keys may be exact `messageId`s (e.g. `o354`) or substrings of the prompt
// message; see `find_answer` for the precedence rule.

use std::collections::BTreeMap;

use crate::Result;

pub type Answers = BTreeMap<String, bool>;

pub fn default_answers() -> Answers {
    [
        ("o163", true),
        ("price exceeds the Percentage constraint", true),
        ("o451", true),
        ("exceeds the Total Value Limit", true),
        ("o354", true),
        ("You are submitting an order without market data", true),
        ("o10331", true),
        ("You are about to submit a stop order", true),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect()
}

pub fn parse_answers_json(input: &str) -> Result<Answers> {
    Ok(serde_json::from_str(input)?)
}

// Folds caller-supplied overrides into the defaults. Override entries win on
// matching keys, and any new keys are added to the resulting map. Defaults
// for keys not mentioned in the overrides are preserved — so a caller who
// only wants to flip one specific prompt to `false` does not have to
// re-spell the entire default set.
pub fn merge_answers(mut defaults: Answers, overrides: Answers) -> Answers {
    defaults.extend(overrides);
    defaults
}

pub fn find_answer<'a>(
    message: &str,
    message_id: Option<&str>,
    answers: &'a Answers,
) -> Option<&'a bool> {
    if let Some(message_id) = message_id {
        if let Some(answer) = answers.get(message_id) {
            return Some(answer);
        }
    }
    answers
        .iter()
        .find_map(|(key, answer)| message.contains(key).then_some(answer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_answers_include_ibind_example_constants() {
        let answers = default_answers();

        for key in [
            "o163",
            "price exceeds the Percentage constraint",
            "o451",
            "exceeds the Total Value Limit",
            "o354",
            "You are submitting an order without market data",
            "o10331",
            "You are about to submit a stop order",
        ] {
            assert_eq!(answers.get(key), Some(&true), "missing key: {key}");
        }
    }

    #[test]
    fn default_answers_match_by_message_id() {
        let answers = default_answers();
        assert_eq!(
            find_answer("different text", Some("o354"), &answers),
            Some(&true)
        );
    }

    #[test]
    fn default_answers_match_by_substring() {
        let answers = default_answers();
        assert_eq!(
            find_answer(
                "The following order exceeds the Total Value Limit of 100,000 USD.",
                None,
                &answers
            ),
            Some(&true)
        );
    }

    #[test]
    fn matches_message_id_before_substring() {
        let answers = parse_answers_json(r#"{"o354":true,"missing data":false}"#).unwrap();
        assert_eq!(
            find_answer("missing data", Some("o354"), &answers),
            Some(&true)
        );
    }

    #[test]
    fn merge_keeps_default_keys_absent_from_overrides() {
        let merged = merge_answers(
            default_answers(),
            parse_answers_json(r#"{"o354":false}"#).unwrap(),
        );

        assert_eq!(merged.get("o354"), Some(&false));
        assert_eq!(merged.get("o163"), Some(&true));
        assert_eq!(merged.get("o451"), Some(&true));
    }

    #[test]
    fn merge_adds_new_override_keys() {
        let merged = merge_answers(
            default_answers(),
            parse_answers_json(r#"{"custom warning":true}"#).unwrap(),
        );

        assert_eq!(merged.get("custom warning"), Some(&true));
        assert_eq!(merged.len(), default_answers().len() + 1);
    }

    #[test]
    fn merge_with_empty_overrides_is_identity() {
        let merged = merge_answers(default_answers(), Answers::new());
        assert_eq!(merged, default_answers());
    }

    #[test]
    fn matches_message_substring() {
        let answers = parse_answers_json(r#"{"Percentage constraint":true}"#).unwrap();
        assert_eq!(
            find_answer(
                "price exceeds the Percentage constraint of 3%",
                None,
                &answers
            ),
            Some(&true)
        );
    }
}
