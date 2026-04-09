use crate::{agent, error, functions};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema, arbitrary::Arbitrary)]
#[schemars(rename = "functions.executions.response.streaming.FunctionExecutionChunk")]
pub struct FunctionExecutionChunk {
    pub id: String,
    pub tasks: Vec<super::TaskChunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub tasks_errors: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub reasoning: Option<super::ReasoningSummaryChunk>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub output: Option<super::super::Output>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub error: Option<error::ResponseError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub retry_token: Option<String>,
    #[arbitrary(with = crate::arbitrary_util::arbitrary_u64)]
    pub created: u64,
    pub function: Option<crate::RemotePath>,
    pub profile: Option<crate::RemotePath>,
    pub object: super::Object,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schemars(extend("omitempty" = true))]
    pub usage: Option<agent::completions::response::Usage>,
}

impl FunctionExecutionChunk {
    pub fn vector_completion_tasks(
        &self,
    ) -> impl Iterator<Item = &super::VectorCompletionTaskChunk> {
        self.tasks
            .iter()
            .flat_map(|task| task.vector_completion_tasks())
    }

    pub fn any_usage(&self) -> bool {
        self.usage
            .as_ref()
            .is_some_and(agent::completions::response::Usage::any_usage)
    }

    pub fn push(
        &mut self,
        FunctionExecutionChunk {
            tasks,
            tasks_errors,
            reasoning,
            output,
            retry_token,
            error,
            usage,
            ..
        }: &FunctionExecutionChunk,
    ) {
        self.push_tasks(tasks);
        if let Some(true) = tasks_errors {
            self.tasks_errors = Some(true);
        }
        match (&mut self.reasoning, &reasoning) {
            (Some(self_reasoning), Some(other_reasoning)) => {
                self_reasoning.push(other_reasoning);
            }
            (None, Some(other_reasoning)) => {
                self.reasoning = Some(other_reasoning.clone());
            }
            _ => {}
        }
        if let Some(output) = output {
            self.output = Some(output.clone());
        }
        if let Some(retry_token) = retry_token {
            self.retry_token = Some(retry_token.clone());
        }
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

    fn push_tasks(&mut self, other_tasks: &[super::TaskChunk]) {
        fn push_task(
            tasks: &mut Vec<super::TaskChunk>,
            other: &super::TaskChunk,
        ) {
            fn find_task(
                tasks: &mut Vec<super::TaskChunk>,
                index: u64,
            ) -> Option<&mut super::TaskChunk> {
                for task in tasks {
                    if task.index() == index {
                        return Some(task);
                    }
                }
                None
            }
            if let Some(task) = find_task(tasks, other.index()) {
                task.push(other);
            } else {
                tasks.push(other.clone());
            }
        }
        for other_task in other_tasks {
            push_task(&mut self.tasks, other_task);
        }
    }

    /// Produces the `(path, file_bytes)` pairs for the log file structure.
    ///
    /// Returns `(reference, files)`. All paths relative to `logs/`.
    #[cfg(feature = "filesystem")]
    pub fn produce_files(&self) -> Option<(serde_json::Value, Vec<(String, Vec<u8>)>)> {
        const PREFIX: &str = "functions/executions/";

        let id = &self.id;
        if id.is_empty() {
            return None;
        }

        let path = format!("{PREFIX}{id}.json");
        let mut files: Vec<(String, Vec<u8>)> = Vec::new();
        let mut task_refs: Vec<serde_json::Value> = Vec::new();

        for task in &self.tasks {
            let (reference, task_files) = task.produce_files();
            task_refs.push(reference);
            files.extend(task_files);
        }

        // Extract reasoning summary
        let reasoning_ref = self.reasoning.as_ref().map(|r| {
            let (reference, reasoning_files) = r.produce_files();
            files.extend(reasoning_files);
            reference
        });

        // Serialize a shell without tasks/reasoning to avoid double-serialization
        let shell = FunctionExecutionChunk {
            id: self.id.clone(),
            tasks: Vec::new(),
            tasks_errors: self.tasks_errors,
            reasoning: None,
            output: self.output.clone(),
            error: self.error.clone(),
            retry_token: Some(String::new()),
            created: self.created,
            function: self.function.clone(),
            profile: self.profile.clone(),
            object: self.object,
            usage: self.usage.clone(),
        };
        let mut root = serde_json::to_value(&shell).unwrap();
        root["tasks"] = serde_json::Value::Array(task_refs);
        if let Some(reasoning_ref) = reasoning_ref {
            root["reasoning"] = reasoning_ref;
        }

        // Extract retry token to a separate file, or remove placeholder
        if let Some(retry_token) = &self.retry_token {
            let rt_path = format!("{PREFIX}retry_token/{id}.json");
            files.push((rt_path.clone(), serde_json::to_vec_pretty(retry_token).unwrap()));
            root["retry_token"] = serde_json::json!({
                "type": "reference",
                "path": rt_path,
            });
        } else if let Some(map) = root.as_object_mut() {
            map.remove("retry_token");
        }

        files.push((path.clone(), serde_json::to_vec_pretty(&root).unwrap()));

        Some((serde_json::json!({ "type": "reference", "path": path }), files))
    }
}
