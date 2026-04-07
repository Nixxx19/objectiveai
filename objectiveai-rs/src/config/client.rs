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

    pub fn read(&self) -> Result<super::Config, super::ConfigError> {
        let path = self.config_path();
        if !path.exists() {
            return Ok(super::Config::default());
        }
        let file = std::fs::File::open(&path)
            .map_err(|e| super::ConfigError::Read(path.clone(), e))?;
        serde_json::from_reader(file)
            .map_err(|e| super::ConfigError::Parse(path, e))
    }

    pub fn write(&self, config: &super::Config) -> Result<(), super::ConfigError> {
        let path = self.config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| super::ConfigError::Write(parent.to_path_buf(), e))?;
        }
        let file = std::fs::File::create(&path)
            .map_err(|e| super::ConfigError::Write(path.clone(), e))?;
        serde_json::to_writer_pretty(file, config)
            .map_err(super::ConfigError::Serialize)?;
        Ok(())
    }
}
