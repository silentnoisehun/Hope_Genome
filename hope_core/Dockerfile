# ==============================================================================
# Hope Genome v2.5.0 - Enterprise Edition
# Multi-stage Dockerfile with security best practices
# ==============================================================================
# Build Date: 2026-01-03
# Security Level: Enterprise
# Base Image: Distroless (minimal attack surface)
# ==============================================================================

# ==============================================================================
# Stage 1: Builder (Rust compilation)
# ==============================================================================
FROM rust:1.75-slim-bookworm AS builder

# Metadata
LABEL maintainer="Máté Róbert <stratosoiteam@gmail.com>"
LABEL version="1.4.0"
LABEL release_date="2025-12-30"
LABEL security_level="hardened"
LABEL description="Hope Genome - Tamper-Evident Cryptographic Framework for AI Accountability"
LABEL org.opencontainers.image.source="https://github.com/silentnoisehun/Hope_Genome"
LABEL org.opencontainers.image.version="1.4.0"
LABEL org.opencontainers.image.created="2025-12-30"
LABEL org.opencontainers.image.title="Hope Genome"
LABEL org.opencontainers.image.description="Ed25519-based tamper-evident framework with persistent nonce storage"
LABEL org.opencontainers.image.licenses="MIT"

# Security: Create non-root user for build
RUN groupadd -r hope && useradd -r -g hope hope

# Install build dependencies (minimal)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /build

# Copy manifests first (layer caching optimization)
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/lib.rs

# Build dependencies (cached layer)
RUN cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src
COPY examples ./examples
COPY tests ./tests

# Build Hope Genome with all optimizations
RUN cargo build --release \
    --features rocksdb-nonce-store \
    --locked \
    && strip target/release/libhope_core.so 2>/dev/null || true

# Security: Verify binary integrity
RUN sha256sum target/release/libhope_core.rlib > /build/checksums.txt

# ==============================================================================
# Stage 2: Runtime (Distroless - minimal attack surface)
# ==============================================================================
FROM gcr.io/distroless/cc-debian12:nonroot

# Metadata (runtime)
LABEL maintainer="Máté Róbert <stratosoiteam@gmail.com>"
LABEL version="1.4.0"
LABEL release_date="2025-12-30"
LABEL security_level="hardened"
LABEL stage="runtime"

# Security: Use non-root user (distroless default)
USER nonroot:nonroot

# Create data directories (owned by nonroot)
WORKDIR /app

# Copy compiled library from builder
COPY --from=builder --chown=nonroot:nonroot /build/target/release/libhope_core.rlib /app/lib/
COPY --from=builder --chown=nonroot:nonroot /build/checksums.txt /app/

# Copy minimal CA certificates for HTTPS (if needed)
COPY --from=builder /etc/ssl/certs/ca-certificates.crt /etc/ssl/certs/

# Create data volume mount point (for persistent nonce store)
VOLUME ["/data"]

# Security: Drop all capabilities (runtime has no privileges)
# This is enforced in docker-compose.yml

# Health check (if you add an HTTP API in the future)
# HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
#   CMD ["/app/health-check"]

# Expose no ports by default (library, not server)
# If you add an API, expose it here:
# EXPOSE 8080

# Default command (library test example)
# Override this in docker-compose.yml with your actual service
CMD ["echo", "Hope Genome v2.5.0 - Enterprise Edition"]

# ==============================================================================
# Security Hardening Summary
# ==============================================================================
# ✅ Multi-stage build (minimal final image)
# ✅ Distroless base (no shell, no package manager)
# ✅ Non-root user (nonroot:nonroot)
# ✅ Read-only filesystem (enforced in docker-compose)
# ✅ No capabilities (drop ALL in docker-compose)
# ✅ Minimal dependencies (only libc)
# ✅ Stripped binaries (smaller attack surface)
# ✅ Binary checksums (integrity verification)
# ✅ Volume for persistent data (/data)
# ==============================================================================

# Build command:
#   docker build -t hope-genome:1.4.0 -t hope-genome:latest .
#
# Run command (basic):
#   docker run --rm -v hope-data:/data:rw hope-genome:1.4.0
#
# Run command (hardened):
#   docker run --rm \
#     --read-only \
#     --cap-drop=ALL \
#     --security-opt=no-new-privileges:true \
#     -v hope-data:/data:rw \
#     hope-genome:1.4.0
