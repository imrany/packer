#!/bin/sh
set -e

BINARY_NAME="packer"
INSTALL_DIR="/usr/local/bin"
TARGET_PATH="${INSTALL_DIR}/${BINARY_NAME}"

echo "Uninstalling ${BINARY_NAME}..."

if [ ! -f "${TARGET_PATH}" ]; then
    echo "⚠️  ${BINARY_NAME} was not found at ${TARGET_PATH}."
    exit 0
fi

if [ -w "${INSTALL_DIR}" ]; then
    rm -f "${TARGET_PATH}"
else
    echo "Elevated permissions required to remove ${TARGET_PATH}"
    sudo rm -f "${TARGET_PATH}"
fi

echo ""
echo "✅ Successfully uninstalled ${BINARY_NAME}!"
