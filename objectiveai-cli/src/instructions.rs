//! Shared helpers for the `instructions` subcommand family and the
//! corresponding `--instructions-id` argument required by every
//! streaming `create` command.
//!
//! Flow:
//! 1. The user runs e.g. `objectiveai agents completions instructions`
//!    → the CLI prints the embedded `INSTRUCTIONS.md` content followed
//!    by `\n\n Instructions ID: <uuid>`.
//! 2. The user passes that ID back to the matching `create` command via
//!    `--instructions-id <ID>`.
//!
//! There is **no** verification of the supplied ID right now — it's
//! purely a required argument. This keeps the scaffolding in place for
//! a future validation step without locking behaviour in.

use clap::Args;

/// Embeddable `--instructions-id <ID>` argument. Flattened into each
/// streaming `create` command's args struct.
#[derive(Args, Clone, Debug)]
pub struct InstructionsIdArg {
    /// ID from the matching `instructions` subcommand. Required.
    #[arg(long)]
    pub instructions_id: String,
}

/// Formats the output of an `instructions` subcommand: the raw
/// `INSTRUCTIONS.md` content, then a blank line, then the ID footer.
///
/// `content` is the compile-time-embedded markdown string (via
/// `include_str!`). The ID is a freshly-generated 128-bit random value
/// (UUIDv4 source of randomness) encoded as base62 and left-padded to
/// 22 characters — matches the same scheme objectiveai-rs uses for
/// content-addressed IDs (`XxHash3_128 → base62 → 22 chars`).
pub fn format_instructions(content: &str) -> String {
    let bits: u128 = uuid::Uuid::new_v4().as_u128();
    let id = format!("{:0>22}", base62::encode(bits));
    format!("{content}\n\n Instructions ID: {id}")
}
