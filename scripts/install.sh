#!/bin/sh
set -e

# ==========================================
# CONFIGURATION
# ==========================================
REPO="imrany/packer"
BINARY_NAME="packer"
INSTALL_DIR="/usr/local/bin"

# Detect OS
OS="$(uname -s)"
case "${OS}" in
    Linux*)     OS_NAME="unknown-linux-gnu";;
    Darwin*)    OS_NAME="apple-darwin";;
    *)          echo "Error: Unsupported OS '${OS}'"; exit 1;;
esac

# Detect Architecture
ARCH="$(uname -m)"
case "${ARCH}" in
    x86_64|amd64) ARCH_NAME="x86_64";;
    arm64|aarch64)
        if [ "${OS_NAME}" = "apple-darwin" ]; then
            ARCH_NAME="aarch64"
        else
            echo "Error: ARM64 Linux builds are not yet provided."; exit 1
        fi
        ;;
    *)          echo "Error: Unsupported architecture '${ARCH}'"; exit 1;;
esac

TARGET="${ARCH_NAME}-${OS_NAME}"
TARBALL_NAME="${BINARY_NAME}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO}/releases/latest/download/${TARBALL_NAME}"

echo "Detected target: ${TARGET}"
echo "Downloading ${BINARY_NAME}..."

# Create temporary directory for extraction
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "${TMP_DIR}"' EXIT

# Download tarball
if command -v curl >/dev/null 2>&1; then
    curl -fsSL "${DOWNLOAD_URL}" -o "${TMP_DIR}/${TARBALL_NAME}"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "${TMP_DIR}/${TARBALL_NAME}" "${DOWNLOAD_URL}"
else
    echo "Error: Neither curl nor wget is installed."
    exit 1
fi

echo "Extracting archive..."
tar -xzf "${TMP_DIR}/${TARBALL_NAME}" -C "${TMP_DIR}"

echo "Installing ${BINARY_NAME} to ${INSTALL_DIR}..."
if [ -w "${INSTALL_DIR}" ]; then
    mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
else
    echo "Elevated permissions required to write to ${INSTALL_DIR}"
    sudo mv "${TMP_DIR}/${BINARY_NAME}" "${INSTALL_DIR}/${BINARY_NAME}"
fi

chmod +x "${INSTALL_DIR}/${BINARY_NAME}"

echo ""
echo "✅ Successfully installed ${BINARY_NAME}!"
echo "Run '${BINARY_NAME} --help' to verify the installation."
