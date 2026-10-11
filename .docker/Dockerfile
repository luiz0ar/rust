FROM rust:1-slim AS builder

WORKDIR /usr/src/app

COPY Cargo.toml Cargo.lock ./
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/app/target/release/src /usr/local/bin/rust-app

ENV HOST=0.0.0.0 \
    PORT=3000

EXPOSE 3000

CMD ["rust-app"]
