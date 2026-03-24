pub mod config;
pub mod objectiveai_authorization;
pub mod openrouter_authorization;
pub mod github_authorization;
pub mod mcp_authorization;
pub mod viewer_signature;
pub mod viewer_address;
pub mod user_agent;
pub mod http_referer;
pub mod x_title;
pub mod commit_author_name;
pub mod commit_author_email;

use clap::Subcommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Headers configuration
    Config { #[command(subcommand)] command: config::Commands },
    /// ObjectiveAI authorization
    ObjectiveaiAuthorization { #[command(subcommand)] command: objectiveai_authorization::Commands },
    /// OpenRouter authorization
    OpenrouterAuthorization { #[command(subcommand)] command: openrouter_authorization::Commands },
    /// GitHub authorization
    GithubAuthorization { #[command(subcommand)] command: github_authorization::Commands },
    /// MCP authorization
    McpAuthorization { #[command(subcommand)] command: mcp_authorization::Commands },
    /// Viewer signature
    ViewerSignature { #[command(subcommand)] command: viewer_signature::Commands },
    /// Viewer address
    ViewerAddress { #[command(subcommand)] command: viewer_address::Commands },
    /// User-Agent header
    UserAgent { #[command(subcommand)] command: user_agent::Commands },
    /// HTTP Referer header
    HttpReferer { #[command(subcommand)] command: http_referer::Commands },
    /// X-Title header
    XTitle { #[command(subcommand)] command: x_title::Commands },
    /// Commit author name
    CommitAuthorName { #[command(subcommand)] command: commit_author_name::Commands },
    /// Commit author email
    CommitAuthorEmail { #[command(subcommand)] command: commit_author_email::Commands },
}

impl Commands {
    pub fn handle(self) -> Result<crate::Output, crate::error::Error> {
        match self {
            Commands::Config { command } => command.handle(),
            Commands::ObjectiveaiAuthorization { command } => command.handle(),
            Commands::OpenrouterAuthorization { command } => command.handle(),
            Commands::GithubAuthorization { command } => command.handle(),
            Commands::McpAuthorization { command } => command.handle(),
            Commands::ViewerSignature { command } => command.handle(),
            Commands::ViewerAddress { command } => command.handle(),
            Commands::UserAgent { command } => command.handle(),
            Commands::HttpReferer { command } => command.handle(),
            Commands::XTitle { command } => command.handle(),
            Commands::CommitAuthorName { command } => command.handle(),
            Commands::CommitAuthorEmail { command } => command.handle(),
        }
    }
}
