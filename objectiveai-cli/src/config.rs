use objectiveai::filesystem::config::Config;
use objectiveai::filesystem::Client;

pub fn filter(f: Option<String>) -> String {
    f.unwrap_or_else(|| ".".to_string())
}

pub async fn read(cli_config: &super::Config) -> Result<(Client, Config), crate::error::Error> {
    let client = Client::new(cli_config.config_base_dir.as_deref(), None::<String>, None::<String>);
    let config = objectiveai::filesystem::config::client::read(&client).await?;
    Ok((client, config))
}

pub async fn write(client: &Client, config: &Config, cli_config: &super::Config) -> Result<(), crate::error::Error> {
    if cli_config.config_set_forbidden {
        return Err(crate::error::Error::ConfigSetForbidden);
    }
    objectiveai::filesystem::config::client::write(client, config).await?;
    Ok(())
}

pub fn format_jq(results: Result<Vec<serde_json::Value>, objectiveai::filesystem::Error>) -> Result<crate::Output, crate::error::Error> {
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
