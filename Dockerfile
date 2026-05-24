FROM rust:1-slim AS builder
WORKDIR /src
COPY . .
RUN cargo build --release --locked

FROM debian:bookworm-slim
RUN useradd --create-home --uid 10001 repo-doctor
COPY --from=builder /src/target/release/repo-doctor /usr/local/bin/repo-doctor
USER repo-doctor
WORKDIR /repo
ENTRYPOINT ["repo-doctor"]
CMD ["check"]
