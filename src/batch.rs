use crate::error::{IpCalcError, Result};
use crate::ipv4::Ipv4Subnet;
use crate::ipv6::Ipv6Subnet;
use serde::Serialize;

/// A subnet calculation result that can be either IPv4 or IPv6.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "version")]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub enum SubnetResult {
    #[serde(rename = "v4")]
    V4(Ipv4Subnet),
    #[serde(rename = "v6")]
    V6(Ipv6Subnet),
}

/// The result for a single CIDR entry in a batch — either a subnet or an error message.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub enum BatchEntryResult {
    Ok { subnet: Box<SubnetResult> },
    Err { error: String },
}

/// A single entry in a batch result, pairing the input CIDR with its result.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct BatchEntry {
    pub cidr: String,
    #[serde(flatten)]
    pub result: BatchEntryResult,
}

/// The top-level result of processing a batch of CIDRs.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct BatchResult {
    pub count: usize,
    pub results: Vec<BatchEntry>,
}

/// Process a batch of CIDR strings, auto-detecting IPv4 vs IPv6 per entry.
///
/// Returns `EmptyCidrList` if the input slice is empty. Individual parsing
/// errors are captured per-entry rather than aborting the entire batch.
pub fn process_batch(cidrs: &[String]) -> Result<BatchResult> {
    if cidrs.is_empty() {
        return Err(IpCalcError::EmptyCidrList);
    }

    let results: Vec<BatchEntry> = cidrs
        .iter()
        .map(|raw| {
            let cidr = raw.trim().to_string();
            let is_ipv6 = cidr.contains(':');
            let result = if is_ipv6 {
                match Ipv6Subnet::from_cidr(&cidr) {
                    Ok(subnet) => BatchEntryResult::Ok {
                        subnet: Box::new(SubnetResult::V6(subnet)),
                    },
                    Err(e) => BatchEntryResult::Err {
                        error: e.to_string(),
                    },
                }
            } else {
                match Ipv4Subnet::from_cidr(&cidr) {
                    Ok(subnet) => BatchEntryResult::Ok {
                        subnet: Box::new(SubnetResult::V4(subnet)),
                    },
                    Err(e) => BatchEntryResult::Err {
                        error: e.to_string(),
                    },
                }
            };
            BatchEntry { cidr, result }
        })
        .collect();

    Ok(BatchResult {
        count: results.len(),
        results,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_single_v4() {
        let cidrs = vec!["192.168.1.0/24".to_string()];
        let result = process_batch(&cidrs).unwrap();
        assert_eq!(result.count, 1);
        assert_eq!(result.results[0].cidr, "192.168.1.0/24");
        match &result.results[0].result {
            BatchEntryResult::Ok { subnet } => match subnet.as_ref() {
                SubnetResult::V4(s) => assert_eq!(s.network_address, "192.168.1.0"),
                SubnetResult::V6(_) => panic!("expected v4"),
            },
            BatchEntryResult::Err { .. } => panic!("expected Ok"),
        }
    }

    #[test]
    fn test_batch_single_v6() {
        let cidrs = vec!["2001:db8::/32".to_string()];
        let result = process_batch(&cidrs).unwrap();
        assert_eq!(result.count, 1);
        match &result.results[0].result {
            BatchEntryResult::Ok { subnet } => match subnet.as_ref() {
                SubnetResult::V6(s) => assert_eq!(s.network_address, "2001:db8::"),
                SubnetResult::V4(_) => panic!("expected v6"),
            },
            BatchEntryResult::Err { .. } => panic!("expected Ok"),
        }
    }

    #[test]
    fn test_batch_mixed() {
        let cidrs = vec!["192.168.1.0/24".to_string(), "2001:db8::/32".to_string()];
        let result = process_batch(&cidrs).unwrap();
        assert_eq!(result.count, 2);
        match &result.results[0].result {
            BatchEntryResult::Ok { subnet } => {
                assert!(matches!(subnet.as_ref(), SubnetResult::V4(_)))
            }
            _ => panic!("expected Ok"),
        }
        match &result.results[1].result {
            BatchEntryResult::Ok { subnet } => {
                assert!(matches!(subnet.as_ref(), SubnetResult::V6(_)))
            }
            _ => panic!("expected Ok"),
        }
    }

    #[test]
    fn test_batch_with_invalid() {
        let cidrs = vec![
            "192.168.1.0/24".to_string(),
            "not-a-cidr".to_string(),
            "10.0.0.0/8".to_string(),
        ];
        let result = process_batch(&cidrs).unwrap();
        assert_eq!(result.count, 3);
        assert!(matches!(
            &result.results[0].result,
            BatchEntryResult::Ok { .. }
        ));
        assert!(matches!(
            &result.results[1].result,
            BatchEntryResult::Err { .. }
        ));
        assert!(matches!(
            &result.results[2].result,
            BatchEntryResult::Ok { .. }
        ));
    }

    #[test]
    fn test_batch_empty() {
        let cidrs: Vec<String> = vec![];
        let result = process_batch(&cidrs);
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_whitespace_trimming() {
        let cidrs = vec!["  192.168.1.0/24  ".to_string()];
        let result = process_batch(&cidrs).unwrap();
        assert_eq!(result.count, 1);
        assert!(matches!(
            &result.results[0].result,
            BatchEntryResult::Ok { .. }
        ));
    }
}
