#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
bin="${REPO_DOCTOR_BIN:-$repo_root/target/debug/repo-doctor}"

if [ ! -x "$bin" ]; then
  cargo build --manifest-path "$repo_root/Cargo.toml"
fi

tmp="$(mktemp -d "${TMPDIR:-/tmp}/repo-doctor-profile-smoke.XXXXXX")"
trap 'rm -rf "$tmp"' EXIT

write_common() {
  mkdir -p "$1"
  printf '# Demo\n' > "$1/README.md"
  printf 'MIT\n' > "$1/LICENSE"
  printf 'target\nnode_modules\n' > "$1/.gitignore"
}

run_case() {
  name="$1"
  shift
  echo "==> $name"
  "$bin" check "$tmp/$name" --format compact "$@"
}

write_common "$tmp/generic"
run_case generic --profile generic

write_common "$tmp/node"
cat > "$tmp/node/package.json" <<'JSON'
{"name":"demo","version":"0.1.0","description":"Demo","license":"MIT","repository":"https://example.com/demo","packageManager":"pnpm@11.3.0","scripts":{"test":"node --test","build":"vite build"},"engines":{"node":">=20"},"dependencies":{"vite":"latest"}}
JSON
printf "lockfileVersion: '9.0'\n" > "$tmp/node/pnpm-lock.yaml"
mkdir -p "$tmp/node/src"
run_case node --profile auto

write_common "$tmp/python"
cat > "$tmp/python/pyproject.toml" <<'TOML'
[project]
name = "demo"
version = "0.1.0"
description = "Demo"
readme = "README.md"
license = "MIT"

[build-system]
requires = ["hatchling"]

[tool.hatch.envs.default.scripts]
test = "python -m unittest"
TOML
printf '\n' > "$tmp/python/uv.lock"
run_case python --profile auto

write_common "$tmp/go/apps/api"
cat > "$tmp/go/apps/api/go.mod" <<'EOF'
module example.com/demo

go 1.22
EOF
run_case go --profile auto

write_common "$tmp/dotnet"
mkdir -p "$tmp/dotnet/src/Demo" "$tmp/dotnet/tests/Demo.Tests"
printf '\n' > "$tmp/dotnet/Demo.slnx"
printf '<Project />\n' > "$tmp/dotnet/src/Demo/Demo.csproj"
printf '<Project />\n' > "$tmp/dotnet/tests/Demo.Tests/Demo.Tests.csproj"
run_case dotnet --profile auto

write_common "$tmp/jvm"
cat > "$tmp/jvm/pom.xml" <<'XML'
<project><modelVersion>4.0.0</modelVersion><parent><groupId>com.example</groupId><artifactId>parent</artifactId><version>1</version></parent><artifactId>demo</artifactId></project>
XML
printf '#!/bin/sh\n' > "$tmp/jvm/mvnw"
run_case jvm --profile auto

write_common "$tmp/php"
cat > "$tmp/php/composer.json" <<'JSON'
{"name":"example/demo","description":"Demo","license":"MIT","type":"library","require":{"php":"^8.3"},"scripts":{"test":"phpunit"}}
JSON
run_case php --profile auto

write_common "$tmp/ruby"
mkdir -p "$tmp/ruby/config"
printf "require 'rails'\n" > "$tmp/ruby/config/application.rb"
printf "source 'https://rubygems.org'\ngem 'rails'\n" > "$tmp/ruby/Gemfile"
printf '\n' > "$tmp/ruby/Gemfile.lock"
run_case ruby --profile auto

write_common "$tmp/cpp"
printf '\n' > "$tmp/cpp/WORKSPACE.bazel"
printf -- '-std=c++20\n' > "$tmp/cpp/compile_flags.txt"
printf '\n' > "$tmp/cpp/conanfile.py"
run_case cpp --profile auto

write_common "$tmp/swift"
printf '// swift-tools-version: 6.0\nlet package = Package(name: "Demo", targets: [.executableTarget(name: "Demo")])\n' > "$tmp/swift/Package.swift"
run_case swift --profile auto

write_common "$tmp/kotlin"
printf 'plugins { id("com.android.application") version "8.0.0" }\n' > "$tmp/kotlin/build.gradle.kts"
mkdir -p "$tmp/kotlin/app/src/androidMain/kotlin"
run_case kotlin --profile auto

write_common "$tmp/docker"
printf 'FROM alpine:3.20\nCMD ["true"]\n' > "$tmp/docker/Dockerfile"
printf 'target\n' > "$tmp/docker/.dockerignore"
run_case docker --profile auto

write_common "$tmp/iac/envs/dev"
printf 'terraform { required_providers {} }\n' > "$tmp/iac/envs/dev/providers.tf"
printf '\n' > "$tmp/iac/envs/dev/.terraform.lock.hcl"
run_case iac --profile auto

write_common "$tmp/docs"
printf 'site_name: Demo\n' > "$tmp/docs/mkdocs.yml"
mkdir -p "$tmp/docs/docs"
run_case docs --profile auto

echo "profile smoke ok"
