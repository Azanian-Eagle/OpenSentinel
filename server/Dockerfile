# Build from project root: docker build -f server/Dockerfile .
FROM rust:1.75 as builder
WORKDIR /usr/src/opensentinel-server
# Copy server code
COPY server/ .
RUN cargo install --path .

FROM debian:buster-slim
WORKDIR /app/server
COPY --from=builder /usr/local/cargo/bin/opensentinel-server /usr/local/bin/opensentinel-server
# Copy client code to ../client relative to WORKDIR /app/server
COPY client /app/client
ENV RUST_LOG=info
CMD ["opensentinel-server"]
