FROM rust:1.70 as builder

ENV SQLX_OFFLINE=true

RUN USER=root cargo new --bin frohike
WORKDIR /frohike
COPY ./Cargo.toml ./Cargo.toml
RUN apt update && apt install -y libavutil-dev libavformat-dev libavfilter-dev libclang-dev
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/frohike*
RUN cargo build --release

FROM debian:bullseye-slim
ARG APP=/usr/src/app

# Install SSL certificates, timezone data, libpq, libav
RUN apt-get update \
    && apt-get install -y ca-certificates tzdata libpq5 libavutil56 libavformat58 libavfilter7 \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r frohike && useradd -g frohike frohike
RUN mkdir -p ${APP}

COPY --from=builder /frohike/target/release/frohike ${APP}/frohike

RUN chown -R frohike:frohike ${APP}

USER frohike
WORKDIR ${APP}

EXPOSE 8000

ENTRYPOINT ["./frohike"]
