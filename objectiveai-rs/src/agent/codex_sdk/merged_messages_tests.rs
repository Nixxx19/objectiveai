use crate::agent::completions::message::{
    Message, RichContent, SimpleContent, SystemMessage, UserMessage,
};
use crate::agent::codex_sdk;

#[test]
fn no_system_no_prefix_no_suffix() {
    let agent = codex_sdk::AgentBase {
        model: "gpt-5".to_string(),
        ..Default::default()
    };
    let messages = vec![
        Message::User(UserMessage {
            content: RichContent::Text("hello".to_string()),
            name: None,
        }),
    ];
    let merged = agent.merged_messages(messages);
    assert_eq!(merged, vec![
        Message::User(UserMessage {
            content: RichContent::Text("hello".to_string()),
            name: None,
        }),
    ]);
}

#[test]
fn prefix_content_only() {
    let agent = codex_sdk::AgentBase {
        model: "gpt-5".to_string(),
        prefix_content: Some(RichContent::Text("context info".to_string())),
        ..Default::default()
    };
    let messages = vec![
        Message::User(UserMessage {
            content: RichContent::Text("user".to_string()),
            name: None,
        }),
    ];
    let merged = agent.merged_messages(messages);
    assert_eq!(merged, vec![
        Message::User(UserMessage {
            content: RichContent::Text("context info".to_string()),
            name: None,
        }),
        Message::User(UserMessage {
            content: RichContent::Text("user".to_string()),
            name: None,
        }),
    ]);
}

#[test]
fn suffix_content_only() {
    let agent = codex_sdk::AgentBase {
        model: "gpt-5".to_string(),
        suffix_content: Some(RichContent::Text("trailing".to_string())),
        ..Default::default()
    };
    let messages = vec![
        Message::User(UserMessage {
            content: RichContent::Text("user".to_string()),
            name: None,
        }),
    ];
    let merged = agent.merged_messages(messages);
    assert_eq!(merged, vec![
        Message::User(UserMessage {
            content: RichContent::Text("user".to_string()),
            name: None,
        }),
        Message::User(UserMessage {
            content: RichContent::Text("trailing".to_string()),
            name: None,
        }),
    ]);
}

#[test]
fn prefix_and_suffix_with_inner_system() {
    let agent = codex_sdk::AgentBase {
        model: "gpt-5".to_string(),
        prefix_content: Some(RichContent::Text("ctx".to_string())),
        suffix_content: Some(RichContent::Text("post".to_string())),
        ..Default::default()
    };
    let messages = vec![
        Message::System(SystemMessage {
            content: SimpleContent::Text("inner-system".to_string()),
            name: None,
        }),
        Message::User(UserMessage {
            content: RichContent::Text("user".to_string()),
            name: None,
        }),
    ];
    let merged = agent.merged_messages(messages);
    assert_eq!(merged, vec![
        Message::System(SystemMessage {
            content: SimpleContent::Text("inner-system".to_string()),
            name: None,
        }),
        Message::User(UserMessage {
            content: RichContent::Text("ctx".to_string()),
            name: None,
        }),
        Message::User(UserMessage {
            content: RichContent::Text("user".to_string()),
            name: None,
        }),
        Message::User(UserMessage {
            content: RichContent::Text("post".to_string()),
            name: None,
        }),
    ]);
}
