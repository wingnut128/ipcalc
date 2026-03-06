use std::sync::Arc;

use axum::{
    Extension, Router,
    extract::{Path, Query},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::{get, post, put},
};
use serde::Deserialize;

use crate::error::IpCalcError;
use crate::ipam::models::*;
use crate::ipam::operations::IpamOps;

// ---------------------------------------------------------------------------
// Error mapping
// ---------------------------------------------------------------------------

fn ipam_error_response(err: IpCalcError) -> Response {
    let status = match &err {
        IpCalcError::InvalidCidr(_)
        | IpCalcError::InvalidPrefixLength { .. }
        | IpCalcError::InvalidInput(_)
        | IpCalcError::InvalidSubnetSplit { .. }
        | IpCalcError::InvalidIpv4Address(_)
        | IpCalcError::InvalidIpv6Address(_) => StatusCode::BAD_REQUEST,

        IpCalcError::SupernetNotFound(_) | IpCalcError::AllocationNotFound(_) => {
            StatusCode::NOT_FOUND
        }

        IpCalcError::AllocationConflict { .. } | IpCalcError::SupernetHasActiveAllocations(_) => {
            StatusCode::CONFLICT
        }

        IpCalcError::NoFreeSpace { .. } => StatusCode::UNPROCESSABLE_ENTITY,

        IpCalcError::DatabaseError(msg)
            if msg.contains("not found") || msg.contains("No supernet") =>
        {
            StatusCode::NOT_FOUND
        }

        IpCalcError::DatabaseError(msg) if msg.contains("overlap") || msg.contains("conflict") => {
            StatusCode::CONFLICT
        }

        _ => StatusCode::INTERNAL_SERVER_ERROR,
    };

    let body = serde_json::json!({ "error": err.to_string() });
    (status, Json(body)).into_response()
}

// ---------------------------------------------------------------------------
// Request/query types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct AllocateSpecificRequest {
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

#[derive(Debug, Deserialize)]
pub struct AutoAllocateBody {
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

#[derive(Debug, Deserialize)]
pub struct AllocationFilterQuery {
    pub status: Option<String>,
    pub resource_id: Option<String>,
    pub resource_type: Option<String>,
    pub environment: Option<String>,
    pub owner: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FreeBlocksQuery {
    pub prefix: Option<u8>,
}

#[derive(Debug, Deserialize)]
pub struct AuditQuery {
    pub entity_type: Option<String>,
    pub entity_id: Option<String>,
    pub action: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct TagsBody {
    pub tags: Vec<Tag>,
}

// ---------------------------------------------------------------------------
// Router
// ---------------------------------------------------------------------------

pub fn create_ipam_router() -> Router {
    Router::new()
        .route(
            "/supernets",
            post(ipam_create_supernet).get(ipam_list_supernets),
        )
        .route(
            "/supernets/{id}",
            get(ipam_get_supernet).delete(ipam_delete_supernet),
        )
        .route("/supernets/{id}/allocate", post(ipam_auto_allocate))
        .route(
            "/supernets/{id}/allocate-specific",
            post(ipam_allocate_specific),
        )
        .route(
            "/supernets/{id}/allocations",
            get(ipam_list_supernet_allocations),
        )
        .route("/supernets/{id}/free", get(ipam_free_blocks))
        .route("/supernets/{id}/utilization", get(ipam_utilization))
        .route(
            "/allocations/{id}",
            get(ipam_get_allocation).patch(ipam_update_allocation),
        )
        .route("/allocations/{id}/release", post(ipam_release_allocation))
        .route("/allocations/{id}/tags", put(ipam_set_tags))
        .route("/find-ip/{address}", get(ipam_find_ip))
        .route("/find-resource/{resource_id}", get(ipam_find_resource))
        .route("/audit", get(ipam_query_audit))
}

// ---------------------------------------------------------------------------
// Handlers
// ---------------------------------------------------------------------------

async fn ipam_create_supernet(
    Extension(ops): Extension<Arc<IpamOps>>,
    Json(body): Json<CreateSupernet>,
) -> impl IntoResponse {
    match ops.create_supernet(&body).await {
        Ok(supernet) => (StatusCode::CREATED, Json(supernet)).into_response(),
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_list_supernets(Extension(ops): Extension<Arc<IpamOps>>) -> impl IntoResponse {
    match ops.list_supernets().await {
        Ok(supernets) => {
            let list = SupernetList {
                count: supernets.len(),
                supernets,
            };
            Json(list).into_response()
        }
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_get_supernet(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match ops.get_supernet(&id).await {
        Ok(supernet) => Json(supernet).into_response(),
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_delete_supernet(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match ops.delete_supernet(&id).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_allocate_specific(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(supernet_id): Path<String>,
    Json(body): Json<AllocateSpecificRequest>,
) -> impl IntoResponse {
    let input = CreateAllocation {
        supernet_id,
        cidr: body.cidr,
        status: body.status,
        resource_id: body.resource_id,
        resource_type: body.resource_type,
        name: body.name,
        description: body.description,
        environment: body.environment,
        owner: body.owner,
        parent_allocation_id: body.parent_allocation_id,
        tags: body.tags,
    };
    match ops.allocate_specific(&input).await {
        Ok(allocation) => (StatusCode::CREATED, Json(allocation)).into_response(),
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_auto_allocate(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(supernet_id): Path<String>,
    Json(body): Json<AutoAllocateBody>,
) -> impl IntoResponse {
    let request = AutoAllocateRequest {
        supernet_id,
        prefix_length: body.prefix_length,
        count: body.count,
        status: body.status,
        resource_id: body.resource_id,
        resource_type: body.resource_type,
        name: body.name,
        description: body.description,
        environment: body.environment,
        owner: body.owner,
        parent_allocation_id: body.parent_allocation_id,
        tags: body.tags,
    };
    match ops.allocate_auto(&request).await {
        Ok(allocations) => {
            let list = AllocationList {
                count: allocations.len(),
                allocations,
            };
            (StatusCode::CREATED, Json(list)).into_response()
        }
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_list_supernet_allocations(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(supernet_id): Path<String>,
    Query(query): Query<AllocationFilterQuery>,
) -> impl IntoResponse {
    let status = query.status.and_then(|s| s.parse().ok());
    let filter = AllocationFilter {
        supernet_id: Some(supernet_id),
        status,
        resource_id: query.resource_id,
        resource_type: query.resource_type,
        environment: query.environment,
        owner: query.owner,
    };
    match ops.list_allocations(&filter).await {
        Ok(allocations) => {
            let list = AllocationList {
                count: allocations.len(),
                allocations,
            };
            Json(list).into_response()
        }
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_free_blocks(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(supernet_id): Path<String>,
    Query(query): Query<FreeBlocksQuery>,
) -> impl IntoResponse {
    match ops.free_blocks(&supernet_id, query.prefix).await {
        Ok(report) => Json(report).into_response(),
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_utilization(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(supernet_id): Path<String>,
) -> impl IntoResponse {
    match ops.utilization(&supernet_id).await {
        Ok(report) => Json(report).into_response(),
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_get_allocation(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match ops.get_allocation(&id).await {
        Ok(allocation) => Json(allocation).into_response(),
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_update_allocation(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(id): Path<String>,
    Json(body): Json<UpdateAllocation>,
) -> impl IntoResponse {
    match ops.update_allocation(&id, &body).await {
        Ok(allocation) => Json(allocation).into_response(),
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_release_allocation(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match ops.release_allocation(&id).await {
        Ok(allocation) => Json(allocation).into_response(),
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_find_ip(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(address): Path<String>,
) -> impl IntoResponse {
    match ops.find_by_ip(&address).await {
        Ok(allocations) => {
            let list = AllocationList {
                count: allocations.len(),
                allocations,
            };
            Json(list).into_response()
        }
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_find_resource(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(resource_id): Path<String>,
) -> impl IntoResponse {
    match ops.find_by_resource(&resource_id).await {
        Ok(allocations) => {
            let list = AllocationList {
                count: allocations.len(),
                allocations,
            };
            Json(list).into_response()
        }
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_query_audit(
    Extension(ops): Extension<Arc<IpamOps>>,
    Query(query): Query<AuditQuery>,
) -> impl IntoResponse {
    let filter = AuditFilter {
        entity_type: query.entity_type,
        entity_id: query.entity_id,
        action: query.action,
        limit: query.limit,
    };
    match ops.query_audit(&filter).await {
        Ok(entries) => {
            let list = AuditList {
                count: entries.len(),
                entries,
            };
            Json(list).into_response()
        }
        Err(e) => ipam_error_response(e),
    }
}

async fn ipam_set_tags(
    Extension(ops): Extension<Arc<IpamOps>>,
    Path(id): Path<String>,
    Json(body): Json<TagsBody>,
) -> impl IntoResponse {
    if let Err(e) = ops.set_tags(&id, &body.tags).await {
        return ipam_error_response(e);
    }
    match ops.get_allocation(&id).await {
        Ok(allocation) => Json(allocation).into_response(),
        Err(e) => ipam_error_response(e),
    }
}
