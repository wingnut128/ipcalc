use std::process::Command;

fn run_ipcalc(args: &[&str]) -> (String, String, bool) {
    let output = Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .output()
        .expect("Failed to run ipcalc");

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    (stdout, stderr, output.status.success())
}

#[test]
fn test_ipv4_json_output() {
    let (stdout, _, success) = run_ipcalc(&["v4", "192.168.1.0/24"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["network_address"], "192.168.1.0");
    assert_eq!(json["broadcast_address"], "192.168.1.255");
    assert_eq!(json["prefix_length"], 24);
    assert_eq!(json["usable_hosts"], 254);
}

#[test]
fn test_ipv4_text_output() {
    let (stdout, _, success) = run_ipcalc(&["v4", "10.0.0.0/8", "--format", "text"]);
    assert!(success);
    assert!(stdout.contains("IPv4 Subnet Calculator"));
    assert!(stdout.contains("Network Address:   10.0.0.0"));
    assert!(stdout.contains("Broadcast Address: 10.255.255.255"));
}

#[test]
fn test_ipv6_json_output() {
    let (stdout, _, success) = run_ipcalc(&["v6", "2001:db8::/32"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["network_address"], "2001:db8::");
    assert_eq!(json["prefix_length"], 32);
    assert_eq!(json["address_type"], "Global Unicast");
}

#[test]
fn test_ipv6_text_output() {
    let (stdout, _, success) = run_ipcalc(&["v6", "fe80::1/64", "--format", "text"]);
    assert!(success);
    assert!(stdout.contains("IPv6 Subnet Calculator"));
    assert!(stdout.contains("Link-Local Unicast"));
}

#[test]
fn test_split_ipv4() {
    let (stdout, _, success) = run_ipcalc(&["split", "192.168.0.0/22", "-p", "27", "-n", "5"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["new_prefix"], 27);
    assert_eq!(json["requested_count"], 5);
    assert_eq!(json["subnets"].as_array().unwrap().len(), 5);
    assert_eq!(json["subnets"][0]["network_address"], "192.168.0.0");
    assert_eq!(json["subnets"][1]["network_address"], "192.168.0.32");
}

#[test]
fn test_split_ipv6() {
    let (stdout, _, success) = run_ipcalc(&["split", "2001:db8::/32", "-p", "48", "-n", "3"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["new_prefix"], 48);
    assert_eq!(json["subnets"].as_array().unwrap().len(), 3);
}

#[test]
fn test_invalid_ipv4() {
    let (_, stderr, success) = run_ipcalc(&["v4", "999.999.999.999/24"]);
    assert!(!success);
    assert!(stderr.contains("Error"));
}

#[test]
fn test_invalid_prefix() {
    let (_, stderr, success) = run_ipcalc(&["v4", "192.168.1.0/33"]);
    assert!(!success);
    assert!(stderr.contains("Error"));
}

#[test]
fn test_file_output() {
    let temp_file = "/tmp/ipcalc_test_output.json";
    let (_, _, success) = run_ipcalc(&["v4", "172.16.0.0/12", "-o", temp_file]);
    assert!(success);

    let content = std::fs::read_to_string(temp_file).expect("Failed to read output file");
    let json: serde_json::Value = serde_json::from_str(&content).expect("Invalid JSON in file");
    assert_eq!(json["network_address"], "172.16.0.0");

    std::fs::remove_file(temp_file).ok();
}

#[test]
fn test_split_too_many_subnets() {
    // /22 can only fit 32 /27 subnets, requesting 100 should fail
    let (_, stderr, success) = run_ipcalc(&["split", "192.168.0.0/22", "-p", "27", "-n", "100"]);
    assert!(!success);
    assert!(stderr.contains("Error"));
}

#[test]
fn test_split_ipv4_max() {
    // Test --max option generates all possible subnets
    let (stdout, _, success) = run_ipcalc(&["split", "192.168.0.0/22", "-p", "27", "--max"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    // /22 to /27 is 5 bits difference, so 32 subnets
    assert_eq!(json["requested_count"], 32);
    assert_eq!(json["subnets"].as_array().unwrap().len(), 32);
}

#[test]
fn test_split_ipv6_max() {
    // Test --max option for IPv6
    let (stdout, _, success) = run_ipcalc(&["split", "2001:db8:abcd::/48", "-p", "52", "--max"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    // /48 to /52 is 4 bits difference, so 16 subnets
    assert_eq!(json["requested_count"], 16);
    assert_eq!(json["subnets"].as_array().unwrap().len(), 16);
}

#[test]
fn test_split_requires_count_or_max() {
    // Neither --count nor --max should fail
    let (_, stderr, success) = run_ipcalc(&["split", "192.168.0.0/22", "-p", "27"]);
    assert!(!success);
    assert!(stderr.contains("Error"));
}

#[test]
fn test_direct_ipv4() {
    let (stdout, _, success) = run_ipcalc(&["192.168.1.0/24"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["network_address"], "192.168.1.0");
    assert_eq!(json["broadcast_address"], "192.168.1.255");
    assert_eq!(json["prefix_length"], 24);
}

#[test]
fn test_direct_ipv6() {
    let (stdout, _, success) = run_ipcalc(&["2001:db8::/32"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["network_address"], "2001:db8::");
    assert_eq!(json["prefix_length"], 32);
    assert_eq!(json["address_type"], "Global Unicast");
}

#[test]
fn test_direct_ipv4_text_format() {
    let (stdout, _, success) = run_ipcalc(&["10.0.0.0/8", "--format", "text"]);
    assert!(success);
    assert!(stdout.contains("IPv4 Subnet Calculator"));
    assert!(stdout.contains("Network Address:   10.0.0.0"));
}

#[test]
fn test_v4_deprecation_warning() {
    let (stdout, stderr, success) = run_ipcalc(&["v4", "192.168.1.0/24"]);
    assert!(success);
    assert!(stderr.contains("deprecated"));
    // Verify it still works
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["network_address"], "192.168.1.0");
}

#[test]
fn test_v6_deprecation_warning() {
    let (stdout, stderr, success) = run_ipcalc(&["v6", "2001:db8::/32"]);
    assert!(success);
    assert!(stderr.contains("deprecated"));
    // Verify it still works
    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["network_address"], "2001:db8::");
}

#[test]
fn test_contains_ipv4_json() {
    let (stdout, _, success) = run_ipcalc(&["contains", "192.168.1.0/24", "192.168.1.100"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["cidr"], "192.168.1.0/24");
    assert_eq!(json["address"], "192.168.1.100");
    assert_eq!(json["contained"], true);
    assert_eq!(json["network_address"], "192.168.1.0");
    assert_eq!(json["broadcast_address"], "192.168.1.255");
}

#[test]
fn test_contains_ipv4_not_contained() {
    let (stdout, _, success) = run_ipcalc(&["contains", "192.168.1.0/24", "10.0.0.1"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["contained"], false);
}

#[test]
fn test_contains_ipv6_json() {
    let (stdout, _, success) = run_ipcalc(&["contains", "2001:db8::/32", "2001:db8::1"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["contained"], true);
    assert_eq!(json["address"], "2001:db8::1");
}

#[test]
fn test_contains_ipv4_text() {
    let (stdout, _, success) = run_ipcalc(&[
        "contains",
        "192.168.1.0/24",
        "192.168.1.100",
        "--format",
        "text",
    ]);
    assert!(success);
    assert!(stdout.contains("Address Containment Check"));
    assert!(stdout.contains("Contained:         Yes"));
    assert!(stdout.contains("Network Address:   192.168.1.0"));
}

#[test]
fn test_contains_invalid_address() {
    let (_, stderr, success) = run_ipcalc(&["contains", "192.168.1.0/24", "not-an-ip"]);
    assert!(!success);
    assert!(stderr.contains("Error"));
}

#[test]
fn test_split_count_only_ipv4() {
    let (stdout, _, success) = run_ipcalc(&["split", "192.168.0.0/22", "-p", "27", "--count-only"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["available_subnets"], "32");
    assert_eq!(json["new_prefix"], 27);
}

#[test]
fn test_split_count_only_ipv6() {
    let (stdout, _, success) = run_ipcalc(&["split", "2001:db8::/64", "-p", "96", "--count-only"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["available_subnets"], "4294967296");
    assert_eq!(json["new_prefix"], 96);
}

#[test]
fn test_split_count_only_ipv6_huge() {
    let (stdout, _, success) = run_ipcalc(&["split", "2001:db8::/32", "-p", "128", "--count-only"]);
    assert!(success);

    let json: serde_json::Value = serde_json::from_str(&stdout).expect("Invalid JSON");
    assert_eq!(json["available_subnets"], "2^96");
}

#[test]
fn test_split_limit_exceeded_ipv4() {
    let (_, stderr, success) = run_ipcalc(&["split", "10.0.0.0/8", "-p", "32", "--max"]);
    assert!(!success);
    assert!(stderr.contains("limit"));
}

#[test]
fn test_split_limit_exceeded_ipv6() {
    let (_, stderr, success) = run_ipcalc(&["split", "2001:db8::/32", "-p", "64", "--max"]);
    assert!(!success);
    assert!(stderr.contains("limit"));
}
