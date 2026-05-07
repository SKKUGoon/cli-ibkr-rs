mod prompt;
#[cfg(test)]
mod tests;

use ibkr_core::model::order::{find_answer, Answers};
use ibkr_core::IbkrClient;
use serde_json::Value;

use crate::error::WorkerError;

#[cfg(test)]
use prompt::OrderPrompt;
use prompt::{final_order_response, order_prompt, too_many_replies};

pub async fn handle_confirmations(
    client: &IbkrClient,
    mut value: Value,
    answers: &Answers,
    max_replies: u32,
) -> Result<Value, WorkerError> {
    for _ in 0..max_replies {
        let Some(prompt) = order_prompt(&value)? else {
            return Ok(final_order_response(value));
        };

        match find_answer(&prompt.message, prompt.message_id.as_deref(), answers) {
            Some(true) => {
                value = client.reply(&prompt.reply_id, true).await?;
            }
            Some(false) => return Err(WorkerError::RejectedOrderAnswer(prompt.describe())),
            None => return Err(WorkerError::MissingOrderAnswer(prompt.describe())),
        }
    }

    Err(too_many_replies(max_replies, value))
}
