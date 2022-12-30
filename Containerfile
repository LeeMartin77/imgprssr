FROM docker.io/rust:1.64 as builder
WORKDIR /app
COPY . .
RUN cargo build -r

FROM docker.io/debian:stable-slim
RUN mkdir /app
RUN mkdir /images
COPY --from=builder /app/target/release/imgprssr /app/imgprssr
CMD ["/app/imgprssr"]
