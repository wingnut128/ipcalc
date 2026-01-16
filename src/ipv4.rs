use crate::error::{IpCalcError, Result};
use serde::Serialize;
use std::net::Ipv4Addr;
use std::str::FromStr;

#[derive(Debug, Clone, Serialize)]
pub struct Ipv4Subnet {
    pub input: String,
    pub network_address: String,
    pub broadcast_address: String,
    pub subnet_mask: String,
    pub wildcard_mask: String,
    pub prefix_length: u8,
    pub first_host: String,
    pub last_host: String,
    pub total_hosts: u64,
    pub usable_hosts: u64,
    pub network_class: String,
    pub is_private: bool,
}

impl Ipv4Subnet {
    pub fn from_cidr(cidr: &str) -> Result<Self> {
        let parts: Vec<&str> = cidr.split('/').collect();
        if parts.len() != 2 {
            return Err(IpCalcError::InvalidCidr(cidr.to_string()));
        }

        let addr = Ipv4Addr::from_str(parts[0])
            .map_err(|_| IpCalcError::InvalidIpv4Address(parts[0].to_string()))?;

        let prefix: u8 = parts[1]
            .parse()
            .map_err(|_| IpCalcError::InvalidCidr(cidr.to_string()))?;

        Self::new(addr, prefix)
    }

    pub fn new(addr: Ipv4Addr, prefix: u8) -> Result<Self> {
        if prefix > 32 {
            return Err(IpCalcError::InvalidPrefixLength(prefix));
        }

        let addr_u32 = u32::from(addr);
        let mask = if prefix == 0 {
            0
        } else {
            !0u32 << (32 - prefix)
        };
        let wildcard = !mask;

        let network = addr_u32 & mask;
        let broadcast = network | wildcard;

        let network_addr = Ipv4Addr::from(network);
        let broadcast_addr = Ipv4Addr::from(broadcast);
        let subnet_mask = Ipv4Addr::from(mask);
        let wildcard_mask = Ipv4Addr::from(wildcard);

        let total_hosts = if prefix == 32 {
            1
        } else {
            2u64.pow((32 - prefix) as u32)
        };
        let usable_hosts = if prefix >= 31 {
            total_hosts
        } else {
            total_hosts.saturating_sub(2)
        };

        let (first_host, last_host) = if prefix >= 31 {
            (network_addr, broadcast_addr)
        } else {
            (Ipv4Addr::from(network + 1), Ipv4Addr::from(broadcast - 1))
        };

        let first_octet = addr.octets()[0];
        let network_class = match first_octet {
            0..=127 => "A",
            128..=191 => "B",
            192..=223 => "C",
            224..=239 => "D (Multicast)",
            240..=255 => "E (Reserved)",
        }
        .to_string();

        let is_private = addr.is_private()
            || (addr.octets()[0] == 100 && (addr.octets()[1] & 0xC0) == 64) // 100.64.0.0/10
            || addr.is_loopback()
            || addr.is_link_local();

        Ok(Self {
            input: format!("{}/{}", addr, prefix),
            network_address: network_addr.to_string(),
            broadcast_address: broadcast_addr.to_string(),
            subnet_mask: subnet_mask.to_string(),
            wildcard_mask: wildcard_mask.to_string(),
            prefix_length: prefix,
            first_host: first_host.to_string(),
            last_host: last_host.to_string(),
            total_hosts,
            usable_hosts,
            network_class,
            is_private,
        })
    }

    pub fn network_addr(&self) -> Ipv4Addr {
        Ipv4Addr::from_str(&self.network_address).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ipv4_subnet_24() {
        let subnet = Ipv4Subnet::from_cidr("192.168.1.100/24").unwrap();
        assert_eq!(subnet.network_address, "192.168.1.0");
        assert_eq!(subnet.broadcast_address, "192.168.1.255");
        assert_eq!(subnet.subnet_mask, "255.255.255.0");
        assert_eq!(subnet.wildcard_mask, "0.0.0.255");
        assert_eq!(subnet.first_host, "192.168.1.1");
        assert_eq!(subnet.last_host, "192.168.1.254");
        assert_eq!(subnet.total_hosts, 256);
        assert_eq!(subnet.usable_hosts, 254);
    }

    #[test]
    fn test_ipv4_subnet_32() {
        let subnet = Ipv4Subnet::from_cidr("10.0.0.1/32").unwrap();
        assert_eq!(subnet.network_address, "10.0.0.1");
        assert_eq!(subnet.broadcast_address, "10.0.0.1");
        assert_eq!(subnet.total_hosts, 1);
        assert_eq!(subnet.usable_hosts, 1);
    }

    #[test]
    fn test_ipv4_subnet_31() {
        let subnet = Ipv4Subnet::from_cidr("10.0.0.0/31").unwrap();
        assert_eq!(subnet.network_address, "10.0.0.0");
        assert_eq!(subnet.broadcast_address, "10.0.0.1");
        assert_eq!(subnet.total_hosts, 2);
        assert_eq!(subnet.usable_hosts, 2);
    }

    #[test]
    fn test_private_address() {
        let subnet = Ipv4Subnet::from_cidr("192.168.1.0/24").unwrap();
        assert!(subnet.is_private);

        let subnet = Ipv4Subnet::from_cidr("8.8.8.0/24").unwrap();
        assert!(!subnet.is_private);
    }

    #[test]
    fn test_invalid_prefix() {
        let result = Ipv4Subnet::from_cidr("192.168.1.0/33");
        assert!(result.is_err());
    }
}
