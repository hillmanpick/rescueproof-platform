FROM rust:1.96-bookworm AS builder
WORKDIR /app
ARG CARGO_BUILD_JOBS=2
ENV CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS}
COPY Cargo.toml Cargo.lock ./
COPY backend ./backend
RUN cargo build --locked --release --bin rescueproof

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/* && useradd --system --uid 10001 app
WORKDIR /app
COPY --from=builder /app/target/release/rescueproof /usr/local/bin/rescueproof
USER 10001
EXPOSE 3000
ENTRYPOINT ["rescueproof"]
CMD ["serve"]
