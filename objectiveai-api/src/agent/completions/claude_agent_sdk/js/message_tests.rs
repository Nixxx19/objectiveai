use super::super::super::content_block_param::*;
use super::super::super::prompt::Prompt;
use super::super::super::sdk_message::*;
use super::build_message;

fn prompt(system_prompt: Option<&str>, session_id: &str, content: MessageParamContent) -> Prompt {
    Prompt {
        system_prompt: system_prompt.map(|s| s.to_string()),
        message: SDKUserMessage {
            r#type: SDKUserMessageType::User,
            message: MessageParam {
                content,
                role: MessageParamRole::User,
            },
            parent_tool_use_id: None,
            is_synthetic: None,
            tool_use_result: None,
            uuid: None,
            session_id: session_id.to_string(),
        },
    }
}

fn text_block(text: &str) -> ContentBlockParam {
    ContentBlockParam::Text(TextBlockParam {
        text: text.to_string(),
        r#type: TextBlockParamType::Text,
        cache_control: None,
        citations: None,
    })
}

fn blocks(blocks: Vec<ContentBlockParam>) -> MessageParamContent {
    MessageParamContent::Blocks(blocks)
}

#[test]
fn test_empty_content() {
    let p = prompt(None, "", blocks(vec![]));
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":[],"role":"user"},"parent_tool_use_id":null,"session_id":""};"#,
    );
}

#[test]
fn test_simple_text() {
    let p = prompt(None, "", blocks(vec![text_block("Hello world")]));
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":[{"text":"Hello world","type":"text"}],"role":"user"},"parent_tool_use_id":null,"session_id":""};"#,
    );
}

#[test]
fn test_multiple_text_blocks() {
    let p = prompt(
        None,
        "",
        blocks(vec![text_block("First"), text_block("Second")]),
    );
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":[{"text":"First","type":"text"},{"text":"Second","type":"text"}],"role":"user"},"parent_tool_use_id":null,"session_id":""};"#,
    );
}

#[test]
fn test_with_session_id() {
    let p = prompt(None, "sess-abc-123", blocks(vec![text_block("hi")]));
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":[{"text":"hi","type":"text"}],"role":"user"},"parent_tool_use_id":null,"session_id":"sess-abc-123"};"#,
    );
}

#[test]
fn test_system_prompt_not_in_message() {
    // system_prompt is on the Prompt, not serialized into the message
    let p = prompt(
        Some("You are helpful"),
        "",
        blocks(vec![text_block("What is 2+2?")]),
    );
    let result = build_message(&p).unwrap();
    assert!(!result.contains("You are helpful"));
    assert_eq!(
        result,
        r#"    const message = {"type":"user","message":{"content":[{"text":"What is 2+2?","type":"text"}],"role":"user"},"parent_tool_use_id":null,"session_id":""};"#,
    );
}

#[test]
fn test_string_content() {
    let p = prompt(None, "", MessageParamContent::String("plain text".to_string()));
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":"plain text","role":"user"},"parent_tool_use_id":null,"session_id":""};"#,
    );
}

#[test]
fn test_special_characters_in_text() {
    let p = prompt(
        None,
        "",
        blocks(vec![text_block("line1\nline2\ttab\"quote\\backslash")]),
    );
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":[{"text":"line1\nline2\ttab\"quote\\backslash","type":"text"}],"role":"user"},"parent_tool_use_id":null,"session_id":""};"#,
    );
}

#[test]
fn test_image_block() {
    let p = prompt(
        None,
        "",
        blocks(vec![ContentBlockParam::Image(ImageBlockParam {
            r#type: ImageBlockParamType::Image,
            source: ImageSource::Base64(Base64ImageSource {
                r#type: Base64ImageSourceType::Base64,
                media_type: Base64ImageSourceMediaType::ImagePng,
                data: "iVBOR".to_string(),
            }),
            cache_control: None,
        })]),
    );
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":[{"source":{"data":"iVBOR","media_type":"image/png","type":"base64"},"type":"image"}],"role":"user"},"parent_tool_use_id":null,"session_id":""};"#,
    );
}

#[test]
fn test_mixed_text_and_image() {
    let p = prompt(
        None,
        "session-42",
        blocks(vec![
            text_block("Describe this:"),
            ContentBlockParam::Image(ImageBlockParam {
                r#type: ImageBlockParamType::Image,
                source: ImageSource::Base64(Base64ImageSource {
                    r#type: Base64ImageSourceType::Base64,
                    media_type: Base64ImageSourceMediaType::ImageJpeg,
                    data: "abc123".to_string(),
                }),
                cache_control: None,
            }),
        ]),
    );
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":[{"text":"Describe this:","type":"text"},{"source":{"data":"abc123","media_type":"image/jpeg","type":"base64"},"type":"image"}],"role":"user"},"parent_tool_use_id":null,"session_id":"session-42"};"#,
    );
}

#[test]
fn test_document_block() {
    let p = prompt(
        None,
        "",
        blocks(vec![ContentBlockParam::Document(DocumentBlockParam {
            r#type: DocumentBlockParamType::Document,
            source: DocumentSource::Base64PDF(Base64PDFSource {
                r#type: Base64PDFSourceType::Base64,
                media_type: Base64PDFSourceMediaType::ApplicationPdf,
                data: "JVBER".to_string(),
            }),
            cache_control: None,
            title: None,
            context: None,
            citations: None,
        })]),
    );
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":[{"source":{"data":"JVBER","media_type":"application/pdf","type":"base64"},"type":"document"}],"role":"user"},"parent_tool_use_id":null,"session_id":""};"#,
    );
}

#[test]
fn test_unicode_content() {
    let p = prompt(None, "", blocks(vec![text_block("日本語テスト 🎉")]));
    assert_eq!(
        build_message(&p).unwrap(),
        r#"    const message = {"type":"user","message":{"content":[{"text":"日本語テスト 🎉","type":"text"}],"role":"user"},"parent_tool_use_id":null,"session_id":""};"#,
    );
}
