use objectiveai::filesystem::config::{Config, ConfigClient, ConfigError};

pub fn filter(f: Option<String>) -> String {
    f.unwrap_or_else(|| ".".to_string())
}

pub async fn read(cli_config: &super::Config) -> Result<(ConfigClient, Config), crate::error::Error> {
    let client = ConfigClient::new(cli_config.config_base_dir.as_deref());
    let config = client.read().await?;
    Ok((client, config))
}

pub async fn write(client: &ConfigClient, config: &Config, cli_config: &super::Config) -> Result<(), crate::error::Error> {
    if cli_config.config_set_forbidden {
        return Err(crate::error::Error::ConfigSetForbidden);
    }
    client.write(config).await?;
    Ok(())
}

pub fn format_jq(results: Result<Vec<serde_json::Value>, ConfigError>) -> Result<crate::Output, crate::error::Error> {
    let results = results?;
    Ok(crate::Output::ConfigGet(match results.len() {
        0 => serde_json::to_string(&serde_json::Value::Null).unwrap(),
        1 => serde_json::to_string(&results[0]).unwrap(),
        _ => serde_json::to_string(&results).unwrap(),
    }))
}

pub fn format_value(v: &impl serde::Serialize) -> String {
    serde_json::to_string(v).unwrap()
}
