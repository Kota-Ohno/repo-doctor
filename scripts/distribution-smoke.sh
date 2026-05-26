#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin="${REPO_DOCTOR_BIN:-$repo_root/target/debug/repo-doctor}"

if [ ! -x "$bin" ]; then
  cargo build --manifest-path "$repo_root/Cargo.toml"
fi

echo "==> Binary and local preflight"
"$bin" --version
"$bin" preflight "$repo_root" --format json >/dev/null

echo "==> Generated CI templates"
for template in generic rust node python go deno bun jvm dotnet php ruby swift cpp docker iac docs; do
  snippet="$("$bin" ci --template "$template" --version v9.9.9)"
  case "$snippet" in
    *"Kota-Ohno/repo-doctor@v9.9.9"*) ;;
    *) echo "generated template missing repo-doctor action: $template" >&2; exit 1 ;;
  esac
done
guard_snippet="$("$bin" ci --guard --version v9.9.9)"
case "$guard_snippet" in
  *"guard --base"*) ;;
  *) echo "generated guard template missing guard command" >&2; exit 1 ;;
esac

echo "==> Install script help"
sh "$repo_root/scripts/install.sh" --help >/dev/null
if command -v pwsh >/dev/null 2>&1; then
  pwsh -NoProfile -File "$repo_root/scripts/install.ps1" -Help >/dev/null
elif command -v powershell >/dev/null 2>&1; then
  powershell -NoProfile -File "$repo_root/scripts/install.ps1" -Help >/dev/null
fi

echo "==> npm wrapper"
if command -v node >/dev/null 2>&1; then
  REPO_DOCTOR_BIN="$bin" node "$repo_root/packaging/npm/bin/repo-doctor.js" --version >/dev/null
else
  echo "node not found; skipping npm wrapper smoke"
fi

echo "==> Action metadata"
grep -q "using: composite" "$repo_root/action.yml"
grep -q "inputs:" "$repo_root/action.yml"
grep -q "version:" "$repo_root/action.yml"

echo "==> Optional Docker build"
if command -v docker >/dev/null 2>&1 && docker version >/dev/null 2>&1; then
  docker build -t repo-doctor-smoke "$repo_root" >/dev/null
else
  echo "docker daemon unavailable; skipping Docker image smoke"
fi

echo "distribution smoke ok"
