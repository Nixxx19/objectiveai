//! Schema types for validating Function input.
//!
//! Defines the expected structure and constraints for input data.
//! Used by remote Functions to document and validate their inputs.

use crate::agent;
use indexmap::IndexMap;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use schemars::JsonSchema;
use super::InputValue;

/// Schema for validating Function input.
///
/// Defines the expected structure and constraints for input data.
/// Used by remote Functions to document and validate their inputs.
#[derive(Debug, Clone, PartialEq, JsonSchema)]
#[schemars(rename = "functions.expression.InputSchema")]
pub enum InputSchema {
    /// An object with named properties.
    Object(ObjectInputSchema),
    /// An array of items.
    Array(ArrayInputSchema),
    /// A string value.
    String(StringInputSchema),
    /// An integer value.
    Integer(IntegerInputSchema),
    /// A floating-point number.
    Number(NumberInputSchema),
    /// A boolean value.
    Boolean(BooleanInputSchema),
    /// An image (URL or base64).
    Image(ImageInputSchema),
    /// Audio content.
    Audio(AudioInputSchema),
    /// Video content.
    Video(VideoInputSchema),
    /// A file.
    File(FileInputSchema),
    /// A union of schemas - input must match at least one.
    AnyOf(AnyOfInputSchema),
}

impl InputSchema {
    /// Returns which media modalities are present anywhere in this schema.
    pub fn modalities(&self) -> Modalities {
        match self {
            InputSchema::Image(_) => Modalities { image: true, ..Modalities::default() },
            InputSchema::Audio(_) => Modalities { audio: true, ..Modalities::default() },
            InputSchema::Video(_) => Modalities { video: true, ..Modalities::default() },
            InputSchema::File(_) => Modalities { file: true, ..Modalities::default() },
            InputSchema::Object(s) => s.modalities(),
            InputSchema::Array(s) => s.modalities(),
            InputSchema::AnyOf(s) => s.modalities(),
            InputSchema::String(_) | InputSchema::Integer(_)
            | InputSchema::Number(_) | InputSchema::Boolean(_) => Modalities::default(),
        }
    }

    /// Validates that an input value conforms to this schema.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match self {
            InputSchema::Object(schema) => schema.validate_input(input),
            InputSchema::Array(schema) => schema.validate_input(input),
            InputSchema::String(schema) => schema.validate_input(input),
            InputSchema::Integer(schema) => schema.validate_input(input),
            InputSchema::Number(schema) => schema.validate_input(input),
            InputSchema::Boolean(schema) => schema.validate_input(input),
            InputSchema::Image(schema) => schema.validate_input(input),
            InputSchema::Audio(schema) => schema.validate_input(input),
            InputSchema::Video(schema) => schema.validate_input(input),
            InputSchema::File(schema) => schema.validate_input(input),
            InputSchema::AnyOf(schema) => schema.validate_input(input),
        }
    }
}

/// Helper enum for deserializing typed schemas (those with a `type` field).
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum TypedInputSchema {
    Object(ObjectInputSchema),
    Array(ArrayInputSchema),
    String(StringInputSchema),
    Integer(IntegerInputSchema),
    Number(NumberInputSchema),
    Boolean(BooleanInputSchema),
    Image(ImageInputSchema),
    Audio(AudioInputSchema),
    Video(VideoInputSchema),
    File(FileInputSchema),
}

impl From<TypedInputSchema> for InputSchema {
    fn from(typed: TypedInputSchema) -> Self {
        match typed {
            TypedInputSchema::Object(s) => InputSchema::Object(s),
            TypedInputSchema::Array(s) => InputSchema::Array(s),
            TypedInputSchema::String(s) => InputSchema::String(s),
            TypedInputSchema::Integer(s) => InputSchema::Integer(s),
            TypedInputSchema::Number(s) => InputSchema::Number(s),
            TypedInputSchema::Boolean(s) => InputSchema::Boolean(s),
            TypedInputSchema::Image(s) => InputSchema::Image(s),
            TypedInputSchema::Audio(s) => InputSchema::Audio(s),
            TypedInputSchema::Video(s) => InputSchema::Video(s),
            TypedInputSchema::File(s) => InputSchema::File(s),
        }
    }
}

impl<'de> Deserialize<'de> for InputSchema {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;

        // Check if this is an AnyOf schema (has anyOf field, no type field)
        if value.get("anyOf").is_some() {
            let schema: AnyOfInputSchema =
                serde_json::from_value(value).map_err(D::Error::custom)?;
            Ok(InputSchema::AnyOf(schema))
        } else {
            // Deserialize as a typed schema
            let typed: TypedInputSchema =
                serde_json::from_value(value).map_err(D::Error::custom)?;
            Ok(typed.into())
        }
    }
}

impl Serialize for InputSchema {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            InputSchema::AnyOf(schema) => schema.serialize(serializer),
            InputSchema::Object(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a ObjectInputSchema,
                }
                Tagged {
                    r#type: "object",
                    schema,
                }
                .serialize(serializer)
            }
            InputSchema::Array(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a ArrayInputSchema,
                }
                Tagged {
                    r#type: "array",
                    schema,
                }
                .serialize(serializer)
            }
            InputSchema::String(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a StringInputSchema,
                }
                Tagged {
                    r#type: "string",
                    schema,
                }
                .serialize(serializer)
            }
            InputSchema::Integer(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a IntegerInputSchema,
                }
                Tagged {
                    r#type: "integer",
                    schema,
                }
                .serialize(serializer)
            }
            InputSchema::Number(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a NumberInputSchema,
                }
                Tagged {
                    r#type: "number",
                    schema,
                }
                .serialize(serializer)
            }
            InputSchema::Boolean(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a BooleanInputSchema,
                }
                Tagged {
                    r#type: "boolean",
                    schema,
                }
                .serialize(serializer)
            }
            InputSchema::Image(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a ImageInputSchema,
                }
                Tagged {
                    r#type: "image",
                    schema,
                }
                .serialize(serializer)
            }
            InputSchema::Audio(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a AudioInputSchema,
                }
                Tagged {
                    r#type: "audio",
                    schema,
                }
                .serialize(serializer)
            }
            InputSchema::Video(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a VideoInputSchema,
                }
                Tagged {
                    r#type: "video",
                    schema,
                }
                .serialize(serializer)
            }
            InputSchema::File(schema) => {
                #[derive(Serialize)]
                #[serde(rename_all = "camelCase")]
                struct Tagged<'a> {
                    r#type: &'static str,
                    #[serde(flatten)]
                    schema: &'a FileInputSchema,
                }
                Tagged {
                    r#type: "file",
                    schema,
                }
                .serialize(serializer)
            }
        }
    }
}

/// Which media modalities are present in a schema.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modalities {
    pub image: bool,
    pub audio: bool,
    pub video: bool,
    pub file: bool,
}

impl Modalities {
    /// Merge two `Modalities` (union).
    pub fn merge(self, other: Self) -> Self {
        Self {
            image: self.image || other.image,
            audio: self.audio || other.audio,
            video: self.video || other.video,
            file: self.file || other.file,
        }
    }
}

/// Schema for a union of possible types - input must match at least one.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.AnyOfInputSchema")]
pub struct AnyOfInputSchema {
    /// The possible schemas that the input can match.
    pub any_of: Vec<InputSchema>,
}

impl AnyOfInputSchema {
    /// Returns which media modalities are present in any variant.
    pub fn modalities(&self) -> Modalities {
        self.any_of.iter().fold(Modalities::default(), |acc, s| acc.merge(s.modalities()))
    }

    /// Validates that an input matches at least one schema in the union.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        self.any_of
            .iter()
            .any(|schema| schema.validate_input(input))
    }
}

/// Schema for an object input with named properties.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.ObjectInputSchema")]
pub struct ObjectInputSchema {
    /// Human-readable description of the object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Schema for each property in the object.
    pub properties: IndexMap<String, InputSchema>,
    /// List of property names that must be present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl ObjectInputSchema {
    /// Returns which media modalities are present in any property.
    pub fn modalities(&self) -> Modalities {
        self.properties.values().fold(Modalities::default(), |acc, s| acc.merge(s.modalities()))
    }

    /// Validates that an input is an object matching this schema.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::Object(map) => {
                let required = self.required.as_deref().unwrap_or(&[]);
                self.properties.iter().all(|(key, schema)| {
                    match map.get(key) {
                        Some(value) => schema.validate_input(value),
                        None => !required.contains(key),
                    }
                })
            }
            _ => false,
        }
    }
}

/// Schema for an array input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.ArrayInputSchema")]
pub struct ArrayInputSchema {
    /// Human-readable description of the array.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum number of items required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,
    /// Maximum number of items allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    /// Schema for each item in the array.
    pub items: Box<InputSchema>,
}

impl ArrayInputSchema {
    /// Returns which media modalities are present in the item schema.
    pub fn modalities(&self) -> Modalities {
        self.items.modalities()
    }

    /// Validates that an input is an array matching this schema.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::Array(array) => {
                if let Some(min_items) = self.min_items
                    && (array.len() as u64) < min_items
                {
                    false
                } else if let Some(max_items) = self.max_items
                    && (array.len() as u64) > max_items
                {
                    false
                } else {
                    array.iter().all(|item| self.items.validate_input(item))
                }
            }
            _ => false,
        }
    }
}

/// Schema for a string input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.StringInputSchema")]
pub struct StringInputSchema {
    /// Human-readable description of the string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// If provided, the string must be one of these values.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
}

impl StringInputSchema {
    /// Validates that an input is a string matching this schema.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::String(s) => {
                if let Some(r#enum) = &self.r#enum {
                    r#enum.contains(s)
                } else {
                    true
                }
            }
            _ => false,
        }
    }
}

/// Schema for an integer input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.IntegerInputSchema")]
pub struct IntegerInputSchema {
    /// Human-readable description of the integer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum allowed value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,
    /// Maximum allowed value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
}

impl IntegerInputSchema {
    /// Validates that an input is an integer matching this schema.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::Integer(integer) => {
                if let Some(minimum) = self.minimum
                    && *integer < minimum
                {
                    false
                } else if let Some(maximum) = self.maximum
                    && *integer > maximum
                {
                    false
                } else {
                    true
                }
            }
            InputValue::Number(number)
                if number.is_finite() && number.fract() == 0.0 =>
            {
                let integer = *number as i64;
                if let Some(minimum) = self.minimum
                    && integer < minimum
                {
                    false
                } else if let Some(maximum) = self.maximum
                    && integer > maximum
                {
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }
}

/// Schema for a floating-point number input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.NumberInputSchema")]
pub struct NumberInputSchema {
    /// Human-readable description of the number.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Minimum allowed value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    /// Maximum allowed value (inclusive).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
}

impl NumberInputSchema {
    /// Validates that an input is a number matching this schema.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::Integer(integer) => {
                let number = *integer as f64;
                if let Some(minimum) = self.minimum
                    && number < minimum
                {
                    false
                } else if let Some(maximum) = self.maximum
                    && number > maximum
                {
                    false
                } else {
                    true
                }
            }
            InputValue::Number(number) => {
                if let Some(minimum) = self.minimum
                    && *number < minimum
                {
                    false
                } else if let Some(maximum) = self.maximum
                    && *number > maximum
                {
                    false
                } else {
                    true
                }
            }
            _ => false,
        }
    }
}

/// Schema for a boolean input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.BooleanInputSchema")]
pub struct BooleanInputSchema {
    /// Human-readable description of the boolean.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl BooleanInputSchema {
    /// Validates that an input is a boolean.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::Boolean(_) => true,
            _ => false,
        }
    }
}

/// Schema for an image input (URL or base64-encoded).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.ImageInputSchema")]
pub struct ImageInputSchema {
    /// Human-readable description of the expected image.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ImageInputSchema {
    /// Validates that an input is an image.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::RichContentPart(
                agent::completions::message::RichContentPart::ImageUrl {
                    ..
                },
            ) => true,
            _ => false,
        }
    }
}

/// Schema for an audio input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.AudioInputSchema")]
pub struct AudioInputSchema {
    /// Human-readable description of the expected audio.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl AudioInputSchema {
    /// Validates that an input is audio content.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::RichContentPart(
                agent::completions::message::RichContentPart::InputAudio {
                    ..
                },
            ) => true,
            _ => false,
        }
    }
}

/// Schema for a video input (URL or base64-encoded).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.VideoInputSchema")]
pub struct VideoInputSchema {
    /// Human-readable description of the expected video.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl VideoInputSchema {
    /// Validates that an input is video content.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::RichContentPart(
                agent::completions::message::RichContentPart::InputVideo {
                    ..
                },
            ) => true,
            InputValue::RichContentPart(
                agent::completions::message::RichContentPart::VideoUrl {
                    ..
                },
            ) => true,
            _ => false,
        }
    }
}

/// Schema for a file input.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
#[schemars(rename = "functions.expression.FileInputSchema")]
pub struct FileInputSchema {
    /// Human-readable description of the expected file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl FileInputSchema {
    /// Validates that an input is a file.
    pub fn validate_input(&self, input: &InputValue) -> bool {
        match input {
            InputValue::RichContentPart(
                agent::completions::message::RichContentPart::File { .. },
            ) => true,
            _ => false,
        }
    }
}
