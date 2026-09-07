# Build from project root: docker build -f server/Dockerfile .
FROM rust:1 as builder
WORKDIR /usr/src/opensentinel-server
# Copy server code
COPY server/ .
RUN cargo install --path .

FROM debian:bookworm-slim
WORKDIR /app/server
RUN apt-get update \
	&& apt-get install -y --no-install-recommends ca-certificates curl \
	&& rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/opensentinel-server /usr/local/bin/opensentinel-server
# Ship the runtime model alongside the service binary.
COPY server/model.onnx /app/server/model.onnx
# Copy client code to ../client relative to WORKDIR /app/server
COPY client /app/client
ENV RUST_LOG=info
EXPOSE 8080
CMD ["opensentinel-server"]
