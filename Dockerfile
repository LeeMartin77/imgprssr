FROM rust:1.98-alpine AS builder
RUN apk add --no-cache musl-dev build-base openssl-dev openssl-libs-static pkgconfig
ENV OPENSSL_STATIC=1
WORKDIR /app
COPY . .
RUN cargo build --release
FROM alpine:3.24.2 AS runner
RUN apk add --no-cache ca-certificates libssl3
RUN mkdir /app
RUN mkdir /images
COPY --from=builder /app/target/release/imgprssr /app/imgprssr
CMD ["/app/imgprssr"]
