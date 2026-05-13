use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Get a function invention state by remote path or favorite name
    Get {
        #[command(flatten)]
        args: crate::get::GetArgs,
    },
}

async fn get_favorites(cli_config: &crate::Config) -> Vec<objectiveai::filesystem::config::Favorite> {
    let (_, mut config) = crate::config::read(cli_config).await.unwrap();
    config.functions().get_favorites().to_vec()
}

impl Commands {
    pub async fn handle(self, cli_config: &crate::Config, handle: &objectiveai_cli_sdk::output::Handle) -> Result<(), crate::error::Error> {
        match self {
            Commands::Get { args } => {
                let path = args.resolve(|| get_favorites(cli_config)).await?;
                let handle = handle.clone();
                crate::api::run(|http_client| async move {
                    let response = objectiveai::functions::inventions::state::get_function_invention_state(&http_client, path).await?;
                    objectiveai_cli_sdk::output::Output::<objectiveai_cli_sdk::output::State>::Notification(objectiveai_cli_sdk::output::Notification { value: 
                        objectiveai_cli_sdk::output::State { state: response },
                     })
                    .emit(&handle).await;
                    Ok(())
                }, false).await
            }
        }
    }
}
