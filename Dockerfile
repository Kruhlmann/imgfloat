FROM rustlang/rust:nightly-bookworm-slim AS builder

WORKDIR /usr/app
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev
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
