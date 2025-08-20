FROM rust:1.72-alpine AS build

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /usr/src/sidecar
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
COPY src ./src
RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache libgcc openssl tini

COPY --from=build /usr/src/sidecar/target/release/sidecar /usr/sidecar
COPY entrypoint.sh /usr/entrypoint.sh
RUN chmod +x /usr/entrypoint.sh

WORKDIR /usr

ENTRYPOINT ["/sbin/tini", "--", "/usr/entrypoint.sh"]
