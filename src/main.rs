use clap::Parser;
use ipcalc::api::create_router;
use ipcalc::cli::{Cli, Commands};
use ipcalc::ipv4::Ipv4Subnet;
use ipcalc::ipv6::Ipv6Subnet;
use ipcalc::output::{OutputFormat, OutputWriter};
use ipcalc::subnet_generator::{generate_ipv4_subnets, generate_ipv6_subnets};
use std::net::SocketAddr;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let format: OutputFormat = cli.format.into();
    let writer = OutputWriter::new(format, cli.output.clone());

    match cli.command {
        Commands::Ipv4 { cidr } => {
            match Ipv4Subnet::from_cidr(&cidr) {
                Ok(subnet) => {
                    let output = writer.write(&subnet).expect("Failed to write output");
                    if cli.output.is_none() {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Ipv6 { cidr } => {
            match Ipv6Subnet::from_cidr(&cidr) {
                Ok(subnet) => {
                    let output = writer.write(&subnet).expect("Failed to write output");
                    if cli.output.is_none() {
                        println!("{}", output);
                    }
                }
                Err(e) => {
                    eprintln!("Error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Commands::Split { cidr, prefix, count } => {
            // Detect IPv4 vs IPv6 based on CIDR format
            let is_ipv6 = cidr.contains(':');

            if is_ipv6 {
                match generate_ipv6_subnets(&cidr, prefix, count) {
                    Ok(result) => {
                        let output = writer.write(&result).expect("Failed to write output");
                        if cli.output.is_none() {
                            println!("{}", output);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            } else {
                match generate_ipv4_subnets(&cidr, prefix, count) {
                    Ok(result) => {
                        let output = writer.write(&result).expect("Failed to write output");
                        if cli.output.is_none() {
                            println!("{}", output);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
        }
        Commands::Serve { address, port } => {
            let addr: SocketAddr = format!("{}:{}", address, port)
                .parse()
                .expect("Invalid address");

            println!("Starting ipcalc API server on http://{}", addr);
            println!("Endpoints:");
            println!("  GET /health              - Health check");
            println!("  GET /v4?cidr=<cidr>      - Calculate IPv4 subnet");
            println!("  GET /v6?cidr=<cidr>      - Calculate IPv6 subnet");
            println!("  GET /v4/split?cidr=<cidr>&prefix=<n>&count=<n> - Split IPv4 supernet");
            println!("  GET /v6/split?cidr=<cidr>&prefix=<n>&count=<n> - Split IPv6 supernet");

            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
            axum::serve(listener, create_router()).await.unwrap();
        }
    }
}
