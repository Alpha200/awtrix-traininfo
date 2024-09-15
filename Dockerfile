FROM rust:alpine3.20 AS builder

ENV RUSTFLAGS="-Ctarget-feature=-crt-static"

RUN apk add --no-cache musl-dev openssl-dev

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

COPY src ./src

RUN cargo build --release

FROM alpine:3.20

RUN apk add --no-cache libgcc libstdc++ openssl tini

COPY --from=builder /app/target/release/awtrix-traininfo /usr/local/bin/awtrix-traininfo

ENTRYPOINT ["/sbin/tini", "--", "awtrix-traininfo"]