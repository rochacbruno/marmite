FROM rust:slim-bookworm AS builder

RUN apt-get update -y && \
    apt-get install -y make g++ pkg-config libssl-dev && \
    rustup target add x86_64-unknown-linux-gnu

WORKDIR /build
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-gnu

FROM debian:bookworm-slim

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates curl && \
    install -m 0755 -d /etc/apt/keyrings && \
    curl -fsSL https://download.docker.com/linux/debian/gpg -o /etc/apt/keyrings/docker.asc && \
    chmod a+r /etc/apt/keyrings/docker.asc && \
    echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.asc] https://download.docker.com/linux/debian bookworm stable" \
      > /etc/apt/sources.list.d/docker.list && \
    apt-get update && \
    apt-get install -y --no-install-recommends docker-ce-cli && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /build/target/x86_64-unknown-linux-gnu/release/marmite-playground /app/marmite-playground
COPY static/ /app/static/
COPY seed/ /app/seed/

ENV PLAYGROUND_BIND=0.0.0.0:3000
ENV MARMITE_RENDER_MODE=docker
ENV MARMITE_DOCKER_IMAGE=ghcr.io/rochacbruno/marmite:latest
ENV PLAYGROUND_SESSIONS_DIR=/tmp/marmite-playground
ENV RUST_LOG=marmite_playground=info

EXPOSE 3000

ENTRYPOINT ["/app/marmite-playground"]
