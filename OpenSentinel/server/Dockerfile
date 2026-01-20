# Build from project root: docker build -f server/Dockerfile .
FROM rust:1.75 as builder
WORKDIR /usr/src/opensentinel-server
# Copy server code
COPY server/ .
RUN cargo install --path .

FROM debian:buster-slim
COPY --from=builder /usr/local/cargo/bin/opensentinel-server /usr/local/bin/opensentinel-server
# Copy client code
COPY client /client
ENV RUST_LOG=info
CMD ["opensentinel-server"]
