//! Developer message types.

use super::simple_content::{SimpleContent, SimpleContentExpression};
use crate::functions;
use functions::expression::{
    ExpressionError, FromStarlarkValue, WithExpression,
};
use serde::{Deserialize, Serialize};
use starlark::values::dict::DictRef as StarlarkDictRef;
use starlark::values::{UnpackValue, Value as StarlarkValue};

/// A developer message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperMessage {
    /// The message content.
    pub content: SimpleContent,
    /// Optional name for the message author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

impl DeveloperMessage {
    pub fn push(&mut self, other: &DeveloperMessage) {
        self.content.push(&other.content);
        if let Some(other_name) = &other.name {
            match &mut self.name {
                Some(self_name) => self_name.push_str(other_name),
                None => self.name = Some(other_name.clone()),
            }
        }
    }

    pub fn has_name(&self) -> bool {
        self.name.as_ref().is_some_and(|n| !n.is_empty())
    }

    /// Prepares the message by normalizing content and optional fields.
    pub fn prepare(&mut self) {
        self.content.prepare();
        if self.name.as_ref().is_some_and(String::is_empty) {
            self.name = None;
        }
    }
}

impl FromStarlarkValue for DeveloperMessage {
    fn from_starlark_value(
        value: &StarlarkValue,
    ) -> Result<Self, ExpressionError> {
        let dict = StarlarkDictRef::from_value(*value).ok_or_else(|| {
            ExpressionError::StarlarkConversionError(
                "DeveloperMessage: expected dict".into(),
            )
        })?;
        let mut content = None;
        let mut name = None;
        for (k, v) in dict.iter() {
            let key = <&str as UnpackValue>::unpack_value(k)
                .map_err(|e| {
                    ExpressionError::StarlarkConversionError(e.to_string())
                })?
                .ok_or_else(|| {
                    ExpressionError::StarlarkConversionError(
                        "DeveloperMessage: expected string key".into(),
                    )
                })?;
            match key {
                "content" => {
                    content = Some(SimpleContent::from_starlark_value(&v)?)
                }
                "name" => name = Option::<String>::from_starlark_value(&v)?,
                _ => {}
            }
        }
        Ok(DeveloperMessage {
            content: content.ok_or_else(|| {
                ExpressionError::StarlarkConversionError(
                    "DeveloperMessage: missing content".into(),
                )
            })?,
            name,
        })
    }
}

/// Expression variant of [`DeveloperMessage`] for dynamic content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeveloperMessageExpression {
    /// The message content expression.
    pub content: functions::expression::WithExpression<SimpleContentExpression>,
    /// Optional name expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<functions::expression::WithExpression<Option<String>>>,
}

impl DeveloperMessageExpression {
    /// Compiles the expression into a concrete [`DeveloperMessage`].
    pub fn compile(
        self,
        params: &functions::expression::Params,
    ) -> Result<DeveloperMessage, functions::expression::ExpressionError> {
        let content = self.content.compile_one(params)?.compile(params)?;
        let name = self
            .name
            .map(|name| name.compile_one(params))
            .transpose()?
            .flatten();
        Ok(DeveloperMessage { content, name })
    }
}

impl FromStarlarkValue for DeveloperMessageExpression {
    fn from_starlark_value(
        value: &StarlarkValue,
    ) -> Result<Self, ExpressionError> {
        let dict = StarlarkDictRef::from_value(*value).ok_or_else(|| {
            ExpressionError::StarlarkConversionError(
                "DeveloperMessageExpression: expected dict".into(),
            )
        })?;
        let mut content = None;
        let mut name = None;
        for (k, v) in dict.iter() {
            let key = <&str as UnpackValue>::unpack_value(k)
                .map_err(|e| {
                    ExpressionError::StarlarkConversionError(e.to_string())
                })?
                .ok_or_else(|| {
                    ExpressionError::StarlarkConversionError(
                        "DeveloperMessageExpression: expected string key"
                            .into(),
                    )
                })?;
            match key {
                "content" => {
                    content = Some(WithExpression::Value(
                        SimpleContentExpression::from_starlark_value(&v)?,
                    ))
                }
                "name" => {
                    name = Some(WithExpression::Value(if v.is_none() {
                        None
                    } else {
                        Some(String::from_starlark_value(&v)?)
                    }));
                }
                _ => {}
            }
        }
        Ok(DeveloperMessageExpression {
            content: content.ok_or_else(|| {
                ExpressionError::StarlarkConversionError(
                    "DeveloperMessageExpression: missing content".into(),
                )
            })?,
            name,
        })
    }
}
