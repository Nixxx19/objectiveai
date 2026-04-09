use crate::agent;
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.inventions.recursive.response.streaming.FunctionInventionRecursiveChunk")]
pub struct FunctionInventionRecursiveChunk {
    pub id: String,
    pub inventions: Vec<super::FunctionInventionChunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub inventions_errors: Option<bool>,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub usage: Option<agent::completions::response::Usage>,
}

impl FunctionInventionRecursiveChunk {
    pub fn push(
        &mut self,
        FunctionInventionRecursiveChunk {
            inventions,
            inventions_errors,
            usage,
            ..
        }: &FunctionInventionRecursiveChunk,
    ) {
        self.push_inventions(inventions);
        if let Some(true) = inventions_errors {
            self.inventions_errors = Some(true);
        }
        match (&mut self.usage, usage) {
            (Some(self_usage), Some(other_usage)) => {
                self_usage.push(other_usage);
            }
            (None, Some(other_usage)) => {
                self.usage = Some(other_usage.clone());
            }
            _ => {}
        }
    }

    fn push_inventions(
        &mut self,
        other_inventions: &[super::FunctionInventionChunk],
    ) {
        fn push_invention(
            inventions: &mut Vec<super::FunctionInventionChunk>,
            other: &super::FunctionInventionChunk,
        ) {
            fn find_invention(
                inventions: &mut Vec<super::FunctionInventionChunk>,
                index: u64,
            ) -> Option<&mut super::FunctionInventionChunk> {
                for invention in inventions {
                    if invention.index == index {
                        return Some(invention);
                    }
                }
                None
            }
            if let Some(existing) = find_invention(inventions, other.index) {
                existing.push(other);
            } else {
                inventions.push(other.clone());
            }
        }
        for other in other_inventions {
            push_invention(&mut self.inventions, other);
        }
    }

    /// Produces the `(path, file_bytes)` pairs for the log file structure.
    ///
    /// Returns `(reference, files)`. All paths relative to `logs/`.
    #[cfg(feature = "filesystem")]
    pub fn produce_files(&self) -> Option<(serde_json::Value, Vec<(String, Vec<u8>)>)> {
        const PREFIX: &str = "functions/inventions/recursive/";

        let id = &self.id;
        if id.is_empty() {
            return None;
        }

        let path = format!("{PREFIX}{id}.json");
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut invention_refs: Vec<serde_json::Value> = Vec::new();

        for invention in &self.inventions {
            let (reference, invention_files) = invention.produce_files();
            invention_refs.push(reference);
            files.extend(invention_files);
        }

        // Serialize a shell without inventions to avoid double-serialization
        let shell = FunctionInventionRecursiveChunk {
            id: self.id.clone(),
            inventions: Vec::new(),
            inventions_errors: self.inventions_errors,
            created: self.created,
            object: self.object,
            usage: self.usage.clone(),
        };
        let mut root = serde_json::to_value(&shell).unwrap();
        root["inventions"] = serde_json::Value::Array(invention_refs);
        files.push((path.clone(), serde_json::to_vec_pretty(&root).unwrap()));

        Some((serde_json::json!({ "type": "reference", "path": path }), files))
    }
}
