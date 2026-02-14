# Multi-stage Dockerfile for Artemis Service Registry
FROM rust:1.85 as builder

WORKDIR /app
COPY . .

# Build release binary
RUN cargo build --release -p artemis

# Runtime stage
FROM debian:bookworm-slim

# Install CA certificates for HTTPS
RUN apt-get update && \
    apt-get install -y ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/artemis /usr/local/bin/artemis

# Create config directory
RUN mkdir -p /etc/artemis

# Expose default port
EXPOSE 8080

# Run artemis server
CMD ["artemis", "server", "--addr", "0.0.0.0:8080"]
