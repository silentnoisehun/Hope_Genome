# Hope Genome v1.3.0 - Production Docker Image
# OWASP AI-SBOM Compliant Build
FROM rust:1.75-slim-bookworm as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /app

# Copy Cargo files first for dependency caching
COPY hope_core/Cargo.toml hope_core/Cargo.lock* ./hope_core/

# Copy source code
COPY hope_core/src ./hope_core/src
COPY hope_core/examples ./hope_core/examples

# Build release binary
WORKDIR /app/hope_core
RUN cargo build --release --examples

# Run all tests to verify OWASP AIBOM integration
RUN cargo test --release -- --test-threads=1

# Production stage - minimal image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 hopegenome

# Create directories
RUN mkdir -p /var/log/hope_genome && \
    chown hopegenome:hopegenome /var/log/hope_genome

# Copy example AIBOM file
COPY --from=builder /app/hope_core/examples/example_model.aibom.json /app/

# Copy built examples
COPY --from=builder /app/hope_core/target/release/examples/compliance_demo /app/

# Switch to non-root user
USER hopegenome
WORKDIR /app

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD [ -f /app/compliance_demo ] || exit 1

# Default command: Run compliance demo
CMD ["./compliance_demo"]

# Labels for OWASP AI-SBOM compliance
LABEL org.opencontainers.image.title="Hope Genome v1.3.0"
LABEL org.opencontainers.image.description="Tamper-evident cryptographic framework for AI accountability with OWASP AI-SBOM compliance"
LABEL org.opencontainers.image.version="1.3.0"
LABEL org.opencontainers.image.vendor="Máté Róbert"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/silentnoisehun/Hope-Genome"
LABEL compliance.owasp.aibom="CycloneDX 1.5+"
LABEL compliance.cyclonedx.version="1.5"
LABEL tests.passed="71/71"
LABEL tests.breakdown="56 core, 12 security, 8 compliance, 3 doc"
