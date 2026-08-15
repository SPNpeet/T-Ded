# ---- build web (PWA + engine WASM) ----
FROM rust:1-bookworm AS wasmbuild
WORKDIR /src
RUN cargo install wasm-pack --locked
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN wasm-pack build crates/aqua-engine --target web --release \
    --out-dir /out/engine-pkg --out-name aqua_engine -- --features wasm

FROM node:22-bookworm-slim AS webbuild
WORKDIR /web
COPY web/package.json web/package-lock.json ./
RUN npm ci
COPY web/ ./
COPY --from=wasmbuild /out/engine-pkg ./src/engine-pkg
RUN node scripts/brand.mjs && npm run build

# ---- build server ----
FROM rust:1-bookworm AS serverbuild
WORKDIR /src
COPY Cargo.toml Cargo.lock ./
COPY crates ./crates
RUN cargo build --release -p teedet-server

# ---- runtime ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=serverbuild /src/target/release/teedet-server /app/teedet-server
COPY --from=webbuild /web/dist /app/web/dist
ENV WEB_DIR=/app/web/dist \
    DATABASE_URL=sqlite:///data/teedet.db \
    PORT=8080 \
    RUST_LOG=info,sqlx=warn
VOLUME ["/data"]
EXPOSE 8080
CMD ["/app/teedet-server"]
