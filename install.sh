#!/usr/bin/env sh

set -x

TMP_INSTALL_DIR="${TMPDIR:-/tmp}/novops-install"
INSTALL_DIR="/usr/local/bin"
YELLOW='\e[0;33m'
RESET='\033[0m'

# Check required tools are available
if ! curl --version > /dev/null
then
    echo "Error: curl is not installed. Please install curl first." >&2
    exit 1
fi

if ! unzip -v > /dev/null
then
    echo "Error: unzip is not installed. Please install unzip first." >&2
    exit 1
fi

# Detect OS and Architecture
OS=$(uname | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
case $ARCH in
    x86_64)
        ARCH="x86_64"
        ;;
    aarch64)
        ARCH="aarch64"
        ;;
    arm64)
        ARCH="aarch64"  # macOS sometimes reports ARM as arm64
        ;;
    *)
        echo "Unsupported architecture: $ARCH"
        exit 1
        ;;
esac

# Define installation directory and zip file
RELEASE_FILE_PREFIX="novops_${OS}_${ARCH}"

ZIP_NAME="${RELEASE_FILE_PREFIX}.zip"
ZIP_PATH="${TMP_INSTALL_DIR}/${ZIP_NAME}"

CHECKSUM_NAME="${RELEASE_FILE_PREFIX}.sha256sum"
CHECKSUM_PATH="${TMP_INSTALL_DIR}/${CHECKSUM_NAME}"

NOVOPS_BIN_TMP_PATH="${TMP_INSTALL_DIR}/novops"

ZIP_URL="https://github.com/PierreBeucher/novops/releases/latest/download/${ZIP_NAME}"
CHECKSUM_URL="https://github.com/PierreBeucher/novops/releases/latest/download/${CHECKSUM_NAME}"

# Download and unzip the package
mkdir -p $TMP_INSTALL_DIR
curl -L "${ZIP_URL}" -o "${ZIP_PATH}"
unzip -o "${ZIP_PATH}" -d "${TMP_INSTALL_DIR}"

# Checksum
curl -L "${CHECKSUM_URL}" -o "${CHECKSUM_PATH}"
sha256sum -c "${CHECKSUM_PATH}"

if [ $? -eq 0 ]; then
    echo "Checksum verification succeeded."
else
    echo "Checksum verification failed."
    exit 1
fi

# Only need sudo to copy to install dir
if [ "$(id -u)" -eq 0 ]; then
    mv "${NOVOPS_BIN_TMP_PATH}" "${INSTALL_DIR}"
else
    sudo mv "${NOVOPS_BIN_TMP_PATH}" "${INSTALL_DIR}"
fi

rm "${ZIP_PATH}"
rm "${CHECKSUM_PATH}"

# Check if /usr/local/bin is in the PATH
# Use case statement for POSIX-compliant pattern matching
case ":${PATH}:" in
    *:/usr/local/bin:*)
        echo "Novops installed successfully."
        ;;
    *)
        echo "${YELLOW}Warning: /usr/local/bin is not in your PATH, novops commands may not work.${RESET}" >&2
        ;;
esac


