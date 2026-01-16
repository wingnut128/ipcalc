use thiserror::Error;

#[derive(Error, Debug)]
pub enum IpCalcError {
    #[error("Invalid IPv4 address: {0}")]
    InvalidIpv4Address(String),

    #[error("Invalid IPv6 address: {0}")]
    InvalidIpv6Address(String),

    #[error("Invalid CIDR notation: {0}")]
    InvalidCidr(String),

    #[error("Invalid prefix length: {0} (must be 0-32 for IPv4, 0-128 for IPv6)")]
    InvalidPrefixLength(u8),

    #[error(
        "Cannot generate {requested} /{new_prefix} subnets from /{original_prefix} (only {available} available)"
    )]
    InsufficientSubnets {
        requested: u64,
        available: u64,
        new_prefix: u8,
        original_prefix: u8,
    },

    #[error(
        "New prefix length {new_prefix} must be greater than original prefix {original_prefix}"
    )]
    InvalidSubnetSplit { new_prefix: u8, original_prefix: u8 },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, IpCalcError>;
