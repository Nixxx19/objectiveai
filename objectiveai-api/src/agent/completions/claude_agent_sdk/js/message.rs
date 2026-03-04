//! Generates the `const message = ...;` JS statement for the Claude Agent SDK subprocess.

use super::super::prompt::Prompt;

/// Builds the `const message = ...;` JS statement from the prompt's SDK user message.
pub fn build_message(prompt: &Prompt) -> Result<String, super::super::Error> {
    let json = serde_json::to_string(&prompt.message)
        .map_err(|e| super::super::Error::Json(e.to_string()))?;
    Ok(format!("    const message = {json};"))
}

#[cfg(test)]
#[path = "message_tests.rs"]
mod tests;
