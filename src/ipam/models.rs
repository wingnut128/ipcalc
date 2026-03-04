use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Supernet
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supernet {
    pub id: String,
    pub cidr: String,
    pub network_address: String,
    pub broadcast_address: String,
    pub prefix_length: u8,
    pub total_hosts: u128,
    pub name: Option<String>,
    pub description: Option<String>,
    pub ip_version: u8,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSupernet {
    pub cidr: String,
    pub name: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupernetList {
    pub supernets: Vec<Supernet>,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Allocation
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AllocationStatus {
    Active,
    Reserved,
    Released,
}

impl std::fmt::Display for AllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Active => write!(f, "active"),
            Self::Reserved => write!(f, "reserved"),
            Self::Released => write!(f, "released"),
        }
    }
}

impl std::str::FromStr for AllocationStatus {
    type Err = String;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "reserved" => Ok(Self::Reserved),
            "released" => Ok(Self::Released),
            other => Err(format!("invalid allocation status: {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Allocation {
    pub id: String,
    pub supernet_id: String,
    pub cidr: String,
    pub network_address: String,
    pub broadcast_address: String,
    pub prefix_length: u8,
    pub total_hosts: u128,
    pub status: AllocationStatus,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub environment: Option<String>,
    pub owner: Option<String>,
    pub parent_allocation_id: Option<String>,
    pub tags: Vec<Tag>,
    pub created_at: String,
    pub updated_at: String,
    pub released_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateAllocation {
    pub supernet_id: String,
    pub cidr: String,
    pub status: Option<AllocationStatus>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub environment: Option<String>,
    pub owner: Option<String>,
    pub parent_allocation_id: Option<String>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AutoAllocateRequest {
    pub supernet_id: String,
    pub prefix_length: u8,
    pub count: Option<u32>,
    pub status: Option<AllocationStatus>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub environment: Option<String>,
    pub owner: Option<String>,
    pub parent_allocation_id: Option<String>,
    pub tags: Option<Vec<Tag>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateAllocation {
    pub name: Option<String>,
    pub description: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub environment: Option<String>,
    pub owner: Option<String>,
    pub status: Option<AllocationStatus>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AllocationFilter {
    pub supernet_id: Option<String>,
    pub status: Option<AllocationStatus>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub environment: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationList {
    pub allocations: Vec<Allocation>,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Tags
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub key: String,
    pub value: String,
}

// ---------------------------------------------------------------------------
// Audit
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub action: String,
    pub details: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct AuditFilter {
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub action: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditList {
    pub entries: Vec<AuditEntry>,
    pub count: usize,
}

// ---------------------------------------------------------------------------
// Utilization / Free Space reports
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationReport {
    pub supernet_id: String,
    pub supernet_cidr: String,
    pub total_addresses: u128,
    pub allocated_addresses: u128,
    pub free_addresses: u128,
    pub utilization_percent: f64,
    pub allocation_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeBlock {
    pub cidr: String,
    pub size: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreeBlocksReport {
    pub supernet_id: String,
    pub supernet_cidr: String,
    pub blocks: Vec<FreeBlock>,
    pub total_free: u128,
}
