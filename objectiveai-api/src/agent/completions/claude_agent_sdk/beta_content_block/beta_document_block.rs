use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaDocumentBlockType {
    Document,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaCitationConfig {
    pub enabled: bool,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaBase64PDFSourceMediaType {
    #[serde(rename = "application/pdf")]
    ApplicationPdf,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaBase64PDFSourceType {
    Base64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaBase64PDFSource {
    pub data: String,
    pub media_type: BetaBase64PDFSourceMediaType,
    pub r#type: BetaBase64PDFSourceType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaPlainTextSourceMediaType {
    #[serde(rename = "text/plain")]
    TextPlain,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BetaPlainTextSourceType {
    Text,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaPlainTextSource {
    pub data: String,
    pub media_type: BetaPlainTextSourceMediaType,
    pub r#type: BetaPlainTextSourceType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum BetaDocumentSource {
    Base64PDF(BetaBase64PDFSource),
    PlainText(BetaPlainTextSource),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct BetaDocumentBlock {
    pub citations: Option<BetaCitationConfig>,
    pub source: BetaDocumentSource,
    pub title: Option<String>,
    pub r#type: BetaDocumentBlockType,
}
