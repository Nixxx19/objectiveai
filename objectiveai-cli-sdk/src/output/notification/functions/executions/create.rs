use serde::{Deserialize, Serialize};

/// Result of `functions executions create`.
///
/// Wire: `{"type":"notification","execution":{"output":...,"errors":[...]}}`.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Execution {
    pub execution: ExecutionResult,
}

/// Body of an execution result: the final task output plus any errors
/// collected from the aggregated chunk tree.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ExecutionResult {
    pub output: objectiveai::functions::expression::TaskOutputOwned,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<CollectedError>,
}

/// A collected error with its location in the execution tree.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollectedError {
    pub path: ErrorPath,
    #[serde(flatten)]
    pub error: objectiveai::error::ResponseError,
}

/// Where in the execution tree an error occurred. Serializes as either
/// `"root"` / `"reasoning"` (the singletons) or as a JSON array of
/// node indices (`Task`).
#[derive(Debug, Clone)]
pub enum ErrorPath {
    Root,
    Task(Vec<u64>),
    Reasoning,
}

impl Serialize for ErrorPath {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ErrorPath::Root => serializer.serialize_str("root"),
            ErrorPath::Task(path) => path.serialize(serializer),
            ErrorPath::Reasoning => serializer.serialize_str("reasoning"),
        }
    }
}

impl<'de> Deserialize<'de> for ErrorPath {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::Error;
        let v = serde_json::Value::deserialize(d)?;
        match v {
            serde_json::Value::String(s) if s == "root" => Ok(ErrorPath::Root),
            serde_json::Value::String(s) if s == "reasoning" => Ok(ErrorPath::Reasoning),
            serde_json::Value::String(s) => {
                Err(D::Error::custom(format!("unknown ErrorPath: {s}")))
            }
            serde_json::Value::Array(_) => {
                let path: Vec<u64> = serde_json::from_value(v).map_err(D::Error::custom)?;
                Ok(ErrorPath::Task(path))
            }
            _ => Err(D::Error::custom("expected string or array for ErrorPath")),
        }
    }
}
