# Build stage
FROM rust:alpine AS builder

# Install build dependencies for Alpine
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /app

# Add musl target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Set environment variables for OpenSSL static linking
ENV OPENSSL_STATIC=1
ENV OPENSSL_DIR=/usr

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Copy source code
COPY src/ src/
COPY migrations/ migrations/
COPY build.rs build.rs

# Build application for musl target
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest

# Install runtime dependencies
RUN apk add --no-cache ca-certificates wget

WORKDIR /app

# Copy binary from build stage
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/stellar-cartography /app/stellar-cartography

COPY data/stellar.db data/stellar.db
RUN mkdir -p data/cache

# Create non-root user
RUN addgroup -g 1000 appuser && \
    adduser -D -s /bin/sh -u 1000 -G appuser appuser && \
    chown -R appuser:appuser /app

# set database permissions to allow read/write to new user
RUN chown appuser:appuser data/stellar.db
RUN chown appuser:appuser data/cache

USER appuser

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info

# Run the binary
CMD ["./stellar-cartography"] 