# Hope Genome v1.4.0 - Hardened Security Edition - Production Docker Image
# OWASP AI-SBOM Compliant Build
# Date: 2025-12-30
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

# Production stage - distroless for minimal attack surface
FROM gcr.io/distroless/cc-debian12:nonroot

# Copy example AIBOM file
COPY --from=builder --chown=nonroot:nonroot /app/hope_core/examples/example_model.aibom.json /app/

# Copy built examples
COPY --from=builder --chown=nonroot:nonroot /app/hope_core/target/release/examples/compliance_demo /app/

# Distroless runs as nonroot user by default (UID 65532)
WORKDIR /app

# Default command: Run compliance demo
CMD ["/app/compliance_demo"]

# Labels for OWASP AI-SBOM compliance
LABEL org.opencontainers.image.title="Hope Genome v1.4.0 - Hardened Security Edition"
LABEL org.opencontainers.image.description="Tamper-evident cryptographic framework with Ed25519, persistent nonces, and OWASP AI-SBOM compliance"
LABEL org.opencontainers.image.version="1.4.0"
LABEL org.opencontainers.image.created="2025-12-30"
LABEL org.opencontainers.image.vendor="Máté Róbert"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/silentnoisehun/Hope_Genome"
LABEL org.opencontainers.image.base.name="gcr.io/distroless/cc-debian12:nonroot"
LABEL compliance.owasp.aibom="CycloneDX 1.5+"
LABEL compliance.cyclonedx.version="1.5"
LABEL security.edition="Hardened"
LABEL security.crypto="Ed25519"
LABEL security.marvin_attack="Eliminated"
LABEL tests.passed="79/79"
LABEL tests.breakdown="67 core, 12 security"
