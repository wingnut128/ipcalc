import { z } from "zod";
import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import {
  calcSubnet,
  splitSubnet,
  containsCheck,
  fromRange,
  summarize,
} from "./ipcalc.js";
import type { IpcalcResult } from "./ipcalc.js";

/** Convert an IpcalcResult into an MCP tool response. */
function toToolResult(result: IpcalcResult) {
  if (!result.success) {
    return {
      isError: true as const,
      content: [{ type: "text" as const, text: result.output }],
    };
  }
  return {
    content: [{ type: "text" as const, text: result.output }],
  };
}

/**
 * Register all ipcalc tools on the given MCP server.
 */
export function registerTools(server: McpServer): void {
  // ── ipv4_calc / ipv6_calc (unified) ──────────────────────────────────
  server.tool(
    "subnet_calc",
    "Calculate IPv4 or IPv6 subnet details from CIDR notation. " +
      "Returns network address, broadcast, mask, host range, total/usable hosts, " +
      "network class (IPv4), address type, and more.",
    {
      cidr: z
        .string()
        .describe(
          "IP address in CIDR notation, e.g. 192.168.1.0/24 or 2001:db8::/48",
        ),
    },
    async ({ cidr }) => toToolResult(await calcSubnet(cidr)),
  );

  // ── subnet_split ─────────────────────────────────────────────────────
  server.tool(
    "subnet_split",
    "Split a supernet into smaller subnets. " +
      "Provide either a count or set max=true to generate all possible subnets. " +
      "Auto-detects IPv4 vs IPv6.",
    {
      cidr: z
        .string()
        .describe("Supernet in CIDR notation, e.g. 10.0.0.0/8"),
      prefix: z
        .number()
        .int()
        .min(0)
        .max(128)
        .describe("New prefix length for the generated subnets"),
      count: z
        .number()
        .int()
        .positive()
        .optional()
        .describe(
          "Number of subnets to generate (omit if using max). " +
            "Mutually exclusive with max.",
        ),
      max: z
        .boolean()
        .optional()
        .describe(
          "Generate all possible subnets (omit or false if using count). " +
            "Mutually exclusive with count.",
        ),
    },
    async ({ cidr, prefix, count, max }) => {
      if (!max && count === undefined) {
        return {
          isError: true as const,
          content: [
            {
              type: "text" as const,
              text: "Either count or max must be specified",
            },
          ],
        };
      }
      return toToolResult(await splitSubnet(cidr, prefix, count, max));
    },
  );

  // ── contains_check ───────────────────────────────────────────────────
  server.tool(
    "contains_check",
    "Check if an IP address is contained within a CIDR range. " +
      "Auto-detects IPv4 vs IPv6.",
    {
      cidr: z
        .string()
        .describe("Network in CIDR notation, e.g. 192.168.1.0/24"),
      address: z
        .string()
        .describe("IP address to check, e.g. 192.168.1.100"),
    },
    async ({ cidr, address }) =>
      toToolResult(await containsCheck(cidr, address)),
  );

  // ── from_range ───────────────────────────────────────────────────────
  server.tool(
    "from_range",
    "Convert an IP address range (start-end) into minimal CIDR blocks. " +
      "Auto-detects IPv4 vs IPv6.",
    {
      start: z
        .string()
        .describe("Start IP address, e.g. 192.168.1.10 or 2001:db8::1"),
      end: z
        .string()
        .describe("End IP address, e.g. 192.168.1.20 or 2001:db8::ff"),
    },
    async ({ start, end }) => toToolResult(await fromRange(start, end)),
  );

  // ── summarize ────────────────────────────────────────────────────────
  server.tool(
    "summarize",
    "Aggregate/summarize a list of CIDRs into the minimal covering set. " +
      "All CIDRs must be the same address family (all IPv4 or all IPv6).",
    {
      cidrs: z
        .array(z.string())
        .min(1)
        .describe(
          "CIDR ranges to summarize, e.g. ['192.168.0.0/24', '192.168.1.0/24']",
        ),
    },
    async ({ cidrs }) => toToolResult(await summarize(cidrs)),
  );
}
