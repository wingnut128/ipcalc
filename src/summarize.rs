use crate::error::{IpCalcError, Result};
use crate::ipv4::Ipv4Subnet;
use crate::ipv6::Ipv6Subnet;
use serde::Serialize;
use std::net::{Ipv4Addr, Ipv6Addr};

// ---------------------------------------------------------------------------
// Result structs
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Ipv4SummaryResult {
    pub input_count: usize,
    pub output_count: usize,
    pub cidrs: Vec<Ipv4Subnet>,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Ipv6SummaryResult {
    pub input_count: usize,
    pub output_count: usize,
    pub cidrs: Vec<Ipv6Subnet>,
}

// ---------------------------------------------------------------------------
// Generic summarization algorithm over (network, prefix) pairs
// ---------------------------------------------------------------------------

fn normalize_and_sort(entries: &mut Vec<(u128, u8)>, bits: u8) {
    // Normalize: zero host bits
    for entry in entries.iter_mut() {
        let mask = if entry.1 == 0 {
            0u128
        } else if bits == 32 {
            (!0u32 << (32 - entry.1)) as u128
        } else {
            !0u128 << (128 - entry.1)
        };
        entry.0 &= mask;
    }

    // Sort by (network asc, prefix asc)
    entries.sort();

    // Dedup exact duplicates
    entries.dedup();
}

fn remove_contained(entries: &mut Vec<(u128, u8)>, bits: u8) {
    if entries.is_empty() {
        return;
    }

    let mut kept: Vec<(u128, u8)> = Vec::with_capacity(entries.len());
    kept.push(entries[0]);

    for &entry in &entries[1..] {
        let last = kept.last().unwrap();
        // Check if entry is contained in last kept entry
        let mask = if last.1 == 0 {
            0u128
        } else if bits == 32 {
            (!0u32 << (32 - last.1)) as u128
        } else {
            !0u128 << (128 - last.1)
        };

        if entry.1 >= last.1 && (entry.0 & mask) == last.0 {
            // entry is contained in last, skip
            continue;
        }
        kept.push(entry);
    }

    *entries = kept;
}

fn merge_siblings(entries: &mut Vec<(u128, u8)>, bits: u8) {
    loop {
        let mut merged = false;
        let mut result: Vec<(u128, u8)> = Vec::with_capacity(entries.len());
        let mut i = 0;

        while i < entries.len() {
            if i + 1 < entries.len() {
                let (net_a, pfx_a) = entries[i];
                let (net_b, pfx_b) = entries[i + 1];

                if pfx_a == pfx_b && pfx_a > 0 {
                    let parent_prefix = pfx_a - 1;
                    let shift = bits - parent_prefix;

                    let parent_a = if shift >= 128 { 0 } else { net_a >> shift };
                    let parent_b = if shift >= 128 { 0 } else { net_b >> shift };

                    if parent_a == parent_b {
                        // Merge into parent
                        let parent_mask = if parent_prefix == 0 {
                            0u128
                        } else if bits == 32 {
                            (!0u32 << (32 - parent_prefix)) as u128
                        } else {
                            !0u128 << (128 - parent_prefix)
                        };
                        result.push((net_a & parent_mask, parent_prefix));
                        merged = true;
                        i += 2;
                        continue;
                    }
                }
            }
            result.push(entries[i]);
            i += 1;
        }

        *entries = result;

        if !merged {
            break;
        }

        // After merging, we may have new containment or new siblings, so re-sort and re-clean
        entries.sort();
        entries.dedup();
        remove_contained(entries, bits);
    }
}

fn summarize_entries(entries: &mut Vec<(u128, u8)>, bits: u8) {
    if entries.is_empty() {
        return;
    }
    normalize_and_sort(entries, bits);
    remove_contained(entries, bits);
    merge_siblings(entries, bits);
}

pub const DEFAULT_MAX_SUMMARIZE_INPUTS: usize = 10_000;

// ---------------------------------------------------------------------------
// Public entry points
// ---------------------------------------------------------------------------

pub fn summarize_ipv4(cidrs: &[String]) -> Result<Ipv4SummaryResult> {
    summarize_ipv4_with_limit(cidrs, DEFAULT_MAX_SUMMARIZE_INPUTS)
}

pub fn summarize_ipv4_with_limit(cidrs: &[String], max_inputs: usize) -> Result<Ipv4SummaryResult> {
    if cidrs.is_empty() {
        return Err(IpCalcError::EmptyCidrList);
    }
    if cidrs.len() > max_inputs {
        return Err(IpCalcError::SummarizeInputLimitExceeded {
            count: cidrs.len(),
            limit: max_inputs,
        });
    }

    let input_count = cidrs.len();

    // Parse and validate all CIDRs, extract (network_u32, prefix) pairs
    let mut entries: Vec<(u128, u8)> = Vec::with_capacity(cidrs.len());
    for cidr in cidrs {
        let subnet = Ipv4Subnet::from_cidr(cidr)?;
        let network_u32 = u32::from(subnet.network_addr());
        entries.push((network_u32 as u128, subnet.prefix_length));
    }

    summarize_entries(&mut entries, 32);

    // Reconstruct Ipv4Subnet from results
    let mut result_cidrs = Vec::with_capacity(entries.len());
    for (network, prefix) in &entries {
        let addr = Ipv4Addr::from(*network as u32);
        let subnet = Ipv4Subnet::new(addr, *prefix)?;
        result_cidrs.push(subnet);
    }

    Ok(Ipv4SummaryResult {
        input_count,
        output_count: result_cidrs.len(),
        cidrs: result_cidrs,
    })
}

pub fn summarize_ipv6(cidrs: &[String]) -> Result<Ipv6SummaryResult> {
    summarize_ipv6_with_limit(cidrs, DEFAULT_MAX_SUMMARIZE_INPUTS)
}

pub fn summarize_ipv6_with_limit(cidrs: &[String], max_inputs: usize) -> Result<Ipv6SummaryResult> {
    if cidrs.is_empty() {
        return Err(IpCalcError::EmptyCidrList);
    }
    if cidrs.len() > max_inputs {
        return Err(IpCalcError::SummarizeInputLimitExceeded {
            count: cidrs.len(),
            limit: max_inputs,
        });
    }

    let input_count = cidrs.len();

    let mut entries: Vec<(u128, u8)> = Vec::with_capacity(cidrs.len());
    for cidr in cidrs {
        let subnet = Ipv6Subnet::from_cidr(cidr)?;
        let network_u128 = u128::from(subnet.network_addr());
        entries.push((network_u128, subnet.prefix_length));
    }

    summarize_entries(&mut entries, 128);

    let mut result_cidrs = Vec::with_capacity(entries.len());
    for (network, prefix) in &entries {
        let addr = Ipv6Addr::from(*network);
        let subnet = Ipv6Subnet::new(addr, *prefix)?;
        result_cidrs.push(subnet);
    }

    Ok(Ipv6SummaryResult {
        input_count,
        output_count: result_cidrs.len(),
        cidrs: result_cidrs,
    })
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adjacent_merge_ipv4() {
        let result =
            summarize_ipv4(&["192.168.0.0/24".to_string(), "192.168.1.0/24".to_string()]).unwrap();
        assert_eq!(result.input_count, 2);
        assert_eq!(result.output_count, 1);
        assert_eq!(result.cidrs[0].network_address, "192.168.0.0");
        assert_eq!(result.cidrs[0].prefix_length, 23);
    }

    #[test]
    fn test_containment_collapse() {
        let result =
            summarize_ipv4(&["10.0.0.0/8".to_string(), "10.1.0.0/16".to_string()]).unwrap();
        assert_eq!(result.output_count, 1);
        assert_eq!(result.cidrs[0].network_address, "10.0.0.0");
        assert_eq!(result.cidrs[0].prefix_length, 8);
    }

    #[test]
    fn test_cascade_merge() {
        // 4x /24 → /22
        let result = summarize_ipv4(&[
            "10.0.0.0/24".to_string(),
            "10.0.1.0/24".to_string(),
            "10.0.2.0/24".to_string(),
            "10.0.3.0/24".to_string(),
        ])
        .unwrap();
        assert_eq!(result.output_count, 1);
        assert_eq!(result.cidrs[0].network_address, "10.0.0.0");
        assert_eq!(result.cidrs[0].prefix_length, 22);
    }

    #[test]
    fn test_duplicates() {
        let result =
            summarize_ipv4(&["192.168.1.0/24".to_string(), "192.168.1.0/24".to_string()]).unwrap();
        assert_eq!(result.output_count, 1);
        assert_eq!(result.cidrs[0].network_address, "192.168.1.0");
        assert_eq!(result.cidrs[0].prefix_length, 24);
    }

    #[test]
    fn test_single_input() {
        let result = summarize_ipv4(&["172.16.0.0/12".to_string()]).unwrap();
        assert_eq!(result.input_count, 1);
        assert_eq!(result.output_count, 1);
        assert_eq!(result.cidrs[0].network_address, "172.16.0.0");
        assert_eq!(result.cidrs[0].prefix_length, 12);
    }

    #[test]
    fn test_non_normalized_input() {
        // 192.168.1.50/24 should normalize to 192.168.1.0/24
        let result = summarize_ipv4(&[
            "192.168.0.50/24".to_string(),
            "192.168.1.100/24".to_string(),
        ])
        .unwrap();
        assert_eq!(result.output_count, 1);
        assert_eq!(result.cidrs[0].network_address, "192.168.0.0");
        assert_eq!(result.cidrs[0].prefix_length, 23);
    }

    #[test]
    fn test_prefix_zero() {
        let result = summarize_ipv4(&["0.0.0.0/0".to_string()]).unwrap();
        assert_eq!(result.output_count, 1);
        assert_eq!(result.cidrs[0].prefix_length, 0);
    }

    #[test]
    fn test_non_adjacent_no_merge() {
        let result =
            summarize_ipv4(&["10.0.0.0/24".to_string(), "10.0.2.0/24".to_string()]).unwrap();
        assert_eq!(result.output_count, 2);
    }

    #[test]
    fn test_ipv6_adjacent_merge() {
        let result =
            summarize_ipv6(&["2001:db8::/48".to_string(), "2001:db8:1::/48".to_string()]).unwrap();
        assert_eq!(result.input_count, 2);
        assert_eq!(result.output_count, 1);
        assert_eq!(result.cidrs[0].network_address, "2001:db8::");
        assert_eq!(result.cidrs[0].prefix_length, 47);
    }

    #[test]
    fn test_empty_input() {
        let result = summarize_ipv4(&[]);
        assert!(result.is_err());
    }

    #[test]
    fn test_summarize_input_limit_exceeded() {
        let cidrs: Vec<String> = (0..5).map(|i| format!("10.{}.0.0/16", i)).collect();
        let result = summarize_ipv4_with_limit(&cidrs, 3);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exceeds maximum"));
    }
}
