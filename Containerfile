FROM docker.io/rust:1.64 as builder
WORKDIR /app
COPY . .
RUN cargo build -r
# Something broke in 20230612 - unpin when we can
FROM docker.io/debian:stable-20230522-slim
RUN apt update && apt install -y ca-certificates && rm -rf /var/lib/apt/lists/*
RUN mkdir /app
RUN mkdir /images
COPY --from=builder /app/target/release/imgprssr /app/imgprssr
CMD ["/app/imgprssr"]
