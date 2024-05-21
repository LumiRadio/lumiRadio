FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY Cargo.toml .
COPY Cargo.lock .
COPY byers/ byers/
COPY frohike/ frohike/
COPY langley/ langley/
COPY judeharley/ judeharley/
COPY migration/ migration/
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

ENV SQLX_OFFLINE=true

RUN apt update && apt install -y libavutil-dev libavformat-dev libavfilter-dev libclang-dev

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --workspace --release --recipe-path recipe.json
COPY Cargo.toml .
COPY Cargo.lock .
COPY byers/ byers/
COPY frohike/ frohike/
COPY langley/ langley/
COPY judeharley/ judeharley/
COPY migration/ migration/
RUN cargo build --release

FROM debian:bookworm-slim AS final

COPY docker/wait-for-it.sh /usr/local/bin/wait-for-it.sh
COPY docker/docker-entrypoint.sh /usr/local/bin/docker-entrypoint.sh

RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libpq5 libavutil57 libavformat59 libavfilter8 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r lumiradio && useradd -g lumiradio lumiradio
RUN mkdir -p /opt/lumiradio/byers
RUN mkdir -p /opt/lumiradio/frohike
RUN mkdir -p /opt/lumiradio/langley
RUN chown -R lumiradio:lumiradio /opt/lumiradio/byers
RUN chown -R lumiradio:lumiradio /opt/lumiradio/frohike
RUN chown -R lumiradio:lumiradio /opt/lumiradio/langley
COPY --from=builder --chown=lumiradio:lumiradio /app/target/release/byers /opt/lumiradio/byers/byers
COPY --from=builder --chown=lumiradio:lumiradio /app/target/release/frohike /opt/lumiradio/frohike/frohike
COPY --from=builder --chown=lumiradio:lumiradio /app/target/release/langley /opt/lumiradio/langley/langley

USER lumiradio
WORKDIR /opt/lumiradio

EXPOSE 8000

ENTRYPOINT [ "/usr/local/bin/docker-entrypoint.sh" ]
