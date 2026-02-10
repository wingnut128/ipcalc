use axum::{
    Router,
    extract::Query,
    http::{StatusCode, header},
    response::{IntoResponse, Json, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use tower_http::trace::TraceLayer;
use tracing::{info, instrument, warn};
#[cfg(feature = "swagger")]
use utoipa::{IntoParams, OpenApi, ToSchema};
#[cfg(feature = "swagger")]
use utoipa_swagger_ui::SwaggerUi;

#[cfg(feature = "swagger")]
use crate::contains::ContainsResult;
use crate::contains::{check_ipv4_contains, check_ipv6_contains};
use crate::ipv4::Ipv4Subnet;
use crate::ipv6::Ipv6Subnet;
#[cfg(feature = "swagger")]
use crate::subnet_generator::{Ipv4SubnetList, Ipv6SubnetList, SplitSummary};
use crate::subnet_generator::{count_subnets, generate_ipv4_subnets, generate_ipv6_subnets};

#[cfg(feature = "swagger")]
#[derive(OpenApi)]
#[openapi(
    paths(
        health,
        version,
        calculate_ipv4,
        calculate_ipv6,
        split_ipv4,
        split_ipv6,
        contains_ipv4,
        contains_ipv6,
    ),
    components(
        schemas(Ipv4Subnet, Ipv6Subnet, Ipv4SubnetList, Ipv6SubnetList, SplitSummary, ContainsResult, SubnetQuery, SplitQuery, ContainsQuery, ErrorResponse, VersionResponse)
    ),
    tags(
        (name = "ipcalc", description = "IP subnet calculator API")
    ),
    info(
        title = "ipcalc API",
        version = env!("CARGO_PKG_VERSION"),
        description = "A fast IPv4 and IPv6 subnet calculator API",
    )
)]
pub struct ApiDoc;

#[derive(Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema, IntoParams))]
pub struct SubnetQuery {
    /// IP address in CIDR notation (e.g., 192.168.1.0/24 or 2001:db8::/48)
    cidr: String,
    /// Pretty print JSON output
    #[serde(default)]
    pretty: bool,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema, IntoParams))]
pub struct SplitQuery {
    /// Network in CIDR notation
    cidr: String,
    /// New prefix length for subnets
    prefix: u8,
    /// Number of subnets to generate. If not provided and max is true, generates all.
    count: Option<u64>,
    /// Generate maximum number of subnets possible.
    #[serde(default)]
    max: bool,
    /// Show only the number of available subnets (no generation)
    #[serde(default, alias = "count-only")]
    count_only: bool,
    /// Pretty print JSON output
    #[serde(default)]
    pretty: bool,
}

#[derive(Deserialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema, IntoParams))]
pub struct ContainsQuery {
    /// Network in CIDR notation (e.g., 192.168.1.0/24)
    cidr: String,
    /// IP address to check (e.g., 192.168.1.100)
    address: String,
    /// Pretty print JSON output
    #[serde(default)]
    pretty: bool,
}

#[derive(Serialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
struct ErrorResponse {
    /// Error message
    error: String,
}

#[derive(Serialize)]
#[cfg_attr(feature = "swagger", derive(ToSchema))]
struct VersionResponse {
    /// Application name
    name: &'static str,
    /// Application version
    version: &'static str,
}

pub fn create_router() -> Router {
    let router = Router::new()
        .route("/health", get(health))
        .route("/version", get(version))
        .route("/v4", get(calculate_ipv4))
        .route("/v6", get(calculate_ipv6))
        .route("/v4/split", get(split_ipv4))
        .route("/v6/split", get(split_ipv6))
        .route("/v4/contains", get(contains_ipv4))
        .route("/v6/contains", get(contains_ipv6));

    #[cfg(feature = "swagger")]
    let router = router
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    router.layer(TraceLayer::new_for_http())
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service is healthy", body = String)
    ),
    tag = "ipcalc"
))]
async fn health() -> &'static str {
    "OK"
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/version",
    responses(
        (status = 200, description = "Version information", body = VersionResponse)
    ),
    tag = "ipcalc"
))]
async fn version() -> Json<VersionResponse> {
    Json(VersionResponse {
        name: env!("CARGO_PKG_NAME"),
        version: env!("CARGO_PKG_VERSION"),
    })
}

/// Helper function to format JSON responses with optional pretty printing
fn json_response<T: Serialize>(value: T, pretty: bool, status: StatusCode) -> Response {
    let json_string = if pretty {
        serde_json::to_string_pretty(&value).unwrap()
    } else {
        serde_json::to_string(&value).unwrap()
    };

    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, "application/json")
        .body(json_string.into())
        .unwrap()
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/v4",
    params(
        SubnetQuery
    ),
    responses(
        (status = 200, description = "IPv4 subnet information", body = Ipv4Subnet),
        (status = 400, description = "Invalid CIDR notation", body = ErrorResponse)
    ),
    tag = "ipcalc"
))]
#[instrument(skip_all, fields(cidr = %params.cidr))]
async fn calculate_ipv4(Query(params): Query<SubnetQuery>) -> impl IntoResponse {
    info!("Calculating IPv4 subnet");
    match Ipv4Subnet::from_cidr(&params.cidr) {
        Ok(subnet) => {
            info!(network = %subnet.network_address, "IPv4 calculation successful");
            json_response(subnet, params.pretty, StatusCode::OK)
        }
        Err(e) => {
            warn!(error = %e, "IPv4 calculation failed");
            json_response(
                ErrorResponse {
                    error: e.to_string(),
                },
                params.pretty,
                StatusCode::BAD_REQUEST,
            )
        }
    }
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/v6",
    params(
        SubnetQuery
    ),
    responses(
        (status = 200, description = "IPv6 subnet information", body = Ipv6Subnet),
        (status = 400, description = "Invalid CIDR notation", body = ErrorResponse)
    ),
    tag = "ipcalc"
))]
#[instrument(skip_all, fields(cidr = %params.cidr))]
async fn calculate_ipv6(Query(params): Query<SubnetQuery>) -> impl IntoResponse {
    info!("Calculating IPv6 subnet");
    match Ipv6Subnet::from_cidr(&params.cidr) {
        Ok(subnet) => {
            info!(network = %subnet.network_address, "IPv6 calculation successful");
            json_response(subnet, params.pretty, StatusCode::OK)
        }
        Err(e) => {
            warn!(error = %e, "IPv6 calculation failed");
            json_response(
                ErrorResponse {
                    error: e.to_string(),
                },
                params.pretty,
                StatusCode::BAD_REQUEST,
            )
        }
    }
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/v4/split",
    params(
        SplitQuery
    ),
    responses(
        (status = 200, description = "Generated IPv4 subnets", body = Ipv4SubnetList),
        (status = 400, description = "Invalid parameters", body = ErrorResponse)
    ),
    tag = "ipcalc"
))]
#[instrument(skip_all, fields(cidr = %params.cidr, prefix = params.prefix, count = ?params.count, max = params.max))]
async fn split_ipv4(Query(params): Query<SplitQuery>) -> impl IntoResponse {
    info!("Splitting IPv4 supernet");

    if params.count_only {
        return match count_subnets(&params.cidr, params.prefix) {
            Ok(summary) => {
                info!(available = %summary.available_subnets, "IPv4 count-only successful");
                json_response(summary, params.pretty, StatusCode::OK)
            }
            Err(e) => {
                warn!(error = %e, "IPv4 count-only failed");
                json_response(
                    ErrorResponse {
                        error: e.to_string(),
                    },
                    params.pretty,
                    StatusCode::BAD_REQUEST,
                )
            }
        };
    }

    // Determine the actual count: None means generate max
    let actual_count = if params.max {
        None
    } else {
        match params.count {
            Some(c) => Some(c),
            None => {
                warn!("Neither count nor max specified");
                return json_response(
                    ErrorResponse {
                        error: "Either 'count' or 'max=true' must be specified".to_string(),
                    },
                    params.pretty,
                    StatusCode::BAD_REQUEST,
                );
            }
        }
    };

    match generate_ipv4_subnets(&params.cidr, params.prefix, actual_count) {
        Ok(result) => {
            info!(
                subnets_generated = result.subnets.len(),
                "IPv4 split successful"
            );
            json_response(result, params.pretty, StatusCode::OK)
        }
        Err(e) => {
            warn!(error = %e, "IPv4 split failed");
            json_response(
                ErrorResponse {
                    error: e.to_string(),
                },
                params.pretty,
                StatusCode::BAD_REQUEST,
            )
        }
    }
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/v6/split",
    params(
        SplitQuery
    ),
    responses(
        (status = 200, description = "Generated IPv6 subnets", body = Ipv6SubnetList),
        (status = 400, description = "Invalid parameters", body = ErrorResponse)
    ),
    tag = "ipcalc"
))]
#[instrument(skip_all, fields(cidr = %params.cidr, prefix = params.prefix, count = ?params.count, max = params.max))]
async fn split_ipv6(Query(params): Query<SplitQuery>) -> impl IntoResponse {
    info!("Splitting IPv6 supernet");

    if params.count_only {
        return match count_subnets(&params.cidr, params.prefix) {
            Ok(summary) => {
                info!(available = %summary.available_subnets, "IPv6 count-only successful");
                json_response(summary, params.pretty, StatusCode::OK)
            }
            Err(e) => {
                warn!(error = %e, "IPv6 count-only failed");
                json_response(
                    ErrorResponse {
                        error: e.to_string(),
                    },
                    params.pretty,
                    StatusCode::BAD_REQUEST,
                )
            }
        };
    }

    // Determine the actual count: None means generate max
    let actual_count = if params.max {
        None
    } else {
        match params.count {
            Some(c) => Some(c),
            None => {
                warn!("Neither count nor max specified");
                return json_response(
                    ErrorResponse {
                        error: "Either 'count' or 'max=true' must be specified".to_string(),
                    },
                    params.pretty,
                    StatusCode::BAD_REQUEST,
                );
            }
        }
    };

    match generate_ipv6_subnets(&params.cidr, params.prefix, actual_count) {
        Ok(result) => {
            info!(
                subnets_generated = result.subnets.len(),
                "IPv6 split successful"
            );
            json_response(result, params.pretty, StatusCode::OK)
        }
        Err(e) => {
            warn!(error = %e, "IPv6 split failed");
            json_response(
                ErrorResponse {
                    error: e.to_string(),
                },
                params.pretty,
                StatusCode::BAD_REQUEST,
            )
        }
    }
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/v4/contains",
    params(
        ContainsQuery
    ),
    responses(
        (status = 200, description = "IPv4 containment check result", body = ContainsResult),
        (status = 400, description = "Invalid parameters", body = ErrorResponse)
    ),
    tag = "ipcalc"
))]
#[instrument(skip_all, fields(cidr = %params.cidr, address = %params.address))]
async fn contains_ipv4(Query(params): Query<ContainsQuery>) -> impl IntoResponse {
    info!("Checking IPv4 address containment");
    match check_ipv4_contains(&params.cidr, &params.address) {
        Ok(result) => {
            info!(
                contained = result.contained,
                "IPv4 containment check successful"
            );
            json_response(result, params.pretty, StatusCode::OK)
        }
        Err(e) => {
            warn!(error = %e, "IPv4 containment check failed");
            json_response(
                ErrorResponse {
                    error: e.to_string(),
                },
                params.pretty,
                StatusCode::BAD_REQUEST,
            )
        }
    }
}

#[cfg_attr(feature = "swagger", utoipa::path(
    get,
    path = "/v6/contains",
    params(
        ContainsQuery
    ),
    responses(
        (status = 200, description = "IPv6 containment check result", body = ContainsResult),
        (status = 400, description = "Invalid parameters", body = ErrorResponse)
    ),
    tag = "ipcalc"
))]
#[instrument(skip_all, fields(cidr = %params.cidr, address = %params.address))]
async fn contains_ipv6(Query(params): Query<ContainsQuery>) -> impl IntoResponse {
    info!("Checking IPv6 address containment");
    match check_ipv6_contains(&params.cidr, &params.address) {
        Ok(result) => {
            info!(
                contained = result.contained,
                "IPv6 containment check successful"
            );
            json_response(result, params.pretty, StatusCode::OK)
        }
        Err(e) => {
            warn!(error = %e, "IPv6 containment check failed");
            json_response(
                ErrorResponse {
                    error: e.to_string(),
                },
                params.pretty,
                StatusCode::BAD_REQUEST,
            )
        }
    }
}
