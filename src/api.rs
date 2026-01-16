use axum::{
    Router,
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument, warn};

use crate::ipv4::Ipv4Subnet;
use crate::ipv6::Ipv6Subnet;
use crate::subnet_generator::{generate_ipv4_subnets, generate_ipv6_subnets};

#[derive(Deserialize)]
pub struct SubnetQuery {
    cidr: String,
}

#[derive(Deserialize)]
pub struct SplitQuery {
    cidr: String,
    prefix: u8,
    count: u64,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

#[derive(Serialize)]
struct VersionResponse {
    name: &'static str,
    version: &'static str,
}

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/v4", get(calculate_ipv4))
        .route("/v6", get(calculate_ipv6))
        .route("/v4/split", get(split_ipv4))
        .route("/v6/split", get(split_ipv6))
        .layer(TraceLayer::new_for_http())
}

async fn health() -> &'static str {
    "OK"
}

async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    })
}

#[instrument(skip_all, fields(cidr = %params.cidr))]
async fn calculate_ipv4(Query(params): Query<SubnetQuery>) -> impl IntoResponse {
    info!("Calculating IPv4 subnet");
    match Ipv4Subnet::from_cidr(&params.cidr) {
        Ok(subnet) => {
            info!(network = %subnet.network_address, "IPv4 calculation successful");
            (StatusCode::OK, Json(serde_json::to_value(subnet).unwrap()))
        }
        Err(e) => {
            warn!(error = %e, "IPv4 calculation failed");
            (
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::to_value(ErrorResponse {
                        error: e.to_string(),
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

#[instrument(skip_all, fields(cidr = %params.cidr))]
async fn calculate_ipv6(Query(params): Query<SubnetQuery>) -> impl IntoResponse {
    info!("Calculating IPv6 subnet");
    match Ipv6Subnet::from_cidr(&params.cidr) {
        Ok(subnet) => {
            info!(network = %subnet.network_address, "IPv6 calculation successful");
            (StatusCode::OK, Json(serde_json::to_value(subnet).unwrap()))
        }
        Err(e) => {
            warn!(error = %e, "IPv6 calculation failed");
            (
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::to_value(ErrorResponse {
                        error: e.to_string(),
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

#[instrument(skip_all, fields(cidr = %params.cidr, prefix = params.prefix, count = params.count))]
async fn split_ipv4(Query(params): Query<SplitQuery>) -> impl IntoResponse {
    info!("Splitting IPv4 supernet");
    match generate_ipv4_subnets(&params.cidr, params.prefix, params.count) {
        Ok(result) => {
            info!(
                subnets_generated = result.subnets.len(),
                "IPv4 split successful"
            );
            (StatusCode::OK, Json(serde_json::to_value(result).unwrap()))
        }
        Err(e) => {
            warn!(error = %e, "IPv4 split failed");
            (
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::to_value(ErrorResponse {
                        error: e.to_string(),
                    })
                    .unwrap(),
                ),
            )
        }
    }
}

#[instrument(skip_all, fields(cidr = %params.cidr, prefix = params.prefix, count = params.count))]
async fn split_ipv6(Query(params): Query<SplitQuery>) -> impl IntoResponse {
    info!("Splitting IPv6 supernet");
    match generate_ipv6_subnets(&params.cidr, params.prefix, params.count) {
        Ok(result) => {
            info!(
                subnets_generated = result.subnets.len(),
                "IPv6 split successful"
            );
            (StatusCode::OK, Json(serde_json::to_value(result).unwrap()))
        }
        Err(e) => {
            warn!(error = %e, "IPv6 split failed");
            (
                StatusCode::BAD_REQUEST,
                Json(
                    serde_json::to_value(ErrorResponse {
                        error: e.to_string(),
                    })
                    .unwrap(),
                ),
            )
        }
    }
}
