FROM rust:1.70 as builder

ENV SQLX_OFFLINE=true

RUN USER=root cargo new --bin byers
WORKDIR /byers
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/byers*
RUN cargo build --release

FROM debian:bullseye-slim
ARG APP=/usr/src/app

# Install SSL certificates, timezone data, libpq
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libpq5 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r byers && useradd -g byers byers
RUN mkdir -p ${APP}

COPY --from=builder /byers/target/release/byers ${APP}/byers
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