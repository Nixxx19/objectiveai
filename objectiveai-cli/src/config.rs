use objectiveai::config::{Config, ConfigClient, ConfigError};

pub fn filter(f: Option<String>) -> String {
    f.unwrap_or_else(|| ".".to_string())
}

pub fn read() -> Result<(ConfigClient, Config), crate::error::Error> {
    let client = ConfigClient::new(None::<String>);
    let config = client.read()?;
    Ok((client, config))
}

pub fn write(client: &ConfigClient, config: &Config) -> Result<(), crate::error::Error> {
    client.write(config)?;
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
