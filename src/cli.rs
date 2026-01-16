use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(name = "ipcalc")]
#[command(version)]
#[command(about = "IP subnet calculator for IPv4 and IPv6", long_about = None)]
pub struct Cli {
    /// IP address in CIDR notation (e.g., 192.168.1.0/24 or 2001:db8::/48)
    #[arg(value_name = "CIDR")]
    pub cidr: Option<String>,

    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Output format (json or text)
    #[arg(short, long, default_value = "json", global = true)]
    pub format: OutputFormatArg,

    /// Output file path (prints to stdout if not specified)
    #[arg(short = 'o', long, global = true)]
    pub output: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Calculate IPv4 subnet information [DEPRECATED: use 'ipcalc <cidr>' instead]
    #[command(name = "v4", hide = true)]
    Ipv4 {
        /// IPv4 address in CIDR notation (e.g., 192.168.1.0/24)
        cidr: String,
    },

    /// Calculate IPv6 subnet/prefix information [DEPRECATED: use 'ipcalc <cidr>' instead]
    #[command(name = "v6", hide = true)]
    Ipv6 {
        /// IPv6 address with prefix (e.g., 2001:db8:abcd::/48)
        cidr: String,
    },

    /// Generate subnets from a supernet
    Split {
        /// Network in CIDR notation (or prefix notation for IPv6)
        cidr: String,

        /// New prefix length for subnets
        #[arg(short = 'p', long)]
        prefix: u8,

        /// Number of subnets to generate (mutually exclusive with --max)
        #[arg(short = 'n', long, conflicts_with = "max")]
        count: Option<u64>,

        /// Generate maximum number of subnets possible
        #[arg(short = 'm', long, conflicts_with = "count")]
        max: bool,
    },

    /// Start the HTTP API server
    Serve {
        /// Address to bind to
        #[arg(short, long, default_value = "127.0.0.1")]
        address: String,

        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,

        /// Log level (trace, debug, info, warn, error)
        #[arg(long, default_value = "info")]
        log_level: String,

        /// Log to file instead of stdout
        #[arg(long)]
        log_file: Option<String>,

        /// Output logs in JSON format
        #[arg(long)]
        log_json: bool,
    },
}

#[derive(Clone, Copy, ValueEnum, Default)]
pub enum OutputFormatArg {
    #[default]
    Json,
    Text,
}

impl From<OutputFormatArg> for crate::output::OutputFormat {
    fn from(arg: OutputFormatArg) -> Self {
        match arg {
            OutputFormatArg::Json => crate::output::OutputFormat::Json,
            OutputFormatArg::Text => crate::output::OutputFormat::Text,
        }
    }
}
