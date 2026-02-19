use axum::body::Body;
use axum::http::{Request, StatusCode, header};
use axum::response::Response;
use http_body_util::BodyExt;
use ipcalc::api::{RouterConfig, create_router};
use tower::ServiceExt;

async fn get(uri: &str) -> (StatusCode, String) {
    let app = create_router(RouterConfig::default());
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let resp: Response = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

async fn get_with_headers(uri: &str) -> (StatusCode, String, axum::http::HeaderMap) {
    let app = create_router(RouterConfig::default());
    let req = Request::builder().uri(uri).body(Body::empty()).unwrap();
    let resp: Response = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let headers = resp.headers().clone();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(body.to_vec()).unwrap(), headers)
}

async fn post_json(uri: &str, json_body: &str) -> (StatusCode, String) {
    let app = create_router(RouterConfig::default());
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json_body.to_string()))
        .unwrap();
    let resp: Response = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    (status, String::from_utf8(body.to_vec()).unwrap())
}

async fn post_json_with_config(
    uri: &str,
    json_body: &str,
    config: RouterConfig,
) -> (StatusCode, String) {
    let app = create_router(config);
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(json_body.to_string()))
        .unwrap();
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

// ── Batch ────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_batch_v4() {
    let (status, body) = post_json("/batch", r#"{"cidrs":["192.168.1.0/24","10.0.0.0/8"]}"#).await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["count"], 2);
    assert_eq!(json["results"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_batch_mixed() {
    let (status, body) =
        post_json("/batch", r#"{"cidrs":["192.168.1.0/24","2001:db8::/32"]}"#).await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["count"], 2);
    assert_eq!(json["results"][0]["subnet"]["version"], "v4");
    assert_eq!(json["results"][1]["subnet"]["version"], "v6");
}

#[tokio::test]
async fn test_batch_with_invalid() {
    let (status, body) = post_json(
        "/batch",
        r#"{"cidrs":["192.168.1.0/24","invalid","10.0.0.0/8"]}"#,
    )
    .await;
    assert_eq!(status, 200);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(json["count"], 3);
    assert!(json["results"][0]["subnet"].is_object());
    assert!(json["results"][1]["error"].is_string());
    assert!(json["results"][2]["subnet"].is_object());
}

#[tokio::test]
async fn test_batch_empty() {
    let (status, body) = post_json("/batch", r#"{"cidrs":[]}"#).await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].is_string());
}

#[tokio::test]
async fn test_batch_pretty() {
    let (status, body) = post_json("/batch", r#"{"cidrs":["192.168.1.0/24"],"pretty":true}"#).await;
    assert_eq!(status, 200);
    // Pretty-printed JSON contains newlines and indentation
    assert!(body.contains('\n'));
    assert!(body.contains("  "));
}

// ── CSV Format ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_v4_csv_format() {
    let (status, body) = get("/v4?cidr=192.168.1.0/24&format=csv").await;
    assert_eq!(status, 200);
    let lines: Vec<&str> = body.lines().collect();
    assert!(lines[0].contains("network_address"));
    assert!(lines[1].contains("192.168.1.0"));
}

#[tokio::test]
async fn test_v6_csv_format() {
    let (status, body) = get("/v6?cidr=2001:db8::/32&format=csv").await;
    assert_eq!(status, 200);
    let lines: Vec<&str> = body.lines().collect();
    assert!(lines[0].contains("network_address"));
    assert!(lines[1].contains("2001:db8::"));
}

#[tokio::test]
async fn test_v4_split_csv_format() {
    let (status, body) = get("/v4/split?cidr=192.168.0.0/24&prefix=26&max=true&format=csv").await;
    assert_eq!(status, 200);
    let data_lines: Vec<&str> = body
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .collect();
    // header + 4 subnets
    assert_eq!(data_lines.len(), 5);
}

// ── YAML Format ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_v4_yaml_format() {
    let (status, body) = get("/v4?cidr=192.168.1.0/24&format=yaml").await;
    assert_eq!(status, 200);
    assert!(body.contains("network_address:"));
    assert!(body.contains("192.168.1.0"));
}

#[tokio::test]
async fn test_v6_yaml_format() {
    let (status, body) = get("/v6?cidr=2001:db8::/32&format=yaml").await;
    assert_eq!(status, 200);
    assert!(body.contains("network_address:"));
    assert!(body.contains("prefix_length:"));
}

// ── Error responses stay JSON regardless of format ──────────────────

#[tokio::test]
async fn test_error_stays_json_with_csv_format() {
    let (status, body) = get("/v4?cidr=invalid&format=csv").await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].is_string());
}

#[tokio::test]
async fn test_error_stays_json_with_yaml_format() {
    let (status, body) = get("/v4?cidr=invalid&format=yaml").await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].is_string());
}

// ── Security Tests ──────────────────────────────────────────────────

#[tokio::test]
async fn test_security_headers_present() {
    let (status, _body, headers) = get_with_headers("/health").await;
    assert_eq!(status, 200);
    assert_eq!(headers.get("x-content-type-options").unwrap(), "nosniff");
    assert_eq!(headers.get("x-frame-options").unwrap(), "DENY");
    assert_eq!(headers.get("cache-control").unwrap(), "no-store");
}

#[tokio::test]
async fn test_batch_size_exceeded() {
    use ipcalc::config::ServerConfig;
    let config = RouterConfig {
        server: ServerConfig {
            max_batch_size: 2,
            ..Default::default()
        },
    };

    // 3 CIDRs with max_batch_size=2 should fail
    let (status, body) = post_json_with_config(
        "/batch",
        r#"{"cidrs":["192.168.1.0/24","10.0.0.0/8","172.16.0.0/12"]}"#,
        config,
    )
    .await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(json["error"].as_str().unwrap().contains("exceeds maximum"));
}

#[tokio::test]
async fn test_swagger_disabled_by_default() {
    let app = create_router(RouterConfig::default());
    let req = Request::builder()
        .uri("/swagger-ui")
        .body(Body::empty())
        .unwrap();
    let resp: Response = app.oneshot(req).await.unwrap();
    // Swagger should not be available (404) when enable_swagger is false
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_input_too_long_rejected() {
    let long_cidr = "a".repeat(300);
    let uri = format!("/v4?cidr={}", long_cidr);
    let (status, body) = get(&uri).await;
    assert_eq!(status, 400);
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert!(
        json["error"]
            .as_str()
            .unwrap()
            .contains("exceeds maximum length")
    );
}

#[tokio::test]
async fn test_body_size_limit() {
    use ipcalc::config::ServerConfig;
    let config = RouterConfig {
        server: ServerConfig {
            max_body_size: 64,
            ..Default::default()
        },
    };

    let app = create_router(config);
    // Send a body larger than 64 bytes
    let large_body = format!(
        r#"{{"cidrs":[{}]}}"#,
        (0..20)
            .map(|i| format!(r#""10.0.{}.0/24""#, i))
            .collect::<Vec<_>>()
            .join(",")
    );
    let req = Request::builder()
        .method("POST")
        .uri("/batch")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(large_body))
        .unwrap();
    let resp: Response = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}
