FROM rust:1.92 AS builder

WORKDIR /usr/src/http-to-nats-proxy
COPY . .
RUN cargo install --path .

FROM debian:trixie-slim

COPY --from=builder /usr/local/cargo/bin/http-to-nats-proxy /usr/local/bin/http-to-nats-proxy

ENTRYPOINT ["http-to-nats-proxy"]
