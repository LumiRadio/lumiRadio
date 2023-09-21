FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder

ENV SQLX_OFFLINE=true

RUN apt update && apt install -y libavutil-dev libavformat-dev libavfilter-dev libclang-dev

COPY --from=planner /app/recipe.json recipe.json

RUN cargo chef cook --release --recipe-path recipe.json

COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
ARG APP=/usr/src/app

# Install SSL certificates, timezone data, libpq, libav
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libpq5 libavutil56 libavformat58 libavfilter7 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r frohike && useradd -g frohike frohike
RUN mkdir -p ${APP}

COPY --from=builder /app/target/release/frohike ${APP}/frohike

RUN chown -R frohike:frohike ${APP}

USER frohike
WORKDIR ${APP}

ENTRYPOINT ["./frohike"]
