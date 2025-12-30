# Hope Genome - Production Docker Image
# Multi-stage build for minimal runtime footprint

# Stage 1: Rust builder
FROM rust:1.75-slim AS builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy source files
COPY hope_core/ ./hope_core/
COPY Cargo.toml Cargo.lock ./

# Build release binary
WORKDIR /build/hope_core
RUN cargo build --release --features python-bindings

# Stage 2: Python runtime
FROM python:3.12-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy built artifacts from builder
COPY --from=builder /build/hope_core/target/release/libhope_core.so /app/
COPY --from=builder /build/hope_core/target/release/hope_core /app/

# Install Python package
RUN pip install --no-cache-dir hope-genome

# Security labels
LABEL org.opencontainers.image.title="Hope Genome"
LABEL org.opencontainers.image.description="Tamper-Evident Cryptographic Framework for AI Accountability"
LABEL org.opencontainers.image.version="1.5.0"
LABEL org.opencontainers.image.authors="stratosoiteam@gmail.com"
LABEL org.opencontainers.image.url="https://github.com/silentnoisehun/Hope_Genome"
LABEL org.opencontainers.image.documentation="https://silentnoisehun.github.io/Hope_Genome/"
LABEL org.opencontainers.image.licenses="MIT"

# Run as non-root user
RUN useradd -m -u 1000 hopegen
USER hopegen

# Default command: Python REPL with hope-genome imported
CMD ["python3", "-c", "import hope_genome as hg; print('Hope Genome v1.5.0 Ready'); print('Example: genome = hg.SealedGenome(rules=[\"Do no harm\"])')"]
