#!/usr/bin/env sh
set -eu

version="${1:-v0.1.1}"
repo="${REPO_DOCTOR_REPO:-Kota-Ohno/repo-doctor}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

gh release download "$version" --repo "$repo" --pattern "*.sha256" --dir "$tmp"

linux_hash="$(awk '{print $1}' "$tmp/repo-doctor-x86_64-unknown-linux-gnu.tar.gz.sha256")"
mac_hash="$(awk '{print $1}' "$tmp/repo-doctor-x86_64-apple-darwin.tar.gz.sha256")"
mac_arm_hash="$(awk '{print $1}' "$tmp/repo-doctor-aarch64-apple-darwin.tar.gz.sha256")"
win_hash="$(awk '{print $1}' "$tmp/repo-doctor-x86_64-pc-windows-msvc.zip.sha256")"

python3 - "$version" "$linux_hash" "$mac_hash" "$mac_arm_hash" "$win_hash" <<'PY'
from pathlib import Path
import re
import sys

version, linux_hash, mac_hash, mac_arm_hash, win_hash = sys.argv[1:]
plain = version.removeprefix("v")

formula = Path("packaging/homebrew/repo-doctor.rb")
text = formula.read_text()
text = re.sub(r'version "[^"]+"', f'version "{plain}"', text, count=1)
hashes = iter([mac_arm_hash, mac_hash, linux_hash])
text = re.sub(r'sha256 "[^"]+"', lambda _: f'sha256 "{next(hashes)}"', text, count=3)
formula.write_text(text)

scoop = Path("packaging/scoop/repo-doctor.json")
text = scoop.read_text()
text = re.sub(r'"version": "[^"]+"', f'"version": "{plain}"', text, count=1)
text = re.sub(r"/v[^/]+/", f"/{version}/", text, count=1)
text = re.sub(r'"hash": "[^"]+"', f'"hash": "{win_hash}"', text, count=1)
scoop.write_text(text)
PY

echo "Updated packaging checksums for $version"
