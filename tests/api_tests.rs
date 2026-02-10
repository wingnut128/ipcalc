use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::Response;
use http_body_util::BodyExt;
use ipcalc::api::create_router;
use tower::ServiceExt;

async fn get(uri: &str) -> (StatusCode, String) {
    let app = create_router();
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let resp: Response = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

// ── Health & Version ────────────────────────────────────────────────

#[tokio::test]
async fn test_health() {
    let (status, body) = get("/health").await;
    assert_eq!(status, 200);
    assert_eq!(body, "OK");
}

#[tokio::test]
async fn test_version() {
    let (status, body) = get("/version").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["name"], "ipcalc");
    assert!(json["version"].is_string());
}

// ── IPv4 ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_v4_valid() {
    let (status, body) = get("/v4?cidr=192.168.1.0/24").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["network_address"], "192.168.1.0");
    assert_eq!(json["broadcast_address"], "192.168.1.255");
    assert_eq!(json["prefix_length"], 24);
}

#[tokio::test]
async fn test_v4_invalid() {
    let (status, body) = get("/v4?cidr=invalid").await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].is_string());
}

// ── IPv6 ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_v6_valid() {
    let (status, body) = get("/v6?cidr=2001:db8::/32").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["network_address"], "2001:db8::");
    assert_eq!(json["prefix_length"], 32);
}

#[tokio::test]
async fn test_v6_invalid() {
    let (status, body) = get("/v6?cidr=invalid").await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].is_string());
}

// ── IPv4 Split ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_v4_split() {
    let (status, body) = get("/v4/split?cidr=192.168.0.0/24&prefix=27&count=5").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["subnets"].as_array().unwrap().len(), 5);
}

#[tokio::test]
async fn test_v4_split_max() {
    let (status, body) = get("/v4/split?cidr=192.168.0.0/24&prefix=26&max=true").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    // /24 split into /26 = 2^(26-24) = 4 subnets
    assert_eq!(json["subnets"].as_array().unwrap().len(), 4);
}

#[tokio::test]
async fn test_v4_split_missing_params() {
    let (status, body) = get("/v4/split?cidr=192.168.0.0/24&prefix=27").await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("count"));
}

// ── IPv6 Split ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_v6_split() {
    let (status, body) = get("/v6/split?cidr=2001:db8::/32&prefix=48&count=3").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["subnets"].as_array().unwrap().len(), 3);
}

// ── IPv4 Contains ───────────────────────────────────────────────────

#[tokio::test]
async fn test_v4_contains_true() {
    let (status, body) = get("/v4/contains?cidr=192.168.1.0/24&address=192.168.1.100").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["contained"], true);
}

#[tokio::test]
async fn test_v4_contains_false() {
    let (status, body) = get("/v4/contains?cidr=192.168.1.0/24&address=10.0.0.1").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["contained"], false);
}

#[tokio::test]
async fn test_v4_contains_invalid() {
    let (status, body) = get("/v4/contains?cidr=192.168.1.0/24&address=bad").await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].is_string());
}

// ── IPv6 Contains ───────────────────────────────────────────────────

#[tokio::test]
async fn test_v6_contains() {
    let (status, body) = get("/v6/contains?cidr=2001:db8::/32&address=2001:db8::1").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["contained"], true);
}

// ── Pretty Output ───────────────────────────────────────────────────

// ── Split Count Only ────────────────────────────────────────────────

#[tokio::test]
async fn test_v4_split_count_only() {
    let (status, body) = get("/v4/split?cidr=192.168.0.0/22&prefix=27&count_only=true").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["available_subnets"], "32");
    assert_eq!(json["new_prefix"], 27);
}

#[tokio::test]
async fn test_v4_split_count_only_hyphenated() {
    let (status, body) = get("/v4/split?cidr=192.168.0.0/22&prefix=27&count-only=true").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["available_subnets"], "32");
    assert_eq!(json["new_prefix"], 27);
}

#[tokio::test]
async fn test_v6_split_count_only() {
    let (status, body) = get("/v6/split?cidr=2001:db8::/64&prefix=96&count_only=true").await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["available_subnets"], "4294967296");
    assert_eq!(json["new_prefix"], 96);
}

#[tokio::test]
async fn test_v6_split_limit_exceeded() {
    let (status, body) = get("/v6/split?cidr=2001:db8::/32&prefix=64&max=true").await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("limit"));
}

// ── Pretty Output ───────────────────────────────────────────────────

#[tokio::test]
async fn test_pretty_output() {
    let (status, body) = get("/v4?cidr=192.168.1.0/24&pretty=true").await;
    assert_eq!(status, 200);
    // Pretty-printed JSON contains newlines and indentation
    assert!(body.contains('\n'));
    assert!(body.contains("  "));
}
