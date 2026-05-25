#!/usr/bin/env sh
set -eu

repo="Kota-Ohno/repo-doctor"
version="${REPO_DOCTOR_VERSION:-latest}"
bin_dir="${REPO_DOCTOR_INSTALL_DIR:-$HOME/.local/bin}"

usage() {
  cat <<'EOF'
Install repo-doctor.

Usage:
  install.sh [--version <tag>] [--dir <path>]

Options:
  --version <tag>  Release tag to install, for example v0.1.1. Default: latest
  --dir <path>     Directory where repo-doctor is installed. Default: ~/.local/bin
  -h, --help       Show this help.
EOF
}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      if [ "$#" -lt 2 ]; then
        echo "--version requires a value" >&2
        exit 1
      fi
      version="$2"
      shift 2
      ;;
    --dir)
      if [ "$#" -lt 2 ]; then
        echo "--dir requires a value" >&2
        exit 1
      fi
      bin_dir="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
  Linux) os_target="unknown-linux-gnu" ;;
  Darwin) os_target="apple-darwin" ;;
  *) echo "unsupported OS: $os" >&2; exit 1 ;;
esac

case "$arch" in
  x86_64|amd64) arch_target="x86_64" ;;
  arm64|aarch64) arch_target="aarch64" ;;
  *) echo "unsupported architecture: $arch" >&2; exit 1 ;;
esac

target="${arch_target}-${os_target}"
if [ "$target" = "aarch64-unknown-linux-gnu" ]; then
  echo "unsupported release target: $target" >&2
  exit 1
fi

if [ -n "${REPO_DOCTOR_BASE_URL:-}" ]; then
  base="$REPO_DOCTOR_BASE_URL"
elif [ "$version" = "latest" ]; then
  base="https://github.com/$repo/releases/latest/download"
else
  base="https://github.com/$repo/releases/download/$version"
fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

mkdir -p "$bin_dir"
echo "Installing repo-doctor $version for $target"
archive="repo-doctor-$target.tar.gz"
curl -fsSL "$base/$archive" -o "$tmp/$archive"
curl -fsSL "$base/$archive.sha256" -o "$tmp/$archive.sha256"
if command -v sha256sum >/dev/null 2>&1; then
  (cd "$tmp" && sha256sum -c "$archive.sha256")
else
  expected="$(awk '{print $1}' "$tmp/$archive.sha256")"
  actual="$(shasum -a 256 "$tmp/$archive" | awk '{print $1}')"
  if [ "$expected" != "$actual" ]; then
    echo "checksum mismatch: expected $expected, got $actual" >&2
    exit 1
  fi
fi
tar -xzf "$tmp/$archive" -C "$tmp"
install -m 0755 "$tmp/repo-doctor" "$bin_dir/repo-doctor"

echo "Installed repo-doctor to $bin_dir/repo-doctor"
case ":$PATH:" in
  *":$bin_dir:"*) ;;
  *) echo "Add $bin_dir to PATH if repo-doctor is not found by your shell." ;;
esac
