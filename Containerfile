FROM rust:1.98-alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /app
COPY . .
RUN cargo build --release
FROM alpine:latest AS runner
RUN apk add --no-cache ca-certificates
RUN mkdir /app
RUN mkdir /images
COPY --from=builder /app/target/release/imgprssr /app/imgprssr
CMD ["/app/imgprssr"]
