use crate::chat;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputMaps {
    One(super::Expression),
    Many(Vec<super::Expression>),
}

impl InputMaps {
    pub fn compile(
        self,
        params: &super::Params,
    ) -> Result<Vec<Vec<Input>>, super::ExpressionError> {
        match self {
            InputMaps::One(expression) => {
                match expression.compile_one_or_many::<Vec<Input>>(params)? {
                    super::OneOrMany::One(one) => Ok(vec![one]),
                    super::OneOrMany::Many(many) => Ok(many),
                }
            }
            InputMaps::Many(expressions) => {
                let mut compiled = Vec::with_capacity(expressions.len());
                for expression in expressions {
                    match expression
                        .compile_one_or_many::<Vec<Input>>(params)?
                    {
                        super::OneOrMany::One(one) => compiled.push(one),
                        super::OneOrMany::Many(many) => {
                            for item in many {
                                compiled.push(item);
                            }
                        }
                    }
                }
                Ok(compiled)
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Input {
    RichContentPart(chat::completions::request::RichContentPart),
    Object(IndexMap<String, Input>),
    Array(Vec<Input>),
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputExpression {
    RichContentPart(chat::completions::request::RichContentPart),
    Object(IndexMap<String, super::WithExpression<InputExpression>>),
    Array(Vec<super::WithExpression<InputExpression>>),
    String(String),
    Integer(i64),
    Number(f64),
    Boolean(bool),
}

impl InputExpression {
    pub fn compile(
        self,
        params: &super::Params,
    ) -> Result<Input, super::ExpressionError> {
        match self {
            InputExpression::RichContentPart(rich_content_part) => {
                Ok(Input::RichContentPart(rich_content_part))
            }
            InputExpression::Object(object) => {
                let mut compiled_object = IndexMap::with_capacity(object.len());
                for (key, value) in object {
                    compiled_object.insert(
                        key,
                        value.compile_one(params)?.compile(params)?,
                    );
                }
                Ok(Input::Object(compiled_object))
            }
            InputExpression::Array(array) => {
                let mut compiled_array = Vec::with_capacity(array.len());
                for item in array {
                    match item.compile_one_or_many(params)? {
                        super::OneOrMany::One(one_item) => {
                            compiled_array.push(one_item.compile(params)?);
                        }
                        super::OneOrMany::Many(many_items) => {
                            for item in many_items {
                                compiled_array.push(item.compile(params)?);
                            }
                        }
                    }
                }
                Ok(Input::Array(compiled_array))
            }
            InputExpression::String(string) => Ok(Input::String(string)),
            InputExpression::Integer(integer) => Ok(Input::Integer(integer)),
            InputExpression::Number(number) => Ok(Input::Number(number)),
            InputExpression::Boolean(boolean) => Ok(Input::Boolean(boolean)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum InputSchema {
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

impl InputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
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
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ObjectInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub properties: IndexMap<String, InputSchema>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<Vec<String>>,
}

impl ObjectInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::Object(map) => {
                self.properties.iter().all(|(key, schema)| {
                    map.get(key)
                        .map(|value| schema.validate_input(value))
                        .unwrap_or(false)
                }) && {
                    if let Some(required) = &self.required {
                        required.iter().all(|key| map.contains_key(key))
                    } else {
                        true
                    }
                }
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArrayInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
    pub items: Box<InputSchema>,
}

impl ArrayInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::Array(array) => {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StringInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
}

impl StringInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::String(s) => {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IntegerInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
}

impl IntegerInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::Integer(integer) => {
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
            Input::Number(number)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NumberInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
}

impl NumberInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::Integer(integer) => {
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
            Input::Number(number) => {
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BooleanInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl BooleanInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::Boolean(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl ImageInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::RichContentPart(
                chat::completions::request::RichContentPart::ImageUrl {
                    ..
                },
            ) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AudioInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl AudioInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::RichContentPart(
                chat::completions::request::RichContentPart::InputAudio {
                    ..
                },
            ) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl VideoInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::RichContentPart(
                chat::completions::request::RichContentPart::InputVideo {
                    ..
                },
            ) => true,
            Input::RichContentPart(
                chat::completions::request::RichContentPart::VideoUrl {
                    ..
                },
            ) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl FileInputSchema {
    pub fn validate_input(&self, input: &Input) -> bool {
        match input {
            Input::RichContentPart(
                chat::completions::request::RichContentPart::File { .. },
            ) => true,
            _ => false,
        }
    }
}
