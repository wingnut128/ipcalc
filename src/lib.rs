//! A fast IPv4 and IPv6 subnet calculator.
//!
//! Provides CLI, TUI, and HTTP API interfaces for subnet calculations,
//! including prefix lookups, subnet splitting, address containment checks,
//! range-to-CIDR conversion, and route summarization.

// Core calculation modules
pub mod batch;
pub mod contains;
pub mod from_range;
pub mod ipv4;
pub mod ipv6;
pub mod subnet_generator;
pub mod summarize;

// I/O and interface modules
pub mod api;
pub mod cli;
pub mod output;

// Infrastructure
pub mod config;
pub mod error;
pub mod logging;

#[cfg(feature = "tui")]
pub mod tui;

// Public API re-exports
pub use batch::{BatchResult, process_batch, process_batch_with_limit};
pub use contains::ContainsResult;
pub use from_range::{Ipv4FromRangeResult, Ipv6FromRangeResult};
pub use ipv4::Ipv4Subnet;
pub use ipv6::Ipv6Subnet;
pub use logging::{LogConfig, init_logging};
pub use output::{OutputFormat, OutputWriter};
pub use summarize::{Ipv4SummaryResult, Ipv6SummaryResult};
