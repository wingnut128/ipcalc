use crate::error::{IpCalcError, Result};
use crate::ipv4::Ipv4Subnet;
use crate::ipv6::Ipv6Subnet;
use serde::Serialize;
use std::net::{Ipv4Addr, Ipv6Addr};

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Ipv4SubnetList {
    pub supernet: Ipv4Subnet,
    pub new_prefix: u8,
    pub requested_count: u64,
    pub subnets: Vec<Ipv4Subnet>,
}

#[derive(Debug, Serialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Ipv6SubnetList {
    pub supernet: Ipv6Subnet,
    pub new_prefix: u8,
    pub requested_count: u64,
    pub subnets: Vec<Ipv6Subnet>,
}

/// Generate IPv4 subnets from a supernet.
/// If count is None, generates the maximum number of subnets possible.
pub fn generate_ipv4_subnets(
    cidr: &str,
    new_prefix: u8,
    count: Option<u64>,
) -> Result<Ipv4SubnetList> {
    let supernet = Ipv4Subnet::from_cidr(cidr)?;

    if new_prefix <= supernet.prefix_length {
        return Err(IpCalcError::InvalidSubnetSplit {
            new_prefix,
            original_prefix: supernet.prefix_length,
        });
    }

    if new_prefix > 32 {
        return Err(IpCalcError::InvalidPrefixLength(new_prefix));
    }

    let bits_diff = new_prefix - supernet.prefix_length;
    let available: u64 = 2u64.pow(bits_diff as u32);

    // Use provided count or maximum available
    let actual_count = match count {
        Some(c) => {
            if c > available {
                return Err(IpCalcError::InsufficientSubnets {
                    requested: c,
                    available,
                    new_prefix,
                    original_prefix: supernet.prefix_length,
                });
            }
            c
        }
        None => available,
    };

    let network_u32 = u32::from(supernet.network_addr());
    let subnet_size = 2u32.pow((32 - new_prefix) as u32);

    let subnets: Result<Vec<Ipv4Subnet>> = (0..actual_count)
        .map(|i| {
            let subnet_network = network_u32 + (i as u32 * subnet_size);
            let addr = Ipv4Addr::from(subnet_network);
            Ipv4Subnet::new(addr, new_prefix)
        })
        .collect();

    Ok(Ipv4SubnetList {
        supernet,
        new_prefix,
        requested_count: actual_count,
        subnets: subnets?,
    })
}

/// Generate IPv6 subnets from a supernet.
/// If count is None, generates the maximum number of subnets possible.
pub fn generate_ipv6_subnets(
    cidr: &str,
    new_prefix: u8,
    count: Option<u64>,
) -> Result<Ipv6SubnetList> {
    let supernet = Ipv6Subnet::from_cidr(cidr)?;

    if new_prefix <= supernet.prefix_length {
        return Err(IpCalcError::InvalidSubnetSplit {
            new_prefix,
            original_prefix: supernet.prefix_length,
        });
    }

    if new_prefix > 128 {
        return Err(IpCalcError::InvalidPrefixLength(new_prefix));
    }

    let bits_diff = new_prefix - supernet.prefix_length;

    // For IPv6, we need to handle larger ranges
    let available: u64 = if bits_diff >= 64 {
        u64::MAX
    } else {
        2u64.pow(bits_diff as u32)
    };

    // Use provided count or maximum available
    let actual_count = match count {
        Some(c) => {
            if c > available {
                return Err(IpCalcError::InsufficientSubnets {
                    requested: c,
                    available,
                    new_prefix,
                    original_prefix: supernet.prefix_length,
                });
            }
            c
        }
        None => available,
    };

    let network_u128 = u128::from(supernet.network_addr());
    let subnet_size: u128 = if new_prefix == 128 {
        1
    } else {
        1u128 << (128 - new_prefix)
    };

    let subnets: Result<Vec<Ipv6Subnet>> = (0..actual_count)
        .map(|i| {
            let subnet_network = network_u128 + (i as u128 * subnet_size);
            let addr = Ipv6Addr::from(subnet_network);
            Ipv6Subnet::new(addr, new_prefix)
        })
        .collect();

    Ok(Ipv6SubnetList {
        supernet,
        new_prefix,
        requested_count: actual_count,
        subnets: subnets?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_ipv4_subnets() {
        let result = generate_ipv4_subnets("192.168.0.0/22", 27, Some(10)).unwrap();
        assert_eq!(result.subnets.len(), 10);
        assert_eq!(result.subnets[0].network_address, "192.168.0.0");
        assert_eq!(result.subnets[0].prefix_length, 27);
        assert_eq!(result.subnets[1].network_address, "192.168.0.32");
        assert_eq!(result.subnets[9].network_address, "192.168.1.32");
    }

    #[test]
    fn test_generate_ipv4_subnets_with_count() {
        // /22 can fit 32 /27 subnets (2^5)
        let result = generate_ipv4_subnets("192.168.0.0/22", 27, Some(32)).unwrap();
        assert_eq!(result.subnets.len(), 32);
    }

    #[test]
    fn test_generate_ipv4_subnets_max() {
        // /22 can fit 32 /27 subnets (2^5), None means generate all
        let result = generate_ipv4_subnets("192.168.0.0/22", 27, None).unwrap();
        assert_eq!(result.subnets.len(), 32);
        assert_eq!(result.requested_count, 32);
    }

    #[test]
    fn test_generate_ipv4_subnets_too_many() {
        // /22 can only fit 32 /27 subnets
        let result = generate_ipv4_subnets("192.168.0.0/22", 27, Some(33));
        assert!(result.is_err());
    }

    #[test]
    fn test_generate_ipv6_subnets() {
        let result = generate_ipv6_subnets("2001:db8::/32", 48, Some(5)).unwrap();
        assert_eq!(result.subnets.len(), 5);
        assert_eq!(result.subnets[0].prefix_length, 48);
    }

    #[test]
    fn test_generate_ipv6_subnets_max() {
        // /48 to /56 is 8 bits difference, so 256 subnets
        let result = generate_ipv6_subnets("2001:db8:abcd::/48", 56, None).unwrap();
        assert_eq!(result.subnets.len(), 256);
        assert_eq!(result.requested_count, 256);
    }

    #[test]
    fn test_invalid_new_prefix_smaller() {
        let result = generate_ipv4_subnets("192.168.0.0/24", 22, Some(1));
        assert!(result.is_err());
    }
}
