FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

ENV SQLX_OFFLINE=true

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
ARG APP=/usr/src/app

# Install SSL certificates, timezone data, libpq
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libpq5 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r langley && useradd -g langley langley
RUN mkdir -p ${APP}

COPY --from=builder /app/target/release/langley ${APP}/langley

RUN chown -R langley:langley ${APP}

USER langley
WORKDIR ${APP}

EXPOSE 8000

CMD ["./langley"]