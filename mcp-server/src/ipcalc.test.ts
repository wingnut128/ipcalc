import {
  calcSubnet,
  splitSubnet,
  containsCheck,
  fromRange,
  summarize,
} from "./ipcalc.js";

// These tests require the ipcalc binary to be built.
// Run `cargo build --release` from the repo root first.

describe("calcSubnet", () => {
  it("calculates IPv4 subnet details", async () => {
    const result = await calcSubnet("192.168.1.0/24");
    expect(result.success).toBe(true);
    const data = JSON.parse(result.output);
    expect(data.network_address).toBe("192.168.1.0");
    expect(data.broadcast_address).toBe("192.168.1.255");
    expect(data.prefix_length).toBe(24);
    expect(data.total_hosts).toBe(256);
    expect(data.usable_hosts).toBe(254);
    expect(data.is_private).toBe(true);
  });

  it("calculates IPv6 subnet details", async () => {
    const result = await calcSubnet("2001:db8::/48");
    expect(result.success).toBe(true);
    const data = JSON.parse(result.output);
    expect(data.network_address).toBe("2001:db8::");
    expect(data.prefix_length).toBe(48);
    expect(data.address_type).toContain("Documentation");
  });

  it("returns error for invalid CIDR", async () => {
    const result = await calcSubnet("not-a-cidr");
    expect(result.success).toBe(false);
  });
});

describe("splitSubnet", () => {
  it("splits IPv4 supernet with count", async () => {
    const result = await splitSubnet("10.0.0.0/8", 16, 3);
    expect(result.success).toBe(true);
    const data = JSON.parse(result.output);
    expect(data.new_prefix).toBe(16);
    expect(data.requested_count).toBe(3);
    expect(data.subnets).toHaveLength(3);
  });

  it("splits with max flag", async () => {
    const result = await splitSubnet("192.168.0.0/24", 26, undefined, true);
    expect(result.success).toBe(true);
    const data = JSON.parse(result.output);
    expect(data.subnets).toHaveLength(4); // /24 -> /26 = 4 subnets
  });

  it("returns error for invalid split", async () => {
    // Can't split /24 into /16 (smaller prefix)
    const result = await splitSubnet("192.168.1.0/24", 16, 1);
    expect(result.success).toBe(false);
  });
});

describe("containsCheck", () => {
  it("detects contained address", async () => {
    const result = await containsCheck("192.168.1.0/24", "192.168.1.100");
    expect(result.success).toBe(true);
    const data = JSON.parse(result.output);
    expect(data.contained).toBe(true);
  });

  it("detects non-contained address", async () => {
    const result = await containsCheck("192.168.1.0/24", "10.0.0.1");
    expect(result.success).toBe(true);
    const data = JSON.parse(result.output);
    expect(data.contained).toBe(false);
  });
});

describe("fromRange", () => {
  it("converts range to CIDRs", async () => {
    const result = await fromRange("192.168.1.0", "192.168.1.255");
    expect(result.success).toBe(true);
    const data = JSON.parse(result.output);
    expect(data.cidr_count).toBe(1);
    expect(data.cidrs[0].prefix_length).toBe(24);
  });

  it("handles non-aligned range", async () => {
    const result = await fromRange("192.168.1.10", "192.168.1.20");
    expect(result.success).toBe(true);
    const data = JSON.parse(result.output);
    expect(data.cidr_count).toBeGreaterThan(1);
  });

  it("returns error for reversed range", async () => {
    const result = await fromRange("192.168.1.20", "192.168.1.10");
    expect(result.success).toBe(false);
  });
});

describe("summarize", () => {
  it("merges adjacent CIDRs", async () => {
    const result = await summarize(["192.168.0.0/24", "192.168.1.0/24"]);
    expect(result.success).toBe(true);
    const data = JSON.parse(result.output);
    expect(data.input_count).toBe(2);
    expect(data.output_count).toBe(1);
    expect(data.cidrs[0].prefix_length).toBe(23);
  });

  it("returns error for empty list", async () => {
    const result = await summarize([]);
    expect(result.success).toBe(false);
  });
});
