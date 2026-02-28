//! Vector response transformation for LLM prompts.
//!
//! Converts vector response options into prompt content parts with labeled keys.

/// Transforms vector responses into prompt content parts.
///
/// Formats responses as a JSON-like structure with prefix keys (e.g., `` `A` ``)
/// as labels, suitable for inclusion in the user message prompt.
pub fn into_parts_for_prompt(
    vector_responses: &[crate::agent::completions::request::RichContent],
    vector_pfx_indices: &[(String, usize)],
) -> Vec<crate::agent::completions::request::RichContentPart> {
    let mut parts = Vec::new();
    for (i, (vector_pfx_key, vector_response_index)) in vector_pfx_indices.iter().enumerate() {
        if i == 0 {
            parts.push(
                crate::agent::completions::request::RichContentPart::Text {
                    text: format!("{{\n    \"{}\": \"", vector_pfx_key),
                },
            );
        } else {
            parts.push(
                crate::agent::completions::request::RichContentPart::Text {
                    text: format!("\",\n    \"{}\": \"", vector_pfx_key),
                },
            );
        }
        match &vector_responses[*vector_response_index] {
            crate::agent::completions::request::RichContent::Text(text) => {
                parts.push(
                    crate::agent::completions::request::RichContentPart::Text {
                        text: json_escape::escape_str(text).to_string(),
                    },
                );
            }
            crate::agent::completions::request::RichContent::Parts(rich_parts) => {
                for rich_part in rich_parts {
                    match rich_part {
                        crate::agent::completions::request::RichContentPart::Text { text } => {
                            parts.push(
                                crate::agent::completions::request::RichContentPart::Text {
                                    text: json_escape::escape_str(text).to_string(),
                                },
                            );
                        }
                        part => {
                            parts.push(part.clone());
                        }
                    }
                }
            }
        }
    }
    if parts.len() > 0 {
        parts.push(
            crate::agent::completions::request::RichContentPart::Text {
                text: "\"\n}".to_string(),
            },
        );
    }
    parts
}
