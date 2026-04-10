use crate::{agent, error, functions};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.inventions.response.streaming.FunctionInventionChunk")]
pub struct FunctionInventionChunk {
    pub id: String,
    pub completions: Vec<super::AgentCompletionChunk>,
    // yielded after steps with the current state
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub state: Option<functions::inventions::State>,
    // yielded at the end
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub path: Option<crate::RemotePath>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub function: Option<functions::FullRemoteFunction>,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub created: u64,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub usage: Option<agent::completions::response::Usage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub error: Option<error::ResponseError>,
}

impl FunctionInventionChunk {
    pub fn push(
        &mut self,
        FunctionInventionChunk {
            completions,
            state,
            path,
            function,
            usage,
            error,
            ..
        }: &FunctionInventionChunk,
    ) {
        self.push_completions(completions);
        if let Some(state) = state {
            self.state = Some(state.clone());
        }
        if let Some(path) = path {
            self.path = Some(path.clone());
        }
        if let Some(function) = function {
            self.function = Some(function.clone());
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
        if let Some(error) = error {
            self.error = Some(error.clone());
        }
    }

    fn push_completions(
        &mut self,
        other_completions: &[super::AgentCompletionChunk],
    ) {
        fn push_completion(
            completions: &mut Vec<super::AgentCompletionChunk>,
            other: &super::AgentCompletionChunk,
        ) {
            fn find_completion(
                completions: &mut Vec<super::AgentCompletionChunk>,
                index: u64,
            ) -> Option<&mut super::AgentCompletionChunk> {
                for completion in completions {
                    if completion.index == index {
                        return Some(completion);
                    }
                }
                None
            }
            if let Some(completion) = find_completion(completions, other.index)
            {
                completion.push(other);
            } else {
                completions.push(other.clone());
            }
        }
        for other_completion in other_completions {
            push_completion(&mut self.completions, other_completion);
        }
    }

    /// Produces the `(path, file_bytes)` pairs for the log file structure.
    ///
    /// Returns `(reference, files)`. All paths relative to `logs/`.
    #[cfg(feature = "filesystem")]
    pub fn produce_files(&self) -> Option<(serde_json::Value, Vec<(String, Vec<u8>)>)> {
        const PREFIX: &str = "functions/inventions/";

        let id = &self.id;
        if id.is_empty() {
            return None;
        }

        let path = format!("{PREFIX}{id}.json");
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut completion_refs: Vec<serde_json::Value> = Vec::new();

        for completion in &self.completions {
            let (reference, completion_files) = completion.produce_files();
            completion_refs.push(reference);
            files.extend(completion_files);
        }

        // Serialize a shell without completions to avoid double-serialization
        let shell = FunctionInventionChunk {
            id: self.id.clone(),
            completions: Vec::new(),
            state: self.state.clone(),
            path: self.path.clone(),
            function: self.function.clone(),
            created: self.created,
            object: self.object,
            usage: self.usage.clone(),
            error: self.error.clone(),
        };
        let mut root = serde_json::to_value(&shell).unwrap();
        root["completions"] = serde_json::Value::Array(completion_refs);
        files.push((path.clone(), serde_json::to_vec_pretty(&root).unwrap()));

        Some((serde_json::json!({ "type": "reference", "path": path }), files))
    }
}
