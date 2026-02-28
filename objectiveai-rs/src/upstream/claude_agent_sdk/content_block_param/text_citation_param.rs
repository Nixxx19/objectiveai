use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CitationCharLocationParamType {
    CharLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CitationCharLocationParam {
    pub cited_text: String,
    pub document_index: f64,
    pub document_title: Option<String>,
    pub end_char_index: f64,
    pub start_char_index: f64,
    pub r#type: CitationCharLocationParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CitationPageLocationParamType {
    PageLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CitationPageLocationParam {
    pub cited_text: String,
    pub document_index: f64,
    pub document_title: Option<String>,
    pub end_page_number: f64,
    pub start_page_number: f64,
    pub r#type: CitationPageLocationParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CitationContentBlockLocationParamType {
    ContentBlockLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CitationContentBlockLocationParam {
    pub cited_text: String,
    pub document_index: f64,
    pub document_title: Option<String>,
    pub end_block_index: f64,
    pub start_block_index: f64,
    pub r#type: CitationContentBlockLocationParamType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CitationWebSearchResultLocationParamType {
    WebSearchResultLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CitationWebSearchResultLocationParam {
    pub cited_text: String,
    pub encrypted_index: String,
    pub title: Option<String>,
    pub r#type: CitationWebSearchResultLocationParamType,
    pub url: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CitationSearchResultLocationParamType {
    SearchResultLocation,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CitationSearchResultLocationParam {
    pub cited_text: String,
    pub end_block_index: f64,
    pub search_result_index: f64,
    pub source: String,
    pub start_block_index: f64,
    pub title: Option<String>,
    pub r#type: CitationSearchResultLocationParamType,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum TextCitationParam {
    CharLocation(CitationCharLocationParam),
    PageLocation(CitationPageLocationParam),
    ContentBlockLocation(CitationContentBlockLocationParam),
    WebSearchResultLocation(CitationWebSearchResultLocationParam),
    SearchResultLocation(CitationSearchResultLocationParam),
}
