#!/usr/bin/env bash

set -euo pipefail

REPO_OWNER="SKKUGoon"
REPO_NAME="cli-ibkr-rs"
BINARY_NAME="ibkrctl"
DOWNLOAD_DIR="${HOME}/downloads"

if [[ -t 1 ]]; then
  C_RESET="$(printf '\033[0m')"
  C_BOLD="$(printf '\033[1m')"
  C_BLUE="$(printf '\033[34m')"
  C_GREEN="$(printf '\033[32m')"
  C_YELLOW="$(printf '\033[33m')"
  C_RED="$(printf '\033[31m')"
else
  C_RESET=""
  C_BOLD=""
  C_BLUE=""
  C_GREEN=""
  C_YELLOW=""
  C_RED=""
fi

info() { echo -e "${C_BLUE}${1}${C_RESET}"; }
warn() { echo -e "${C_YELLOW}${1}${C_RESET}"; }
success() { echo -e "${C_GREEN}${1}${C_RESET}"; }
error() { echo -e "${C_RED}${1}${C_RESET}" >&2; }

detect_target() {
  local os arch
  os="$(uname -s)"
  arch="$(uname -m)"

  case "${os}:${arch}" in
    Linux:x86_64)
      echo "x86_64-unknown-linux-gnu"
      ;;
    Darwin:arm64)
      echo "aarch64-apple-darwin"
      ;;
    *)
      error "Unsupported platform: ${os} ${arch}"
      error "Available release targets: x86_64-unknown-linux-gnu, aarch64-apple-darwin"
      exit 1
      ;;
  esac
}

TARGET="$(detect_target)"

echo -e "${C_BOLD}IBKR CLI GitHub Release Deployer${C_RESET}"
info "Target: ${TARGET}"
echo

read -r -p "Enter ibkrctl version (example: 0.1.0 or v0.1.0): " VERSION_INPUT

if [[ -z "${VERSION_INPUT}" ]]; then
  error "Version is required."
  exit 1
fi

VERSION="${VERSION_INPUT#v}"
if [[ ! "${VERSION}" =~ ^[0-9]+(\.[0-9]+){1,2}([.-][A-Za-z0-9]+)?$ ]]; then
  error "Invalid version format: ${VERSION_INPUT}"
  warn "Use versions like: 0.1.0 or v0.1.0"
  exit 1
fi

TAG="v${VERSION}"
ARCHIVE_NAME="${BINARY_NAME}-${TAG}-${TARGET}.tar.gz"
DOWNLOAD_URL="https://github.com/${REPO_OWNER}/${REPO_NAME}/releases/download/${TAG}/${ARCHIVE_NAME}"

mkdir -p "${DOWNLOAD_DIR}"
cd "${DOWNLOAD_DIR}"

echo
info "Downloading: ${DOWNLOAD_URL}"
curl -fL -o "${ARCHIVE_NAME}" "${DOWNLOAD_URL}"

info "Extracting: ${ARCHIVE_NAME}"
EXTRACT_DIR="$(mktemp -d)"
tar -xzf "${ARCHIVE_NAME}" -C "${EXTRACT_DIR}"

INSTALL_SOURCE="${EXTRACT_DIR}"
shopt -s nullglob
subdirs=("${EXTRACT_DIR}"/*/)
shopt -u nullglob
if [[ "${#subdirs[@]}" -eq 1 ]]; then
  INSTALL_SOURCE="${subdirs[0]%/}"
fi

echo
warn "Installing executables to /usr/local/bin (sudo required)..."

installed_count=0
for f in "${INSTALL_SOURCE}"/*; do
  if [[ -f "${f}" && -x "${f}" ]]; then
    sudo install -m 0755 "${f}" /usr/local/bin/
    ((installed_count += 1))
  fi
done

if [[ "${installed_count}" -eq 0 ]]; then
  error "No executable files found in extracted archive."
  exit 1
fi

info "Cleaning up downloaded files..."
rm -f "${ARCHIVE_NAME}"
rm -rf "${EXTRACT_DIR}"

echo
success "Installed ${installed_count} binary file(s) into /usr/local/bin."
success "Done."
