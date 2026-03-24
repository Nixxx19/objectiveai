pub mod config;
pub mod x_objectiveai_authorization;
pub mod x_openrouter_authorization;
pub mod x_github_authorization;
pub mod x_mcp_authorization;
pub mod x_viewer_signature;
pub mod x_viewer_address;
pub mod user_agent;
pub mod http_referer;
pub mod x_title;
pub mod x_commit_author_name;
pub mod x_commit_author_email;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Headers configuration
    Config { #[command(subcommand)] command: config::Commands },
    /// X-OBJECTIVEAI-AUTHORIZATION
    XObjectiveaiAuthorization { #[command(subcommand)] command: x_objectiveai_authorization::Commands },
    /// X-OPENROUTER-AUTHORIZATION
    XOpenrouterAuthorization { #[command(subcommand)] command: x_openrouter_authorization::Commands },
    /// X-GITHUB-AUTHORIZATION
    XGithubAuthorization { #[command(subcommand)] command: x_github_authorization::Commands },
    /// X-MCP-AUTHORIZATION
    XMcpAuthorization { #[command(subcommand)] command: x_mcp_authorization::Commands },
    /// X-VIEWER-SIGNATURE
    XViewerSignature { #[command(subcommand)] command: x_viewer_signature::Commands },
    /// X-VIEWER-ADDRESS
    XViewerAddress { #[command(subcommand)] command: x_viewer_address::Commands },
    /// User-Agent
    UserAgent { #[command(subcommand)] command: user_agent::Commands },
    /// HTTP-Referer
    HttpReferer { #[command(subcommand)] command: http_referer::Commands },
    /// X-Title
    XTitle { #[command(subcommand)] command: x_title::Commands },
    /// X-COMMIT-AUTHOR-NAME
    XCommitAuthorName { #[command(subcommand)] command: x_commit_author_name::Commands },
    /// X-COMMIT-AUTHOR-EMAIL
    XCommitAuthorEmail { #[command(subcommand)] command: x_commit_author_email::Commands },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::XObjectiveaiAuthorization { command } => command.handle(),
            Commands::XOpenrouterAuthorization { command } => command.handle(),
            Commands::XGithubAuthorization { command } => command.handle(),
            Commands::XMcpAuthorization { command } => command.handle(),
            Commands::XViewerSignature { command } => command.handle(),
            Commands::XViewerAddress { command } => command.handle(),
            Commands::UserAgent { command } => command.handle(),
            Commands::HttpReferer { command } => command.handle(),
            Commands::XTitle { command } => command.handle(),
            Commands::XCommitAuthorName { command } => command.handle(),
            Commands::XCommitAuthorEmail { command } => command.handle(),
        }
    }
}
