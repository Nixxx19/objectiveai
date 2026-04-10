use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ConfigClient {
    base_dir: PathBuf,
}

impl ConfigClient {
    pub fn new(base_dir: Option<impl Into<PathBuf>>) -> Self {
        let base_dir = match base_dir {
            Some(dir) => dir.into(),
            None => {
                #[cfg(feature = "env")]
                if let Ok(dir) = std::env::var("CONFIG_BASE_DIR") {
                    return Self { base_dir: PathBuf::from(dir) };
                }
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".objectiveai")
            }
        };
        Self { base_dir }
    }

    fn config_path(&self) -> PathBuf {
        self.base_dir.join("config.json")
    }

    pub async fn read(&self) -> Result<super::Config, super::ConfigError> {
        let path = self.config_path();
        match tokio::fs::read(&path).await {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|e| super::ConfigError::Parse(path, e)),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(super::Config::default())
            }
            Err(e) => Err(super::ConfigError::Read(path, e)),
        }
    }

    pub async fn write(&self, config: &super::Config) -> Result<(), super::ConfigError> {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await
                .map_err(|e| super::ConfigError::Write(parent.to_path_buf(), e))?;
        }
        let bytes = serde_json::to_vec_pretty(config)
            .map_err(super::ConfigError::Serialize)?;
        tokio::fs::write(&path, bytes).await
            .map_err(|e| super::ConfigError::Write(path, e))?;
        Ok(())
    }
}
