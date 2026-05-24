#!/usr/bin/env node
const { spawnSync } = require("node:child_process");
const path = require("node:path");

const exe = process.platform === "win32" ? "repo-doctor.exe" : "repo-doctor";
const bin = process.env.REPO_DOCTOR_BIN || path.join(__dirname, "..", "vendor", exe);
const result = spawnSync(bin, process.argv.slice(2), { stdio: "inherit" });

if (result.error) {
  console.error(result.error.message);
  process.exit(1);
}

process.exit(result.status ?? 1);
