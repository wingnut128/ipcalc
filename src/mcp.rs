use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::ServerCapabilities;
use rmcp::{ServerHandler, ServiceExt, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;

use crate::contains::{check_ipv4_contains, check_ipv6_contains};
use crate::from_range::{from_range_ipv4, from_range_ipv6};
use crate::ipv4::Ipv4Subnet;
use crate::ipv6::Ipv6Subnet;
use crate::subnet_generator::{count_subnets, generate_ipv4_subnets, generate_ipv6_subnets};
use crate::summarize::{summarize_ipv4, summarize_ipv6};

#[derive(Debug, Deserialize, JsonSchema)]
struct SubnetCalcParams {
    /// IP address in CIDR notation, e.g. 192.168.1.0/24 or 2001:db8::/48
    cidr: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SubnetSplitParams {
    /// Supernet in CIDR notation, e.g. 10.0.0.0/8
    cidr: String,
    /// New prefix length for the generated subnets
    prefix: u8,
    /// Number of subnets to generate (mutually exclusive with max)
    count: Option<u64>,
    /// Generate all possible subnets (mutually exclusive with count)
    max: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct ContainsCheckParams {
    /// Network in CIDR notation, e.g. 192.168.1.0/24
    cidr: String,
    /// IP address to check, e.g. 192.168.1.100
    address: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct FromRangeParams {
    /// Start IP address, e.g. 192.168.1.10 or 2001:db8::1
    start: String,
    /// End IP address, e.g. 192.168.1.20 or 2001:db8::ff
    end: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct SummarizeParams {
    /// CIDR ranges to summarize, e.g. ["192.168.0.0/24", "192.168.1.0/24"]
    cidrs: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct IpCalcMcp {
    tool_router: ToolRouter<Self>,
}

impl IpCalcMcp {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }
}

impl Default for IpCalcMcp {
    fn default() -> Self {
        Self::new()
    }
}

fn is_ipv6(s: &str) -> bool {
    s.contains(':')
}

fn result_to_string<T: serde::Serialize>(result: crate::error::Result<T>) -> String {
    match result {
        Ok(val) => serde_json::to_string_pretty(&val).unwrap_or_else(|e| format!("Error: {e}")),
        Err(e) => format!("Error: {e}"),
    }
}

#[tool_router]
impl IpCalcMcp {
    #[tool(
        name = "subnet_calc",
        description = "Calculate IPv4 or IPv6 subnet details from CIDR notation. Returns network address, broadcast, mask, host range, total/usable hosts, network class (IPv4), address type, and more."
    )]
    async fn subnet_calc(&self, Parameters(params): Parameters<SubnetCalcParams>) -> String {
        if is_ipv6(&params.cidr) {
            result_to_string(Ipv6Subnet::from_cidr(&params.cidr))
        } else {
            result_to_string(Ipv4Subnet::from_cidr(&params.cidr))
        }
    }

    #[tool(
        name = "subnet_split",
        description = "Split a supernet into smaller subnets. Provide either a count or set max=true to generate all possible subnets. Auto-detects IPv4 vs IPv6."
    )]
    async fn subnet_split(&self, Parameters(params): Parameters<SubnetSplitParams>) -> String {
        let max = params.max.unwrap_or(false);
        if !max && params.count.is_none() {
            if let Ok(summary) = count_subnets(&params.cidr, params.prefix) {
                return serde_json::to_string_pretty(&summary)
                    .unwrap_or_else(|e| format!("Error: {e}"));
            }
            return "Error: Either count or max must be specified".to_string();
        }

        let count = if max {
            match count_subnets(&params.cidr, params.prefix) {
                Ok(summary) => summary.available_subnets.parse::<u64>().unwrap_or(u64::MAX),
                Err(e) => return format!("Error: {e}"),
            }
        } else {
            params.count.unwrap_or(1)
        };

        if is_ipv6(&params.cidr) {
            result_to_string(generate_ipv6_subnets(
                &params.cidr,
                params.prefix,
                Some(count),
            ))
        } else {
            result_to_string(generate_ipv4_subnets(
                &params.cidr,
                params.prefix,
                Some(count),
            ))
        }
    }

    #[tool(
        name = "contains_check",
        description = "Check if an IP address is contained within a CIDR range. Auto-detects IPv4 vs IPv6."
    )]
    async fn contains_check(&self, Parameters(params): Parameters<ContainsCheckParams>) -> String {
        if is_ipv6(&params.cidr) {
            result_to_string(check_ipv6_contains(&params.cidr, &params.address))
        } else {
            result_to_string(check_ipv4_contains(&params.cidr, &params.address))
        }
    }

    #[tool(
        name = "from_range",
        description = "Convert an IP address range (start-end) into minimal CIDR blocks. Auto-detects IPv4 vs IPv6."
    )]
    async fn from_range(&self, Parameters(params): Parameters<FromRangeParams>) -> String {
        if is_ipv6(&params.start) {
            result_to_string(from_range_ipv6(&params.start, &params.end))
        } else {
            result_to_string(from_range_ipv4(&params.start, &params.end))
        }
    }

    #[tool(
        name = "summarize",
        description = "Aggregate/summarize a list of CIDRs into the minimal covering set. All CIDRs must be the same address family (all IPv4 or all IPv6)."
    )]
    async fn summarize(&self, Parameters(params): Parameters<SummarizeParams>) -> String {
        if params.cidrs.is_empty() {
            return "Error: At least one CIDR is required".to_string();
        }
        if is_ipv6(&params.cidrs[0]) {
            result_to_string(summarize_ipv6(&params.cidrs))
        } else {
            result_to_string(summarize_ipv4(&params.cidrs))
        }
    }
}

#[tool_handler]
impl ServerHandler for IpCalcMcp {
    fn get_info(&self) -> rmcp::model::ServerInfo {
        rmcp::model::ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(rmcp::model::Implementation::new(
                "ipcalc",
                env!("CARGO_PKG_VERSION"),
            ))
    }
}

pub async fn run_mcp_server() -> crate::error::Result<()> {
    let server = IpCalcMcp::new();
    let transport = rmcp::transport::io::stdio();
    let service = server
        .serve(transport)
        .await
        .map_err(|e| crate::error::IpCalcError::InvalidInput(format!("MCP server error: {e}")))?;
    service
        .waiting()
        .await
        .map_err(|e| crate::error::IpCalcError::InvalidInput(format!("MCP server error: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_ipv6() {
        assert!(is_ipv6("2001:db8::/32"));
        assert!(is_ipv6("::1"));
        assert!(!is_ipv6("192.168.1.0/24"));
        assert!(!is_ipv6("10.0.0.1"));
    }

    #[tokio::test]
    async fn test_subnet_calc_ipv4() {
        let server = IpCalcMcp::new();
        let result = server
            .subnet_calc(Parameters(SubnetCalcParams {
                cidr: "192.168.1.0/24".into(),
            }))
            .await;
        assert!(result.contains("192.168.1.0"));
        assert!(result.contains("192.168.1.255"));
        assert!(!result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_subnet_calc_ipv6() {
        let server = IpCalcMcp::new();
        let result = server
            .subnet_calc(Parameters(SubnetCalcParams {
                cidr: "2001:db8::/48".into(),
            }))
            .await;
        assert!(result.contains("2001:db8::"));
        assert!(!result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_subnet_calc_invalid() {
        let server = IpCalcMcp::new();
        let result = server
            .subnet_calc(Parameters(SubnetCalcParams {
                cidr: "not-a-cidr".into(),
            }))
            .await;
        assert!(result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_subnet_split_with_count() {
        let server = IpCalcMcp::new();
        let result = server
            .subnet_split(Parameters(SubnetSplitParams {
                cidr: "10.0.0.0/8".into(),
                prefix: 16,
                count: Some(3),
                max: None,
            }))
            .await;
        assert!(result.contains("10.0.0.0"));
        assert!(!result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_subnet_split_with_max() {
        let server = IpCalcMcp::new();
        let result = server
            .subnet_split(Parameters(SubnetSplitParams {
                cidr: "192.168.0.0/24".into(),
                prefix: 26,
                count: None,
                max: Some(true),
            }))
            .await;
        assert!(result.contains("192.168.0.0"));
        assert!(!result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_subnet_split_no_count_no_max() {
        let server = IpCalcMcp::new();
        let result = server
            .subnet_split(Parameters(SubnetSplitParams {
                cidr: "10.0.0.0/8".into(),
                prefix: 16,
                count: None,
                max: None,
            }))
            .await;
        // Should return count summary, not an error
        assert!(result.contains("available_subnets"));
    }

    #[tokio::test]
    async fn test_contains_check_ipv4_contained() {
        let server = IpCalcMcp::new();
        let result = server
            .contains_check(Parameters(ContainsCheckParams {
                cidr: "192.168.1.0/24".into(),
                address: "192.168.1.100".into(),
            }))
            .await;
        assert!(result.contains("true"));
        assert!(!result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_contains_check_ipv4_not_contained() {
        let server = IpCalcMcp::new();
        let result = server
            .contains_check(Parameters(ContainsCheckParams {
                cidr: "192.168.1.0/24".into(),
                address: "10.0.0.1".into(),
            }))
            .await;
        assert!(result.contains("false"));
    }

    #[tokio::test]
    async fn test_contains_check_ipv6() {
        let server = IpCalcMcp::new();
        let result = server
            .contains_check(Parameters(ContainsCheckParams {
                cidr: "2001:db8::/32".into(),
                address: "2001:db8::1".into(),
            }))
            .await;
        assert!(result.contains("true"));
    }

    #[tokio::test]
    async fn test_from_range_ipv4() {
        let server = IpCalcMcp::new();
        let result = server
            .from_range(Parameters(FromRangeParams {
                start: "192.168.1.0".into(),
                end: "192.168.1.255".into(),
            }))
            .await;
        assert!(result.contains("192.168.1.0/24"));
        assert!(!result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_from_range_ipv6() {
        let server = IpCalcMcp::new();
        let result = server
            .from_range(Parameters(FromRangeParams {
                start: "2001:db8::".into(),
                end: "2001:db8::ff".into(),
            }))
            .await;
        assert!(result.contains("2001:db8::"));
        assert!(!result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_summarize_ipv4() {
        let server = IpCalcMcp::new();
        let result = server
            .summarize(Parameters(SummarizeParams {
                cidrs: vec!["192.168.0.0/24".into(), "192.168.1.0/24".into()],
            }))
            .await;
        assert!(result.contains("192.168.0.0/23"));
        assert!(!result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_summarize_ipv6() {
        let server = IpCalcMcp::new();
        let result = server
            .summarize(Parameters(SummarizeParams {
                cidrs: vec!["2001:db8::/48".into(), "2001:db8:1::/48".into()],
            }))
            .await;
        assert!(!result.starts_with("Error"));
    }

    #[tokio::test]
    async fn test_summarize_empty() {
        let server = IpCalcMcp::new();
        let result = server
            .summarize(Parameters(SummarizeParams { cidrs: vec![] }))
            .await;
        assert!(result.starts_with("Error"));
    }
}
