use indexmap::IndexMap;
use rand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum JsonSchema {
    String(StringJsonSchema),
    Number(NumberJsonSchema),
    Integer(IntegerJsonSchema),
    Boolean(BooleanJsonSchema),
    Array(ArrayJsonSchema),
    Object(ObjectJsonSchema),
}

impl JsonSchema {
    pub fn generate(&self) -> serde_json::Value {
        self.generate_from_rng(&mut rand::rng())
    }

    pub fn generate_from_rng(&self, rng: &mut impl Rng) -> serde_json::Value {
        match self {
            JsonSchema::String(s) => s.generate_from_rng(rng),
            JsonSchema::Number(n) => n.generate_from_rng(rng),
            JsonSchema::Integer(i) => i.generate_from_rng(rng),
            JsonSchema::Boolean(b) => b.generate_from_rng(rng),
            JsonSchema::Array(a) => a.generate_from_rng(rng),
            JsonSchema::Object(o) => o.generate_from_rng(rng),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StringJsonSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#enum: Option<Vec<String>>,
}

impl StringJsonSchema {
    pub fn generate(&self) -> serde_json::Value {
        self.generate_from_rng(&mut rand::rng())
    }

    pub fn generate_from_rng(&self, rng: &mut impl Rng) -> serde_json::Value {
        let s = match &self.r#enum {
            Some(variants) if !variants.is_empty() => {
                variants[rng.gen_range(0..variants.len())].clone()
            }
            Some(_) => String::new(),
            None => {
                let len = rng.gen_range(1..=32);
                (0..len)
                    .map(|_| {
                        const CHARS: &[u8] =
                            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
                        CHARS[rng.gen_range(0..CHARS.len())] as char
                    })
                    .collect()
            }
        };
        serde_json::Value::String(s)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NumberJsonSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
}

impl NumberJsonSchema {
    pub fn generate(&self) -> serde_json::Value {
        self.generate_from_rng(&mut rand::rng())
    }

    pub fn generate_from_rng(&self, rng: &mut impl Rng) -> serde_json::Value {
        let min = self.minimum.unwrap_or(0.0);
        let max = self.maximum.unwrap_or(100.0);
        serde_json::Value::Number(
            serde_json::Number::from_f64(rng.gen_range(min..=max)).unwrap_or_else(|| {
                serde_json::Number::from_f64(0.0).unwrap()
            }),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct IntegerJsonSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<i64>,
}

impl IntegerJsonSchema {
    pub fn generate(&self) -> serde_json::Value {
        self.generate_from_rng(&mut rand::rng())
    }

    pub fn generate_from_rng(&self, rng: &mut impl Rng) -> serde_json::Value {
        let min = self.minimum.unwrap_or(0);
        let max = self.maximum.unwrap_or(100);
        serde_json::json!(rng.gen_range(min..=max))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BooleanJsonSchema;

impl BooleanJsonSchema {
    pub fn generate(&self) -> serde_json::Value {
        self.generate_from_rng(&mut rand::rng())
    }

    pub fn generate_from_rng(&self, rng: &mut impl Rng) -> serde_json::Value {
        serde_json::Value::Bool(rng.gen_bool(0.5))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArrayJsonSchema {
    pub items: Box<JsonSchema>,
    #[serde(rename = "minItems", skip_serializing_if = "Option::is_none")]
    pub min_items: Option<u64>,
    #[serde(rename = "maxItems", skip_serializing_if = "Option::is_none")]
    pub max_items: Option<u64>,
}

impl ArrayJsonSchema {
    pub fn generate(&self) -> serde_json::Value {
        self.generate_from_rng(&mut rand::rng())
    }

    pub fn generate_from_rng(&self, rng: &mut impl Rng) -> serde_json::Value {
        let min = self.min_items.unwrap_or(1).max(1) as usize;
        let max = self.max_items.unwrap_or(10).max(min as u64) as usize;
        let len = rng.gen_range(min..=max);
        let items: Vec<serde_json::Value> =
            (0..len).map(|_| self.items.generate_from_rng(rng)).collect();
        serde_json::Value::Array(items)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObjectJsonSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<IndexMap<String, JsonSchema>>,
}

impl ObjectJsonSchema {
    pub fn generate(&self) -> serde_json::Value {
        self.generate_from_rng(&mut rand::rng())
    }

    pub fn generate_from_rng(&self, rng: &mut impl Rng) -> serde_json::Value {
        let map = match &self.properties {
            Some(props) => props
                .iter()
                .map(|(k, v)| (k.clone(), v.generate_from_rng(rng)))
                .collect(),
            None => serde_json::Map::new(),
        };
        serde_json::Value::Object(map)
    }
}

#[cfg(test)]
#[path = "json_schema_tests.rs"]
mod tests;
