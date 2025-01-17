FROM rustlang/rust:nightly-bookworm-slim AS base
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev

FROM base AS cache
ENV CARGO_HOME=/cargo
WORKDIR /usr/dependencies
RUN mkdir /cargo && cargo new --lib cache
WORKDIR /usr/dependencies/cache
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

FROM base AS builder
ENV CARGO_HOME=/cargo
WORKDIR /usr/app
COPY --from=cache /cargo /cargo
COPY Cargo.toml Cargo.lock ./
COPY ./src ./src/
COPY ./.git  ./.git/
RUN cargo build --release

FROM debian:bookworm-slim
RUN mkdir -p /var/www/imgfloat \
    && apt-get update && apt-get install -y --no-install-recommends libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates
COPY --from=builder /usr/app/target/release/imgfloat /usr/local/bin/imgfloat
COPY ./client /var/www/imgfloat
ENTRYPOINT [ "/usr/local/bin/imgfloat" ]
