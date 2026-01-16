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
