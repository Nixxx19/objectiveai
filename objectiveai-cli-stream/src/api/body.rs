//! JSON body input — mirrors the JSON variants of
//! `objectiveai-cli/src/api/body.rs::BodySource`. The python flavors
//! stay in the original CLI since they require its `python` module.

use std::path::PathBuf;

use clap::Args;

/// `--body-inline` xor `--body-file`. Exactly one is required.
#[derive(Args, Debug, Clone)]
#[group(required = true, multiple = false)]
pub struct BodySource {
    /// Inline JSON body.
    #[arg(long)]
    pub body_inline: Option<String>,

    /// Path to a JSON body file.
    #[arg(long)]
    pub body_file: Option<PathBuf>,
}

impl BodySource {
    /// Parse the configured body into `T`.
    pub fn resolve<T: serde::de::DeserializeOwned>(self) -> Result<T, String> {
        if let Some(inline) = self.body_inline {
            let mut de = serde_json::Deserializer::from_str(&inline);
            return serde_path_to_error::deserialize(&mut de).map_err(|e| {
                format!("--body-inline parse error at `{}`: {}", e.path(), e.inner())
            });
        }
        if let Some(path) = self.body_file {
            let contents = std::fs::read_to_string(&path)
                .map_err(|e| format!("--body-file read error ({}): {e}", path.display()))?;
            let mut de = serde_json::Deserializer::from_str(&contents);
            return serde_path_to_error::deserialize(&mut de).map_err(|e| {
                format!(
                    "--body-file parse error ({}) at `{}`: {}",
                    path.display(),
                    e.path(),
                    e.inner()
                )
            });
        }
        unreachable!("clap group ensures one is set")
    }
}
