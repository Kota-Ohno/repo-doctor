set dotenv-load := false

default:
    just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all --check

lint:
    cargo clippy --all-targets --all-features -- -D warnings

test:
    cargo nextest run

doc-test:
    cargo test --doc

audit:
    cargo audit
    cargo deny check

typos:
    typos

toml:
    taplo fmt --check

check: fmt-check lint test doc-test audit typos toml

smoke-template:
    rm -rf "$HOME/src/template-smoke-cli"
    scripts/new-from-template.sh template-smoke-cli
    cd "$HOME/src/template-smoke-cli" && cargo fmt --all --check
    cd "$HOME/src/template-smoke-cli" && cargo clippy --all-targets --all-features -- -D warnings
    cd "$HOME/src/template-smoke-cli" && cargo nextest run
    cd "$HOME/src/template-smoke-cli" && cargo run -- check
    rm -rf "$HOME/src/template-smoke-cli"
