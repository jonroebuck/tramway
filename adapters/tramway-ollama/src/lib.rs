//! Ollama adapter for the tramway framework.
//!
//! This crate is an **adapter** in the tramway architecture: it bridges the
//! [`Intelligence`] port defined in `tramway-core` and the Ollama HTTP API,
//! routing prompts to a model served by a locally running Ollama instance.
//!
//! The primary entry point is [`LocalIntelligence`], re-exported from
//! [`local_intelligence`] for convenience.
//!
//! # Usage
//!
//! ```rust,no_run
//! use tramway_ollama::LocalIntelligence;
//! use tramway_core::{Intelligence, IntelligenceContext};
//!
//! # #[tokio::main]
//! # async fn main() {
//! let intel = LocalIntelligence::new("http://localhost:11434", "llama3");
//! let ctx = IntelligenceContext {
//!     input: "What is 2 + 2?".to_string(),
//!     system: String::new(),
//!     history: vec![],
//!     metadata: Default::default(),
//! };
//! let response = intel.respond(ctx).await.unwrap();
//! # }
//! ```

pub mod local_intelligence;

pub use local_intelligence::LocalIntelligence;
