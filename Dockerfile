FROM rustlang/rust:nightly-bookworm-slim AS base
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev make libsqlite3-dev

FROM base AS client-builder
WORKDIR /usr/app
COPY client/Cargo.toml client/Dioxus.toml client/Makefile ./
COPY ./client/src ./src/
RUN make clean \
    && BUILD_FLAGS=--release make

FROM base AS server-builder
WORKDIR /usr/app
COPY server/Cargo.toml server/Cargo.lock server/Makefile ./
COPY ./server/src ./src/
COPY ./.git  ./.git/
RUN make clean \
    && BUILD_FLAGS=--release make

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
COPY --from=server-builder /usr/app/target/release/imgfloat /usr/local/bin/imgfloat
COPY ./migrations /usr/share/imgfloat/migrations
COPY ./client /var/www/imgfloat
COPY ./entrypoint /usr/local/bin/docker-entrypoint
ENTRYPOINT ["/usr/local/bin/docker-entrypoint"]
