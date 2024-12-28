FROM rust:1.83 AS builder

WORKDIR /usr/src/http-to-nats-proxy
COPY . .
RUN cargo install --path .

FROM debian:bookworm-slim

# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
# RUN apt-get update && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/http-to-nats-proxy /usr/local/bin/http-to-nats-proxy
ENTRYPOINT ["http-to-nats-proxy"]
