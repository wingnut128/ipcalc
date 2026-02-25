import { execFile } from "node:child_process";
import { promisify } from "node:util";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const execFileAsync = promisify(execFile);

const __dirname = dirname(fileURLToPath(import.meta.url));

/**
 * Resolve the ipcalc binary path.
 * Priority: IPCALC_BIN env var > release build in repo > PATH lookup.
 */
function resolveBinary(): string {
  if (process.env.IPCALC_BIN) {
    return process.env.IPCALC_BIN;
  }
  // When installed alongside the Rust project, check the release build
  const repoBinary = resolve(__dirname, "../../target/release/ipcalc");
  return repoBinary;
}

const IPCALC_BIN = resolveBinary();
const TIMEOUT_MS = 10_000;

export interface IpcalcResult {
  success: boolean;
  output: string;
}

/**
 * Execute the ipcalc binary with the given arguments and return JSON output.
 */
export async function runIpcalc(args: string[]): Promise<IpcalcResult> {
  try {
    const { stdout, stderr } = await execFileAsync(IPCALC_BIN, args, {
      timeout: TIMEOUT_MS,
      maxBuffer: 10 * 1024 * 1024, // 10 MB
    });

    if (stderr && !stderr.startsWith("Warning:")) {
      return { success: false, output: stderr.trim() };
    }

    return { success: true, output: stdout.trim() };
  } catch (err: unknown) {
    const msg =
      err instanceof Error ? err.message : "Unknown error executing ipcalc";
    return { success: false, output: msg };
  }
}

/**
 * Calculate IPv4 or IPv6 subnet details from CIDR notation.
 */
export async function calcSubnet(cidr: string): Promise<IpcalcResult> {
  return runIpcalc([cidr, "--format", "json"]);
}

/**
 * Split a supernet into smaller subnets.
 */
export async function splitSubnet(
  cidr: string,
  prefix: number,
  count?: number,
  max?: boolean,
): Promise<IpcalcResult> {
  const args = ["split", cidr, "-p", String(prefix), "--format", "json"];
  if (max) {
    args.push("--max");
  } else if (count !== undefined) {
    args.push("-n", String(count));
  }
  return runIpcalc(args);
}

/**
 * Check if an IP address is contained within a CIDR range.
 */
export async function containsCheck(
  cidr: string,
  address: string,
): Promise<IpcalcResult> {
  return runIpcalc(["contains", cidr, address, "--format", "json"]);
}

/**
 * Convert an IP range to minimal CIDR blocks.
 */
export async function fromRange(
  start: string,
  end: string,
): Promise<IpcalcResult> {
  return runIpcalc(["from-range", start, end, "--format", "json"]);
}

/**
 * Summarize/aggregate CIDRs into the minimal covering set.
 */
export async function summarize(cidrs: string[]): Promise<IpcalcResult> {
  return runIpcalc(["summarize", ...cidrs, "--format", "json"]);
}
