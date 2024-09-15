# Use the official Rust image as the build stage
FROM rust:alpine as builder

# Install necessary dependencies
RUN apk add --no-cache musl-dev openssl-dev pkgconfig

# Create a new directory for the application
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

# Use a minimal Alpine Linux image for the runtime stage
FROM alpine:latest

# Install necessary runtime dependencies
RUN apk add --no-cache libgcc libstdc++ openssl tini

# Copy the built application from the builder stage
COPY --from=builder /app/target/release/awtrix-traininfo /usr/local/bin/awtrix-traininfo

# Set the entrypoint to the application
ENTRYPOINT ["/sbin/tini", "--", "awtrix-traininfo"]