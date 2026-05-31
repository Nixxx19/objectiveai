//! Mechanical bridge from the SDK's `ContentBlock` family to
//! `rmcp::model::Content`. The two type families have the same five
//! variants; the bridge is one match arm per variant.
//!
//! Only `Text`, `Image`, `Audio` are ever produced by our formatter
//! today (the SDK's `From<RichContentPart> for ContentBlock` never
//! emits `EmbeddedResource` or `ResourceLink`), so the
//! resource-shaped arms exist for completeness and as a no-op
//! safety net.

use objectiveai_sdk::mcp::shared::ResourceContentsUnion;
use objectiveai_sdk::mcp::tool::ContentBlock;
use rmcp::model::{
    Annotated, Content, RawAudioContent, RawContent, RawResource,
    RawTextContent, ResourceContents,
};

/// Convert one SDK `ContentBlock` into one rmcp `Content`. Drops
/// SDK-side `annotations` (rmcp's `Content::no_annotation()` /
/// `Annotated { annotations: None }` is what `Content::text` /
/// `Content::image` produce, and our formatter doesn't carry
/// annotations through).
pub fn into_rmcp_content(block: ContentBlock) -> Content {
    match block {
        ContentBlock::Text(t) => Annotated {
            raw: RawContent::Text(RawTextContent {
                text: t.text,
                meta: None,
            }),
            annotations: None,
        },
        ContentBlock::Image(i) => Annotated {
            raw: RawContent::Image(rmcp::model::RawImageContent {
                data: i.data,
                mime_type: i.mime_type,
                meta: None,
            }),
            annotations: None,
        },
        ContentBlock::Audio(a) => Annotated {
            raw: RawContent::Audio(RawAudioContent {
                data: a.data,
                mime_type: a.mime_type,
            }),
            annotations: None,
        },
        ContentBlock::EmbeddedResource(er) => Annotated {
            raw: RawContent::Resource(rmcp::model::RawEmbeddedResource {
                resource: rcu_to_rmcp(er.resource),
                meta: None,
            }),
            annotations: None,
        },
        ContentBlock::ResourceLink(rl) => Annotated {
            raw: RawContent::ResourceLink(RawResource {
                uri: rl.uri,
                name: rl.name,
                title: rl.title,
                description: rl.description,
                mime_type: rl.mime_type,
                size: None,
                icons: None,
                meta: None,
            }),
            annotations: None,
        },
    }
}

fn rcu_to_rmcp(rcu: ResourceContentsUnion) -> ResourceContents {
    match rcu {
        ResourceContentsUnion::Text(t) => ResourceContents::TextResourceContents {
            uri: t.base.uri,
            mime_type: t.base.mime_type,
            text: t.text,
            meta: None,
        },
        ResourceContentsUnion::Blob(b) => ResourceContents::BlobResourceContents {
            uri: b.base.uri,
            mime_type: b.base.mime_type,
            blob: b.blob,
            meta: None,
        },
    }
}
