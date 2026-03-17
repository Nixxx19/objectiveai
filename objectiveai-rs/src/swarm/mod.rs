//! Swarm definitions and validation.
//!
//! An **Swarm** is a collection of [`Agent`](crate::agent::Agent)s
//! used together. Swarms are the foundation of ObjectiveAI's multi-model approach.
//!
//! # Key Properties
//!
//! - **Immutable**: Any change produces a new Swarm ID
//! - **No weights**: Weights are execution-time parameters, not part of the Swarm
//! - **Content-addressed**: IDs are deterministically computed from the definition
//! - **Deduplicated**: Duplicate agents are merged with their counts summed
//! - **Bounded**: Total agent count must be between 1 and 128 (individual agents with
//!   `count: 0` are skipped, but the sum of all counts must be at least 1)
//!
//! # Example
//!
//! ```
//! use objectiveai::swarm::{SwarmBase, Swarm};
//! use objectiveai::agent::{AgentBase, AgentBaseWithFallbacksAndCount};
//! use objectiveai::agent::openrouter;
//! use objectiveai::agent::completions::message::{Message, SystemMessage, SimpleContent};
//!
//! let swarm_base = SwarmBase {
//!     agents: vec![
//!         // A simple GPT-4 configuration
//!         AgentBaseWithFallbacksAndCount {
//!             count: 1,
//!             inner: AgentBase::Openrouter(openrouter::AgentBase {
//!                 model: "openai/gpt-4o".to_string(),
//!                 ..Default::default()
//!             }),
//!             fallbacks: None,
//!         },
//!         // Claude with a system prompt
//!         AgentBaseWithFallbacksAndCount {
//!             count: 1,
//!             inner: AgentBase::Openrouter(openrouter::AgentBase {
//!                 model: "anthropic/claude-3.5-sonnet".to_string(),
//!                 output_mode: openrouter::OutputMode::JsonSchema,
//!                 prefix_messages: Some(vec![
//!                     Message::System(SystemMessage {
//!                         content: SimpleContent::Text("You are a careful evaluator.".to_string()),
//!                         name: None,
//!                     }),
//!                 ]),
//!                 ..Default::default()
//!             }),
//!             fallbacks: None,
//!         },
//!         // Gemini with lower temperature
//!         AgentBaseWithFallbacksAndCount {
//!             count: 2, // Include 2 instances
//!             inner: AgentBase::Openrouter(openrouter::AgentBase {
//!                 model: "google/gemini-2.0-flash-001".to_string(),
//!                 output_mode: openrouter::OutputMode::ToolCall,
//!                 temperature: Some(0.3),
//!                 ..Default::default()
//!             }),
//!             fallbacks: None,
//!         },
//!     ],
//! };
//!
//! let swarm: Swarm = swarm_base.try_into().unwrap();
//! println!("Swarm ID: {}", swarm.id);
//! ```

mod swarm;
pub mod response;

pub use swarm::*;

#[cfg(test)]
mod swarm_tests;

#[cfg(feature = "http")]
mod http;

#[cfg(feature = "http")]
pub use http::*;
