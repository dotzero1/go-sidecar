FROM rust:1.89-alpine AS build

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /usr/src/sidecar
COPY Cargo.toml Cargo.lock ./
# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
# Remove the dummy build artifacts and copy real source
RUN rm -rf src target/release/sidecar
COPY src ./src
# Build the actual application
RUN cargo build --release

FROM alpine:latest

RUN apk add --no-cache libgcc openssl

COPY --from=build /usr/src/sidecar/target/release/sidecar /usr/sidecar

WORKDIR /usr

ENTRYPOINT ["/usr/sidecar"]
