FROM rust:latest as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p api

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/api .
COPY config.toml .
CMD ["./api"]
