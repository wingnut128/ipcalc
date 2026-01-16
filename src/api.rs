use axum::{
    extract::Query,
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};

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

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v4", get(calculate_ipv4))
        .route("/v6", get(calculate_ipv6))
        .route("/v4/split", get(split_ipv4))
        .route("/v6/split", get(split_ipv6))
}

async fn health() -> &'static str {
    "OK"
}

async fn calculate_ipv4(Query(params): Query<SubnetQuery>) -> impl IntoResponse {
    match Ipv4Subnet::from_cidr(&params.cidr) {
        Ok(subnet) => (StatusCode::OK, Json(serde_json::to_value(subnet).unwrap())),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(ErrorResponse { error: e.to_string() }).unwrap()),
        ),
    }
}

async fn calculate_ipv6(Query(params): Query<SubnetQuery>) -> impl IntoResponse {
    match Ipv6Subnet::from_cidr(&params.cidr) {
        Ok(subnet) => (StatusCode::OK, Json(serde_json::to_value(subnet).unwrap())),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(ErrorResponse { error: e.to_string() }).unwrap()),
        ),
    }
}

async fn split_ipv4(Query(params): Query<SplitQuery>) -> impl IntoResponse {
    match generate_ipv4_subnets(&params.cidr, params.prefix, params.count) {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(ErrorResponse { error: e.to_string() }).unwrap()),
        ),
    }
}

async fn split_ipv6(Query(params): Query<SplitQuery>) -> impl IntoResponse {
    match generate_ipv6_subnets(&params.cidr, params.prefix, params.count) {
        Ok(result) => (StatusCode::OK, Json(serde_json::to_value(result).unwrap())),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(ErrorResponse { error: e.to_string() }).unwrap()),
        ),
    }
}
