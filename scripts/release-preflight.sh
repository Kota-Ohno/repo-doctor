#!/usr/bin/env bash
set -euo pipefail

require_tag=false
while [ "$#" -gt 0 ]; do
  case "$1" in
    --require-tag)
      require_tag=true
      shift
      ;;
    -h|--help)
      cat <<'EOF'
Run local distribution preflight checks.

Usage:
  scripts/release-preflight.sh [--require-tag]

Checks:
  - package version consistency
  - optional local tag presence
  - release build and local archive checksum
  - install.sh against file:// release assets
  - npm wrapper installer against file:// release assets
  - repo-doctor guard
EOF
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 1
      ;;
  esac
done

need() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

need cargo
need curl
need git
need tar

node_bin="${NODE:-node}"
if ! command -v "$node_bin" >/dev/null 2>&1; then
  echo "missing required command: node" >&2
  echo "set NODE=/path/to/node when Node.js is installed outside PATH" >&2
  exit 1
fi

if command -v sha256sum >/dev/null 2>&1; then
  checksum_cmd=sha256sum
elif command -v shasum >/dev/null 2>&1; then
  checksum_cmd="shasum -a 256"
else
  echo "missing required command: sha256sum or shasum" >&2
  exit 1
fi

version="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
npm_version="$("$node_bin" -p 'require("./packaging/npm/package.json").version')"
action_version="$(sed -n 's/^    default: \(v[0-9].*\)$/\1/p' action.yml | head -n1)"
tag="v${version}"

if [ "$npm_version" != "$version" ]; then
  echo "version mismatch: Cargo.toml=${version}, packaging/npm/package.json=${npm_version}" >&2
  exit 1
fi

if [ "$action_version" != "$tag" ]; then
  echo "version mismatch: action.yml default=${action_version}, expected ${tag}" >&2
  exit 1
fi

if [ "$require_tag" = true ] && ! git rev-parse -q --verify "refs/tags/${tag}" >/dev/null; then
  echo "missing local tag: ${tag}" >&2
  exit 1
fi

case "$(uname -s)" in
  Linux) os_target="unknown-linux-gnu" ;;
  Darwin) os_target="apple-darwin" ;;
  *) echo "unsupported preflight OS: $(uname -s)" >&2; exit 1 ;;
esac

case "$(uname -m)" in
  x86_64|amd64) arch_target="x86_64" ;;
  arm64|aarch64) arch_target="aarch64" ;;
  *) echo "unsupported preflight architecture: $(uname -m)" >&2; exit 1 ;;
esac

target="${arch_target}-${os_target}"
if [ "$target" = "aarch64-unknown-linux-gnu" ]; then
  echo "unsupported release target: ${target}" >&2
  exit 1
fi

mkdir -p target
tmp="$(mktemp -d "${PWD}/target/release-preflight.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

echo "==> Building release binary for ${target}"
cargo build --release --locked --target "$target"

dist="$tmp/dist"
mkdir -p "$dist"
cp "target/${target}/release/repo-doctor" "$dist/repo-doctor"
archive="repo-doctor-${target}.tar.gz"
tar -C "$dist" -czf "$tmp/$archive" repo-doctor
(
  cd "$tmp"
  $checksum_cmd "$archive" > "$archive.sha256"
)

echo "==> Testing install.sh against local release assets"
REPO_DOCTOR_BASE_URL="file://$tmp" sh scripts/install.sh --version "$tag" --dir "$tmp/bin"
"$tmp/bin/repo-doctor" --version

echo "==> Testing npm wrapper against local release assets"
cp -R packaging/npm "$tmp/npm"
node_script="$tmp/npm/scripts/install.js"
node_base_url="file://$tmp"
if [ "${node_bin##*.}" = "exe" ] && command -v wslpath >/dev/null 2>&1; then
  node_script="$(wslpath -w "$node_script")"
  node_base_url="file:///$(wslpath -w "$tmp" | sed 's#\\#/#g')"
fi
REPO_DOCTOR_BASE_URL="$node_base_url" REPO_DOCTOR_TARGET="$target" "$node_bin" "$node_script"
npm_bin="$tmp/npm/vendor/repo-doctor"
if [ ! -x "$npm_bin" ] && [ -x "$tmp/npm/vendor/repo-doctor.exe" ]; then
  npm_bin="$tmp/npm/vendor/repo-doctor.exe"
fi
"$npm_bin" --version

echo "==> Running repo-doctor guard"
cargo run -- guard --format compact

echo "release preflight ok for ${tag} (${target})"
