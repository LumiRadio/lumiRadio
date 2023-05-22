FROM rust:1.67 as builder

ENV SQLX_OFFLINE=true

RUN USER=root cargo new --bin langley
WORKDIR /langley
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/langley*
RUN cargo build --release

FROM debian:buster-slim
ARG APP=/usr/src/app

# Install SSL certificates, timezone data, libpq
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libpq5 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r langley && useradd -g langley langley
RUN mkdir -p ${APP}

COPY --from=builder /langley/target/release/langley ${APP}/langley

RUN chown -R langley:langley ${APP}

USER langley
WORKDIR ${APP}

EXPOSE 8000

CMD ["./langley"]