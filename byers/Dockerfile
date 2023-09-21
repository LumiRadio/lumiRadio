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

RUN groupadd -r byers && useradd -g byers byers
RUN mkdir -p ${APP}

COPY --from=builder /app/target/release/byers ${APP}/byers
COPY ./scripts/wait-for-it.sh /usr/local/bin/wait-for-it.sh
COPY ./scripts/docker-entrypoint.sh ${APP}/docker-entrypoint.sh

RUN chmod +x /usr/local/bin/wait-for-it.sh
RUN chmod +x ${APP}/docker-entrypoint.sh
RUN chown -R byers:byers ${APP}

USER byers
WORKDIR ${APP}

EXPOSE 8000

ENTRYPOINT [ "./docker-entrypoint.sh" ]
CMD ["./byers"]
