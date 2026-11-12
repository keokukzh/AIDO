import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { createReadStream, createWriteStream } from "node:fs";
import { access, rename } from "node:fs/promises";
import { join } from "node:path";
import { pipeline } from "node:stream/promises";
import { fileURLToPath } from "node:url";

const directory = fileURLToPath(new URL("../tooling/rea/", import.meta.url));
const executable = join(directory, "node.exe");
const expected = "b3094d0b49f9ad602262a9921551737bb97637c05dd357a06ae98188d7290aa3";
const license = join(directory, "NODE-LICENSE");

async function checksum(path) {
  const hash = createHash("sha256");
  try {
    for await (const chunk of createReadStream(path)) hash.update(chunk);
    return hash.digest("hex");
  } catch (error) {
    if (error.code === "ENOENT") return undefined;
    throw error;
  }
}

async function download(url, path) {
  const response = await fetch(url, { signal: AbortSignal.timeout(120_000) });
  if (!response.ok || !response.body) throw new Error(`Download failed with HTTP ${response.status}`);
  await pipeline(response.body, createWriteStream(path));
}

if (await checksum(executable) !== expected) {
  const temporary = executable + ".download";
  await download("https://nodejs.org/dist/v24.16.0/win-x64/node.exe", temporary);
  if (await checksum(temporary) !== expected) throw new Error("Node.js SHA-256 mismatch.");
  await rename(temporary, executable);
}

try {
  await access(license);
} catch {
  await download("https://raw.githubusercontent.com/nodejs/node/v24.16.0/LICENSE", license + ".download");
  await rename(license + ".download", license);
}

const install = spawnSync("cmd.exe", ["/d", "/s", "/c", "npm.cmd ci --omit=dev --ignore-scripts --no-audit --no-fund"], {
  cwd: directory, stdio: "inherit", windowsHide: true,
});
if (install.status !== 0) process.exit(install.status ?? 1);

const entry = join(directory, "node_modules", "rea-agents", "scripts", "rea.mjs");
const verification = spawnSync(executable, [entry, "--version"], { encoding: "utf8", windowsHide: true });
if (verification.status !== 0 || verification.stdout.trim() !== "6.7.0") {
  throw new Error("Bundled REA 6.7.0 could not be verified.");
}
process.stdout.write("Bundled REA 6.7.0 verified.\n");

const agentDesktopBin = fileURLToPath(new URL("../tooling/agent-desktop/agent-desktop.exe", import.meta.url));
const agentDesktopExpected = "d274667031322e50b9c2b1ab81c2ca46835d52cc932769a5c34bd4d481fd94c3";
if (await checksum(agentDesktopBin) !== agentDesktopExpected) {
  throw new Error("Bundled agent-desktop SHA-256 mismatch.");
}
process.stdout.write("Bundled agent-desktop SHA-256 verified.\n");
