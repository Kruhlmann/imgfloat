FROM rustlang/rust:nightly-bookworm-slim AS base
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev make libsqlite3-dev

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
COPY Cargo.toml Cargo.lock Makefile ./
COPY ./src ./src/
COPY ./.git  ./.git/
RUN BUILD_FLAGS=--release make

FROM debian:bookworm-slim
RUN mkdir -p /var/www/imgfloat /usr/share/imgfloat /etc/imgfloat \
    && apt-get update \
    && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl-dev \
    libsqlite3-0 \
    curl \
    xz-utils \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates \
    && curl \
    --proto '=https' \
    --tlsv1.2 \
    -LsSf \
    https://github.com/diesel-rs/diesel/releases/latest/download/diesel_cli-installer.sh \
    | sh \
    && ln -sf /root/.cargo/bin/diesel /usr/local/bin/diesel \
    && printf '[migrations_directory]\ndir = "/usr/share/imgfloat/migrations"\n' >/etc/imgfloat/diesel.toml
COPY --from=builder /usr/app/target/release/imgfloat /usr/local/bin/imgfloat
COPY ./migrations /usr/share/imgfloat/migrations
COPY ./client /var/www/imgfloat
COPY ./entrypoint /usr/local/bin/docker-entrypoint
ENTRYPOINT ["/usr/local/bin/docker-entrypoint"]
