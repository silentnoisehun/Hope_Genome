# ============================================================
# HOPE GENOME - DOCKER IMAGE v1.7.1
# ============================================================
#
# Vas Szigora Edition - AI Accountability in One Container!
#
# Usage:
#   docker build -t hope-genome .
#   docker run -it hope-genome              # Interactive mode
#   docker run hope-genome demo             # Run demo
#   docker run hope-genome audit            # Blockchain audit demo
#   docker run hope-genome watchdog         # Watchdog demo
#   docker run hope-genome test             # Run tests
#
# Mate Robert + Claude
# VAS SZIGORA - 2026.01.01.
# ============================================================

# Simple Python image (no Rust build needed - using PyPI package)
FROM python:3.11-slim

# Labels
LABEL org.opencontainers.image.title="Hope Genome"
LABEL org.opencontainers.image.description="Tamper-Evident Cryptographic Framework for AI Accountability - VAS SZIGORA Edition"
LABEL org.opencontainers.image.version="1.7.1"
LABEL org.opencontainers.image.authors="Mate Robert <stratosoiteam@gmail.com>"
LABEL org.opencontainers.image.url="https://github.com/silentnoisehun/Hope_Genome"
LABEL org.opencontainers.image.documentation="https://silentnoisehun.github.io/Hope_Genome/"
LABEL org.opencontainers.image.licenses="MIT"
LABEL org.opencontainers.image.source="https://github.com/silentnoisehun/Hope_Genome"

# Environment
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
ENV HOPE_GENOME_DOCKER=1
ENV HOPE_GENOME_VERSION=1.7.1

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*

# Install Hope Genome from PyPI
RUN pip install --no-cache-dir hope-genome requests

# Copy demo files
COPY demo/*.py /app/demo/
COPY demo/*.md /app/demo/
COPY hope_core/python/hope_genome/blockchain_audit.py /app/blockchain_audit.py

# Copy entrypoint
COPY docker-entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

# Create directories
RUN mkdir -p /app/audit_chain /app/results

# Create non-root user for security
RUN useradd -m -u 1000 hopegen && \
    chown -R hopegen:hopegen /app
USER hopegen

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD python -c "import hope_genome; print('OK')" || exit 1

# Expose volume for audit logs
VOLUME ["/app/audit_chain"]

# Entrypoint
ENTRYPOINT ["/app/entrypoint.sh"]
CMD ["help"]
