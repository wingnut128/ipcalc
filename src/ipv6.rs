use crate::error::{IpCalcError, Result};
use serde::Serialize;
use std::net::Ipv6Addr;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "swagger", derive(utoipa::ToSchema))]
pub struct Ipv6Subnet {
    pub input: String,
    pub network_address: String,
    pub network_address_full: String,
    pub last_address: String,
    pub last_address_full: String,
    pub prefix_length: u8,
    pub total_addresses: String,
    pub hextets: Vec<String>,
    pub address_type: String,
}

const MAX_INPUT_LENGTH: usize = 256;

impl Ipv6Subnet {
    pub fn from_cidr(cidr: &str) -> Result<Self> {
        if cidr.len() > MAX_INPUT_LENGTH {
            return Err(IpCalcError::InputTooLong {
                length: cidr.len(),
                limit: MAX_INPUT_LENGTH,
            });
        }
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err(IpCalcError::InvalidCidr(cidr.to_string()));
        }

        let addr = Ipv6Addr::from_str(parts[0])
            .map_err(|_| IpCalcError::InvalidIpv6Address(parts[0].to_string()))?;

        let prefix: u8 = parts[1]
            .parse()
            .map_err(|_| IpCalcError::InvalidCidr(cidr.to_string()))?;

        Self::new(addr, prefix)
    }

    pub fn new(addr: Ipv6Addr, prefix: u8) -> Result<Self> {
        if prefix > 128 {
            return Err(IpCalcError::InvalidPrefixLength(prefix));
        }

        let addr_u128 = u128::from(addr);
        let mask = if prefix == 0 {
            0
        } else {
            !0u128 << (128 - prefix)
        };

        let network = addr_u128 & mask;
        let last = network | !mask;

        let network_addr = Ipv6Addr::from(network);
        let last_addr = Ipv6Addr::from(last);

        let segments = network_addr.segments();
        let hextets: Vec<String> = segments.iter().map(|s| format!("{:04x}", s)).collect();

        let total_addresses = if prefix == 128 {
            "1".to_string()
        } else {
            let bits = 128 - prefix;
            if bits <= 64 {
                format!("{}", 2u128.pow(bits as u32))
            } else {
                format!("2^{}", bits)
            }
        };

        let address_type = Self::determine_address_type(&network_addr);

        Ok(Self {
            input: format!("{}/{}", addr, prefix),
            network_address: network_addr.to_string(),
            network_address_full: hextets.join(":"),
            last_address: last_addr.to_string(),
            last_address_full: Self::format_full(&last_addr),
            prefix_length: prefix,
            total_addresses,
            hextets,
            address_type,
        })
    }

    fn format_full(addr: &Ipv6Addr) -> String {
        addr.segments()
            .iter()
            .map(|s| format!("{:04x}", s))
            .collect::<Vec<_>>()
            .join(":")
    }

    fn determine_address_type(addr: &Ipv6Addr) -> String {
        if addr.is_loopback() {
            "Loopback (RFC 4291)".to_string()
        } else if addr.is_unspecified() {
            "Unspecified (RFC 4291)".to_string()
        } else if addr.is_multicast() {
            "Multicast (RFC 4291)".to_string()
        } else if Self::is_link_local(addr) {
            "Link-Local Unicast (RFC 4291)".to_string()
        } else if Self::is_unique_local(addr) {
            "Unique Local Address (RFC 4193)".to_string()
        } else if Self::is_documentation(addr) {
            "Documentation (RFC 3849)".to_string()
        } else if Self::is_global_unicast(addr) {
            "Global Unicast (RFC 4291)".to_string()
        } else {
            "Other".to_string()
        }
    }

    fn is_documentation(addr: &Ipv6Addr) -> bool {
        let segments = addr.segments();
        segments[0] == 0x2001 && segments[1] == 0x0db8
    }

    fn is_link_local(addr: &Ipv6Addr) -> bool {
        let segments = addr.segments();
        (segments[0] & 0xffc0) == 0xfe80
    }

    fn is_unique_local(addr: &Ipv6Addr) -> bool {
        let segments = addr.segments();
        (segments[0] & 0xfe00) == 0xfc00
    }

    fn is_global_unicast(addr: &Ipv6Addr) -> bool {
        let segments = addr.segments();
        (segments[0] & 0xe000) == 0x2000
    }

    pub fn network_addr(&self) -> Ipv6Addr {
        Ipv6Addr::from_str(&self.network_address).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv6_subnet_64() {
        let subnet = Ipv6Subnet::from_cidr("2001:db8:85a3::8a2e:370:7334/64").unwrap();
        assert_eq!(subnet.network_address, "2001:db8:85a3::");
        assert_eq!(
            subnet.network_address_full,
            "2001:0db8:85a3:0000:0000:0000:0000:0000"
        );
        assert_eq!(subnet.prefix_length, 64);
        assert_eq!(subnet.address_type, "Documentation (RFC 3849)");
    }

    #[test]
    fn test_ipv6_subnet_128() {
        let subnet = Ipv6Subnet::from_cidr("::1/128").unwrap();
        assert_eq!(subnet.network_address, "::1");
        assert_eq!(subnet.total_addresses, "1");
        assert_eq!(subnet.address_type, "Loopback (RFC 4291)");
    }

    #[test]
    fn test_ipv6_link_local() {
        let subnet = Ipv6Subnet::from_cidr("fe80::1/10").unwrap();
        assert_eq!(subnet.address_type, "Link-Local Unicast (RFC 4291)");
    }

    #[test]
    fn test_ipv6_ula() {
        let subnet = Ipv6Subnet::from_cidr("fd00::1/8").unwrap();
        assert_eq!(subnet.address_type, "Unique Local Address (RFC 4193)");
    }

    #[test]
    fn test_ipv6_documentation() {
        let subnet = Ipv6Subnet::from_cidr("2001:db8::/32").unwrap();
        assert_eq!(subnet.address_type, "Documentation (RFC 3849)");
    }

    #[test]
    fn test_ipv6_global_unicast() {
        let subnet = Ipv6Subnet::from_cidr("2001:4860::/32").unwrap();
        assert_eq!(subnet.address_type, "Global Unicast (RFC 4291)");
    }

    #[test]
    fn test_invalid_prefix() {
        let result = Ipv6Subnet::from_cidr("2001:db8::/129");
        assert!(result.is_err());
    }

    #[test]
    fn test_input_too_long() {
        let long_input = "a".repeat(300);
        let result = Ipv6Subnet::from_cidr(&long_input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("exceeds maximum length"));
    }
}
