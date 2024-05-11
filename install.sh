#!/usr/bin/env sh

TMP_INSTALL_DIR="${TMPDIR:-/tmp}/novops-install"
INSTALL_DIR="/usr/local/bin"
YELLOW='\e[0;33m'
RESET='\033[0m'

#
# From (and credit to) https://github.com/client9/shlib
#

#
# is_command: returns true if command exists
#
# `which` is not portable, in particular is often
# not available on RedHat/CentOS systems.
#
# `type` is implemented in many shells but technically not
# part of the posix spec.
#
# `command -v`
#
is_command() {
  command -v "$1" >/dev/null
  #type "$1" > /dev/null 2> /dev/null
}

# hash_sha256_verify validates a binary against a checksum.txt file
hash_sha256_verify() {
  TARGET=$1
  checksums=$2

  if [ -z "$checksums" ]; then
    echo "hash_sha256_verify checksum file not specified in arg2"  >&2
    return 1
  fi

  # http://stackoverflow.com/questions/2664740/extract-file-basename-without-path-and-extension-in-bash
  BASENAME=${TARGET##*/}

  want=$(grep "${BASENAME}" "${checksums}" 2>/dev/null | tr '\t' ' ' | cut -d ' ' -f 1)

  # if file does not exist $want will be empty
  if [ -z "$want" ]; then
    echo "hash_sha256_verify unable to find checksum for '${TARGET}' in '${checksums}'"  >&2
    return 1
  fi
  got=$(hash_sha256 "$TARGET")
  if [ "$want" != "$got" ]; then
    echo "hash_sha256_verify checksum for '$TARGET' did not verify ${want} vs $got"  >&2
    return 1
  fi
}

# hash_sha256: compute SHA256 of $1 or stdin
#
# ## Example
#
# ```bash
# $ hash_sha256 foobar.tar.gz
# 237982738471928379137
# ```
#
# note lack of pipes to make sure errors are
# caught regardless of shell settings
# sha256sum NOFILE | cut ...
# won't fail unless setpipefail is on
#
hash_sha256() {
  TARGET=${1:-/dev/stdin}
  if is_command gsha256sum; then
    # mac homebrew, others
    hash=$(gsha256sum "$TARGET") || return 1
    echo "$hash" | cut -d ' ' -f 1
  elif is_command sha256sum; then
    # gnu, busybox
    hash=$(sha256sum "$TARGET") || return 1
    echo "$hash" | cut -d ' ' -f 1
  elif is_command shasum; then
    # darwin, freebsd?
    hash=$(shasum -a 256 "$TARGET" 2>/dev/null) || return 1
    echo "$hash" | cut -d ' ' -f 1
  elif is_command openssl; then
    hash=$(openssl -dst openssl dgst -sha256 "$TARGET") || return 1
    echo "$hash" | cut -d ' ' -f a
  else
    log_crit "hash_sha256 unable to find command to compute sha-256 hash"
    return 1
  fi
}

# End https://github.com/client9/shlib


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
case $OS in
    darwin)
        OS_RELEASE="macos"
        ;;
    linux)
        OS_RELEASE="linux"
        ;;
    *)
        echo "Unsupported OS: $OS"
        exit 1
        ;;
esac

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
RELEASE_FILE_PREFIX="novops_${OS_RELEASE}_${ARCH}"

ZIP_NAME="${RELEASE_FILE_PREFIX}.zip"
ZIP_PATH="${TMP_INSTALL_DIR}/${ZIP_NAME}"

CHECKSUM_NAME="${RELEASE_FILE_PREFIX}.sha256sum"
CHECKSUM_PATH="${TMP_INSTALL_DIR}/${CHECKSUM_NAME}"

NOVOPS_BIN_TMP_PATH="${TMP_INSTALL_DIR}/novops"

ZIP_URL="https://github.com/PierreBeucher/novops/releases/latest/download/${ZIP_NAME}"
CHECKSUM_URL="https://github.com/PierreBeucher/novops/releases/latest/download/${CHECKSUM_NAME}"

echo "Downloading Novops release..."

mkdir -p $TMP_INSTALL_DIR
curl -s -L "${ZIP_URL}" -o "${ZIP_PATH}"
curl -s -L "${CHECKSUM_URL}" -o "${CHECKSUM_PATH}"

echo "Extracting and veryfing checksum..."

unzip -q -o "${ZIP_PATH}" -d "${TMP_INSTALL_DIR}"
hash_sha256_verify "${NOVOPS_BIN_TMP_PATH}" "${CHECKSUM_PATH}"

if [ $? -eq 0 ]; then
    echo "Checksum verification succeeded."
else
    echo "Checksum verification failed."
    exit 1
fi

# Only need sudo to copy to install dir
if [ "$(id -u)" -eq 0 ]; then
    echo "Copying to ${INSTALL_DIR}..."
    mkdir -p $INSTALL_DIR
    mv "${NOVOPS_BIN_TMP_PATH}" "${INSTALL_DIR}"
else
    echo "Copying to ${INSTALL_DIR}... (you may be prompted for sudo password)"
    sudo mkdir -p $INSTALL_DIR
    sudo mv "${NOVOPS_BIN_TMP_PATH}" "${INSTALL_DIR}"
fi

echo "Cleanup temporary files..."

rm "${ZIP_PATH}"
rm "${CHECKSUM_PATH}"

# Check install dir is in the PATH
# Use case statement for POSIX-compliant pattern matching
case ":${PATH}:" in
    *:${INSTALL_DIR}:*)
        novops --version
        echo "Novops has been successfully installed ✨"
        ;;
    *)
        echo -e "${YELLOW}Warning: ${INSTALL_DIR} is not in your PATH, novops commands may not work.${RESET}" >&2
        ;;
esac

