#!/bin/bash

# ==============================================================================
# Hope Genome - Hardened Setup & Verification Script
#
# This script ensures the environment is ready for a secure deployment,
# verifies the cryptographic integrity of the codebase, and builds a
# hardened Docker image with OWASP AI-SBOM labels.
#
# Status: Auditor-Ready
# ==============================================================================

set -e # Exit immediately if a command exits with a non-zero status.

# --- Helper Functions ---
print_header() {
  echo "=============================================================================="
  echo "  $1"
  echo "=============================================================================="
}

check_command() {
  if ! command -v $1 &> /dev/null; then
    echo "❌ Error: $1 is not installed. Please install it before running this script."
    exit 1
  fi
  echo "✅ $1 is available."
}


# --- 1. Environment Verification ---
print_header "Step 1: Verifying Environment"
check_command rustc
check_command cargo
check_command docker


# --- 2. Install HSM Dependencies ---
print_header "Step 2: Installing Hardware Security Module (HSM) Dependencies"
echo "This script will attempt to install HSM drivers using 'sudo apt-get'."
echo "If you are not on a Debian-based system, you may need to install these manually."

# Check if sudo is available
if ! command -v sudo &> /dev/null; then
    echo "⚠️ sudo command not found. Please install packages manually if required:"
    echo "   - libpcsclite1"
    echo "   - softhsm2"
else
    # Use sudo to install packages
    sudo apt-get update
    sudo apt-get install -y libpcsclite1 softhsm2
fi

echo "✅ HSM dependencies check/install complete."


# --- 3. Cryptographic & Security Verification ---
print_header "Step 3: Running Full Release Test Suite (131+ tests)"
echo "This verifies the cryptographic integrity and security guarantees of the codebase."

# Run all tests, including those requiring special features, in release mode.
# --all-features ensures that HSM and TEE code paths are compiled and tested.
# --release ensures that tests are run on the optimized production build.
cargo test --release --all-features -- --nocapture

echo "✅ All 131+ tests passed successfully."


# --- 4. Build Hardened Docker Image with OWASP AI-SBOM Labels ---
print_header "Step 4: Building Hardened Docker Image"
IMAGE_NAME="hope-genome-auditor:1.4.0-hardened"
BUILD_DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
VCS_REF=$(git rev-parse --short HEAD)

# See https://owasp.org/www-project-ai-bom/ for label specifications
docker build \
  --label "org.opencontainers.image.created=$BUILD_DATE" \
  --label "org.opencontainers.image.authors=Máté Róbert <stratosoiteam@gmail.com>" \
  --label "org.opencontainers.image.url=https://github.com/silentnoisehun/Hope-Genome" \
  --label "org.opencontainers.image.documentation=https://github.com/silentnoisehun/Hope-Genome/blob/main/README.md" \
  --label "org.opencontainers.image.source=https://github.com/silentnoisehun/Hope-Genome.git" \
  --label "org.opencontainers.image.version=1.4.0" \
  --label "org.opencontainers.image.revision=$VCS_REF" \
  --label "org.opencontainers.image.vendor=Hope-Genome-Project" \
  --label "org.opencontainers.image.title=Hope Genome Auditor" \
  --label "org.opencontainers.image.description=Tamper-evident cryptographic framework for AI accountability." \
  --label "org.cyclonedx.aibom.format=CycloneDX" \
  --label "org.cyclonedx.aibom.version=1.5" \
  --label "org.cyclonedx.aibom.url=file://aibom.xml" \
  -t $IMAGE_NAME .

echo "✅ Docker image '$IMAGE_NAME' built successfully."
print_header "Setup Complete. The system is ready for hardened deployment."