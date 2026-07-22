mod prompt;
#[cfg(test)]
mod tests;

use dialoguer::Confirm;
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

pub(crate) async fn handle_interactive_confirmations(
    client: &IbkrClient,
    mut submission_response: Value,
    max_replies: u32,
) -> Result<Value, WorkerError> {
    for _ in 0..max_replies {
        let Some(server_prompt) = order_prompt(&submission_response)? else {
            return Ok(final_order_response(submission_response));
        };
        let should_confirm = Confirm::new()
            .with_prompt(server_prompt.describe())
            .default(false)
            .interact()?;
        submission_response = client
            .reply(&server_prompt.reply_id, should_confirm)
            .await?;
        if !should_confirm {
            return Ok(submission_response);
        }
    }

    Err(too_many_replies(max_replies, submission_response))
}
