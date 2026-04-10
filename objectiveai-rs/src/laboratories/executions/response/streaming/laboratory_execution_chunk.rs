use crate::{agent, error};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Streaming chunk for a laboratory execution.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "laboratories.executions.response.streaming.LaboratoryExecutionChunk")]
pub struct LaboratoryExecutionChunk {
    pub id: String,
    pub builders: Vec<super::BuilderChunk>,
    pub evaluations: Vec<super::EvaluationChunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub error: Option<error::ResponseError>,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub usage: Option<agent::completions::response::Usage>,
}

impl LaboratoryExecutionChunk {
    pub fn push(
        &mut self,
        LaboratoryExecutionChunk {
            builders,
            evaluations,
            error,
            usage,
            ..
        }: &LaboratoryExecutionChunk,
    ) {
        self.push_builders(builders);
        self.push_evaluations(evaluations);
        if let Some(error) = error {
            self.error = Some(error.clone());
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

    fn push_builders(&mut self, others: &[super::BuilderChunk]) {
        for other in others {
            if let Some(existing) = self.builders.iter_mut().find(|c| c.index == other.index) {
                existing.push(other);
            } else {
                self.builders.push(other.clone());
            }
        }
    }

    fn push_evaluations(&mut self, others: &[super::EvaluationChunk]) {
        for other in others {
            if let Some(existing) = self
                .evaluations
                .iter_mut()
                .find(|c| c.index == other.index)
            {
                existing.push(other);
            } else {
                self.evaluations.push(other.clone());
            }
        }
    }

    /// Produces the `(path, file_bytes)` pairs for the log file structure.
    ///
    /// Returns `(reference, files)`. All paths relative to `logs/`.
    #[cfg(feature = "filesystem")]
    pub fn produce_files(&self) -> Option<(serde_json::Value, Vec<(String, Vec<u8>)>)> {
        const PREFIX: &str = "laboratories/executions/";

        let id = &self.id;
        if id.is_empty() {
            return None;
        }

        let path = format!("{PREFIX}{id}.json");
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut builder_refs: Vec<serde_json::Value> = Vec::new();
        let mut evaluation_refs: Vec<serde_json::Value> = Vec::new();

        for builder in &self.builders {
            let (reference, builder_files) = builder.produce_files();
            builder_refs.push(reference);
            files.extend(builder_files);
        }

        for evaluation in &self.evaluations {
            let (reference, eval_files) = evaluation.produce_files();
            evaluation_refs.push(reference);
            files.extend(eval_files);
        }

        // Serialize a shell without builders/evaluations to avoid double-serialization
        let shell = LaboratoryExecutionChunk {
            id: self.id.clone(),
            builders: Vec::new(),
            evaluations: Vec::new(),
            error: self.error.clone(),
            created: self.created,
            object: self.object,
            usage: self.usage.clone(),
        };
        let mut root = serde_json::to_value(&shell).unwrap();
        root["builders"] = serde_json::Value::Array(builder_refs);
        root["evaluations"] = serde_json::Value::Array(evaluation_refs);
        files.push((path.clone(), serde_json::to_vec_pretty(&root).unwrap()));

        Some((serde_json::json!({ "type": "reference", "path": path }), files))
    }
}
