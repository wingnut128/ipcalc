use clap::{CommandFactory, Parser};
use ipcalc::api::create_router;
use ipcalc::cli::{Cli, Commands};
use ipcalc::ipv4::Ipv4Subnet;
use ipcalc::ipv6::Ipv6Subnet;
use ipcalc::logging::{LogConfig, init_logging, parse_log_level};
use ipcalc::output::{OutputFormat, OutputWriter};
use ipcalc::subnet_generator::{generate_ipv4_subnets, generate_ipv6_subnets};
use std::io::{self, Write};
use std::net::SocketAddr;
use tracing::info;

/// Print to stdout, handling broken pipe errors gracefully.
/// When output is piped to commands like `head`, the pipe may close early.
fn print_stdout(s: &str) {
    if let Err(e) = writeln!(io::stdout(), "{}", s) {
        if e.kind() == io::ErrorKind::BrokenPipe {
            std::process::exit(0);
        }
        eprintln!("Error writing to stdout: {}", e);
        std::process::exit(1);
    }
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let format: OutputFormat = cli.format.into();
    let writer = OutputWriter::new(format, cli.output.clone());

    // Handle direct CIDR input (auto-detect)
    if let Some(cidr) = cli.cidr {
        let is_ipv6 = cidr.contains(':');
        if is_ipv6 {
            match Ipv6Subnet::from_cidr(&cidr) {
                Ok(subnet) => {
                    let output = writer.write(&subnet).expect("Failed to write output");
                    if cli.output.is_none() {
                        print_stdout(&output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        } else {
            match Ipv4Subnet::from_cidr(&cidr) {
                Ok(subnet) => {
                    let output = writer.write(&subnet).expect("Failed to write output");
                    if cli.output.is_none() {
                        print_stdout(&output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        return;
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Ipv4 { cidr }) => {
            eprintln!("Warning: 'v4' is deprecated, use 'ipcalc <cidr>' instead");
            match Ipv4Subnet::from_cidr(&cidr) {
                Ok(subnet) => {
                    let output = writer.write(&subnet).expect("Failed to write output");
                    if cli.output.is_none() {
                        print_stdout(&output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Ipv6 { cidr }) => {
            eprintln!("Warning: 'v6' is deprecated, use 'ipcalc <cidr>' instead");
            match Ipv6Subnet::from_cidr(&cidr) {
                Ok(subnet) => {
                    let output = writer.write(&subnet).expect("Failed to write output");
                    if cli.output.is_none() {
                        print_stdout(&output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(Commands::Split {
            cidr,
            prefix,
            count,
            max,
        }) => {
            // Detect IPv4 vs IPv6 based on CIDR format
            let is_ipv6 = cidr.contains(':');

            // Determine the actual count to use
            let actual_count = if max {
                None // Signal to generate maximum
            } else {
                match count {
                    Some(c) => Some(c),
                    None => {
                        eprintln!("Error: Either --count or --max must be specified");
                        std::process::exit(1);
                    }
                }
            };

            if is_ipv6 {
                match generate_ipv6_subnets(&cidr, prefix, actual_count) {
                    Ok(result) => {
                        let output = writer.write(&result).expect("Failed to write output");
                        if cli.output.is_none() {
                            print_stdout(&output);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match generate_ipv4_subnets(&cidr, prefix, actual_count) {
                    Ok(result) => {
                        let output = writer.write(&result).expect("Failed to write output");
                        if cli.output.is_none() {
                            print_stdout(&output);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Some(Commands::Serve {
            address,
            port,
            log_level,
            log_file,
            log_json,
        }) => {
            // Parse and validate log level
            let level = match parse_log_level(&log_level) {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            };

            // Initialize logging
            let log_config = LogConfig::new(level).with_json(log_json);
            let log_config = match log_file {
                Some(path) => log_config.with_file(path),
                None => log_config,
            };

            // Keep the guard alive for the lifetime of the program
            let _guard = init_logging(&log_config);

            let addr: SocketAddr = format!("{}:{}", address, port)
                .parse()
                .expect("Invalid address");

            info!("Starting ipcalc API server on http://{}", addr);
            info!("Log level: {}", log_level);

            // Print to stdout as well for visibility
            println!("Starting ipcalc API server on http://{}", addr);
            println!("Endpoints:");
            println!("  GET /health              - Health check");
            println!("  GET /version             - Version information");
            println!("  GET /v4?cidr=<cidr>      - Calculate IPv4 subnet");
            println!("  GET /v6?cidr=<cidr>      - Calculate IPv6 subnet");
            println!("  GET /v4/split?cidr=<cidr>&prefix=<n>&count=<n> - Split IPv4 supernet");
            println!("  GET /v6/split?cidr=<cidr>&prefix=<n>&count=<n> - Split IPv6 supernet");

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, create_router()).await.unwrap();
        }
        None => {
            // Show help when no arguments are provided
            Cli::command().print_help().expect("Failed to print help");
            println!(); // Add a newline for better formatting
            std::process::exit(0);
        }
    }
}
