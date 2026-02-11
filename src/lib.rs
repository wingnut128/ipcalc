pub mod api;
pub mod cli;
pub mod contains;
pub mod error;
pub mod from_range;
pub mod ipv4;
pub mod ipv6;
pub mod logging;
pub mod output;
pub mod subnet_generator;
pub mod summarize;

#[cfg(feature = "tui")]
pub mod tui;

pub use contains::ContainsResult;
pub use error::IpCalcError;
pub use from_range::{Ipv4FromRangeResult, Ipv6FromRangeResult};
pub use ipv4::Ipv4Subnet;
pub use ipv6::Ipv6Subnet;
pub use logging::{LogConfig, init_logging};
pub use output::{OutputFormat, OutputWriter};
pub use summarize::{Ipv4SummaryResult, Ipv6SummaryResult};
