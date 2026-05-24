const fs = require("node:fs");
const https = require("node:https");
const os = require("node:os");
const path = require("node:path");
const { createHash } = require("node:crypto");
const { execFileSync } = require("node:child_process");

const root = path.join(__dirname, "..");
const vendor = path.join(root, "vendor");
const version = process.env.REPO_DOCTOR_VERSION || "latest";

function target() {
  const arch = os.arch() === "x64" ? "x86_64" : os.arch() === "arm64" ? "aarch64" : null;
  const platform = {
    linux: "unknown-linux-gnu",
    darwin: "apple-darwin",
    win32: "pc-windows-msvc"
  }[os.platform()];
  if (!arch || !platform) throw new Error(`unsupported platform: ${os.platform()} ${os.arch()}`);
  const triple = `${arch}-${platform}`;
  if (triple === "aarch64-unknown-linux-gnu" || triple === "aarch64-pc-windows-msvc") {
    throw new Error(`unsupported release target: ${triple}`);
  }
  return triple;
}

function download(url, dest) {
  return new Promise((resolve, reject) => {
    https.get(url, (res) => {
      if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
        return download(res.headers.location, dest).then(resolve, reject);
      }
      if (res.statusCode !== 200) return reject(new Error(`download failed: ${url} ${res.statusCode}`));
      const out = fs.createWriteStream(dest);
      res.pipe(out);
      out.on("finish", () => out.close(resolve));
    }).on("error", reject);
  });
}

async function main() {
  fs.mkdirSync(vendor, { recursive: true });
  const triple = target();
  const isWindows = os.platform() === "win32";
  const ext = isWindows ? "zip" : "tar.gz";
  const base = version === "latest"
    ? "https://github.com/Kota-Ohno/repo-doctor/releases/latest/download"
    : `https://github.com/Kota-Ohno/repo-doctor/releases/download/${version}`;
  const archive = path.join(os.tmpdir(), `repo-doctor-${triple}.${ext}`);
  const checksum = `${archive}.sha256`;

  await download(`${base}/repo-doctor-${triple}.${ext}`, archive);
  await download(`${base}/repo-doctor-${triple}.${ext}.sha256`, checksum);
  const expected = fs.readFileSync(checksum, "utf8").trim().split(/\s+/)[0].toLowerCase();
  const actual = createHash("sha256").update(fs.readFileSync(archive)).digest("hex");
  if (expected !== actual) throw new Error(`checksum mismatch: expected ${expected}, got ${actual}`);

  if (isWindows) {
    execFileSync("powershell", ["-NoProfile", "-Command", `Expand-Archive -Force '${archive}' '${vendor}'`], { stdio: "inherit" });
  } else {
    execFileSync("tar", ["-xzf", archive, "-C", vendor], { stdio: "inherit" });
    fs.chmodSync(path.join(vendor, "repo-doctor"), 0o755);
  }
}

main().catch((error) => {
  console.error(error.message);
  process.exit(1);
});
