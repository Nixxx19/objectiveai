use crate::functions;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

pub type ScalarFunctionInputSchema = functions::expression::ObjectInputSchema;

pub mod scalar_function_input_schema {
    use crate::functions;
    pub fn transpile(
        this: super::ScalarFunctionInputSchema,
    ) -> functions::expression::InputSchema {
        functions::expression::InputSchema::Object(this)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorFunctionInputSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<functions::expression::ObjectInputSchema>,
    pub items: functions::expression::InputSchema,
}

impl VectorFunctionInputSchema {
    /// Returns which media modalities are present in context and/or items.
    pub fn modalities(&self) -> functions::expression::Modalities {
        let ctx = self.context.as_ref()
            .map(|c| c.modalities())
            .unwrap_or_default();
        ctx.merge(self.items.modalities())
    }

    pub fn transpile(self) -> functions::expression::InputSchema {
        functions::expression::InputSchema::Object(
            functions::expression::ObjectInputSchema {
                description: None,
                required: Some(if self.context.is_some() {
                    vec!["context".to_string(), "items".to_string()]
                } else {
                    vec!["items".to_string()]
                }),
                properties: {
                    let mut map =
                        IndexMap::with_capacity(if self.context.is_some() {
                            2
                        } else {
                            1
                        });
                    if let Some(context) = self.context {
                        map.insert(
                            "context".to_string(),
                            functions::expression::InputSchema::Object(context),
                        );
                    }
                    map.insert("items".to_string(), self.items);
                    map
                },
            },
        )
    }
}

pub type ScalarFunctionInputExpression = functions::expression::Expression;

pub mod scalar_function_input_expression {
    use crate::functions;
    pub fn transpile(
        this: super::ScalarFunctionInputExpression,
    ) -> functions::expression::WithExpression<
        functions::expression::InputExpression,
    > {
        functions::expression::WithExpression::Expression(this)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorFunctionInputExpression {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<functions::expression::Expression>,
    pub items: functions::expression::Expression,
}

impl VectorFunctionInputExpression {
    pub fn transpile(
        self,
    ) -> functions::expression::WithExpression<
        functions::expression::InputExpression,
    > {
        functions::expression::WithExpression::Value(
            functions::expression::InputExpression::Object({
                let mut map =
                    IndexMap::with_capacity(if self.context.is_some() {
                        2
                    } else {
                        1
                    });
                if let Some(context) = self.context {
                    map.insert(
                        "context".to_string(),
                        functions::expression::WithExpression::Expression(
                            context,
                        ),
                    );
                }
                map.insert(
                    "items".to_string(),
                    functions::expression::WithExpression::Expression(
                        self.items,
                    ),
                );
                map
            }),
        )
    }
}

pub type ScalarFunctionInput = IndexMap<String, functions::expression::Input>;

pub mod scalar_function_input {
    use crate::functions;
    pub fn transpile(
        this: super::ScalarFunctionInput,
    ) -> functions::expression::Input {
        functions::expression::Input::Object(this)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorFunctionInput {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<IndexMap<String, functions::expression::Input>>,
    pub items: Vec<functions::expression::Input>,
}

impl VectorFunctionInput {
    pub fn transpile(self) -> functions::expression::Input {
        functions::expression::Input::Object({
            let mut map = IndexMap::with_capacity(if self.context.is_some() {
                2
            } else {
                1
            });
            if let Some(context) = self.context {
                map.insert(
                    "context".to_string(),
                    functions::expression::Input::Object(context),
                );
            }
            map.insert(
                "items".to_string(),
                functions::expression::Input::Array(self.items),
            );
            map
        })
    }
}
