# Build static binary (musl) for minimal runtime image; no DB or secrets at build time.
FROM rust:1-bookworm AS builder
RUN apt-get update \
    && apt-get install -y --no-install-recommends musl-tools cmake \
    && rm -rf /var/lib/apt/lists/*
RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY api ./api
COPY src ./src
COPY .cargo ./.cargo

ENV CARGO_NET_GIT_FETCH_WITH_CLI=true
RUN cargo build --release --target x86_64-unknown-linux-musl --bin main

# Distroless static: no shell, CA store, or package manager — only what you copy in.
FROM gcr.io/distroless/static-debian12:nonroot

# HTTP listen defaults (override at `docker run` if needed).
ENV LISTEN_ADDR=0.0.0.0
ENV PORT=3000

# Database and CORS are supplied at runtime (not embedded in the image):
#   PG_DATABASE_URL   — required for the app to serve redirects
#   FRONTEND_URL      — optional CORS allow-origin

COPY --from=builder --chown=nonroot:nonroot /app/target/x86_64-unknown-linux-musl/release/main /main

USER nonroot:nonroot
EXPOSE 3000
ENTRYPOINT ["/main"]
