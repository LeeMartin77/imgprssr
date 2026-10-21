# ---- Builder stage ----
FROM rust:1.98-alpine AS builder

# Install build dependencies (musl-dev for static linking, openssl for crypto)
RUN apk add --no-cache musl-dev build-base openssl-dev openssl-libs-static pkgconfig

ENV OPENSSL_STATIC=1

WORKDIR /app

# Copy manifest files first for layer caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src
COPY src/ src/

# Build release binary
RUN cargo build --release

# ---- Runner stage ----
FROM alpine:3.24.2 AS runner

# Install runtime dependencies
RUN apk add --no-cache ca-certificates libssl3

# Create non-root user and group
RUN addgroup -S appuser && adduser -S appuser -G appuser

# Create application directories with proper ownership
RUN mkdir -p /app/images && chown -R appuser:appuser /app

# Copy binary from builder
COPY --from=builder --chown=appuser:appuser /app/target/release/imgprssr /app/imgprssr

# Switch to non-root user
USER appuser

# Expose port (documented as configurable via env)
EXPOSE 3000

WORKDIR /app

CMD ["/app/imgprssr"]
