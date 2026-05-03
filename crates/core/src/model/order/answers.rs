use std::collections::BTreeMap;

use crate::Result;

pub type Answers = BTreeMap<String, bool>;

pub fn parse_answers_json(input: &str) -> Result<Answers> {
    Ok(serde_json::from_str(input)?)
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
    fn matches_message_id_before_substring() {
        let answers = parse_answers_json(r#"{"o354":true,"missing data":false}"#).unwrap();
        assert_eq!(
            find_answer("missing data", Some("o354"), &answers),
            Some(&true)
        );
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
