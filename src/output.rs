use crate::contains::ContainsResult;
use crate::error::Result;
use crate::ipv4::Ipv4Subnet;
use crate::ipv6::Ipv6Subnet;
use crate::subnet_generator::{Ipv4SubnetList, Ipv6SubnetList, SplitSummary};
use serde::Serialize;
use std::fmt::Write as FmtWrite;
use std::fs::File;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    #[default]
    Json,
    Text,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "text" | "plain" | "txt" => Ok(Self::Text),
            _ => Err(format!("Unknown output format: {}", s)),
        }
    }
}

pub struct OutputWriter {
    format: OutputFormat,
    file_path: Option<String>,
}

impl OutputWriter {
    pub fn new(format: OutputFormat, file_path: Option<String>) -> Self {
        Self { format, file_path }
    }

    pub fn write<T: Serialize + TextOutput>(&self, data: &T) -> Result<String> {
        let output = match self.format {
            OutputFormat::Json => serde_json::to_string_pretty(data)?,
            OutputFormat::Text => data.to_text(),
        };

        if let Some(ref path) = self.file_path {
            let mut file = File::create(Path::new(path))?;
            file.write_all(output.as_bytes())?;
        }

        Ok(output)
    }
}

pub trait TextOutput {
    fn to_text(&self) -> String;
}

impl TextOutput for Ipv4Subnet {
    fn to_text(&self) -> String {
        let mut out = String::new();
        writeln!(out, "IPv4 Subnet Calculator").unwrap();
        writeln!(out, "======================").unwrap();
        writeln!(out, "Input:             {}", self.input).unwrap();
        writeln!(out, "Network Address:   {}", self.network_address).unwrap();
        writeln!(out, "Broadcast Address: {}", self.broadcast_address).unwrap();
        writeln!(out, "Subnet Mask:       {}", self.subnet_mask).unwrap();
        writeln!(out, "Wildcard Mask:     {}", self.wildcard_mask).unwrap();
        writeln!(out, "Prefix Length:     /{}", self.prefix_length).unwrap();
        writeln!(out, "First Host:        {}", self.first_host).unwrap();
        writeln!(out, "Last Host:         {}", self.last_host).unwrap();
        writeln!(out, "Total Hosts:       {}", self.total_hosts).unwrap();
        writeln!(out, "Usable Hosts:      {}", self.usable_hosts).unwrap();
        writeln!(out, "Network Class:     {}", self.network_class).unwrap();
        writeln!(
            out,
            "Private Address:   {}",
            if self.is_private { "Yes" } else { "No" }
        )
        .unwrap();
        out
    }
}

impl TextOutput for Ipv6Subnet {
    fn to_text(&self) -> String {
        let mut out = String::new();
        writeln!(out, "IPv6 Subnet Calculator").unwrap();
        writeln!(out, "======================").unwrap();
        writeln!(out, "Input:               {}", self.input).unwrap();
        writeln!(out, "Network Address:     {}", self.network_address).unwrap();
        writeln!(out, "Network (Full):      {}", self.network_address_full).unwrap();
        writeln!(out, "Last Address:        {}", self.last_address).unwrap();
        writeln!(out, "Last Address (Full): {}", self.last_address_full).unwrap();
        writeln!(out, "Prefix Length:       /{}", self.prefix_length).unwrap();
        writeln!(out, "Total Addresses:     {}", self.total_addresses).unwrap();
        writeln!(out, "Hextets:             {}", self.hextets.join(":")).unwrap();
        writeln!(out, "Address Type:        {}", self.address_type).unwrap();
        out
    }
}

impl TextOutput for ContainsResult {
    fn to_text(&self) -> String {
        let mut out = String::new();
        writeln!(out, "Address Containment Check").unwrap();
        writeln!(out, "=========================").unwrap();
        writeln!(out, "Subnet:            {}", self.cidr).unwrap();
        writeln!(out, "Address:           {}", self.address).unwrap();
        writeln!(
            out,
            "Contained:         {}",
            if self.contained { "Yes" } else { "No" }
        )
        .unwrap();
        writeln!(out, "Network Address:   {}", self.network_address).unwrap();
        writeln!(out, "Broadcast Address: {}", self.broadcast_address).unwrap();
        out
    }
}

impl TextOutput for Ipv4SubnetList {
    fn to_text(&self) -> String {
        let mut out = String::new();
        writeln!(out, "IPv4 Subnet Generator").unwrap();
        writeln!(out, "=====================").unwrap();
        writeln!(out, "Supernet: {}", self.supernet.input).unwrap();
        writeln!(out, "New Prefix: /{}", self.new_prefix).unwrap();
        writeln!(out, "Generated {} subnets:\n", self.requested_count).unwrap();

        for (i, subnet) in self.subnets.iter().enumerate() {
            writeln!(
                out,
                "  {}. {}/{} (Hosts: {}-{})",
                i + 1,
                subnet.network_address,
                subnet.prefix_length,
                subnet.first_host,
                subnet.last_host
            )
            .unwrap();
        }
        out
    }
}

impl TextOutput for Ipv6SubnetList {
    fn to_text(&self) -> String {
        let mut out = String::new();
        writeln!(out, "IPv6 Subnet Generator").unwrap();
        writeln!(out, "=====================").unwrap();
        writeln!(out, "Supernet: {}", self.supernet.input).unwrap();
        writeln!(out, "New Prefix: /{}", self.new_prefix).unwrap();
        writeln!(out, "Generated {} subnets:\n", self.requested_count).unwrap();

        for (i, subnet) in self.subnets.iter().enumerate() {
            writeln!(
                out,
                "  {}. {}/{}",
                i + 1,
                subnet.network_address,
                subnet.prefix_length
            )
            .unwrap();
        }
        out
    }
}

impl TextOutput for SplitSummary {
    fn to_text(&self) -> String {
        let mut out = String::new();
        writeln!(out, "Subnet Split Summary").unwrap();
        writeln!(out, "====================").unwrap();
        writeln!(out, "Supernet:           {}", self.supernet).unwrap();
        writeln!(out, "New Prefix:         /{}", self.new_prefix).unwrap();
        writeln!(out, "Available Subnets:  {}", self.available_subnets).unwrap();
        out
    }
}
