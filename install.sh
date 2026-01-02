#!/usr/bin/env bash
set -euo pipefail

if [ "${VERBOSE:-}" = "1" ]; then
  set -x
fi

REPO="${REPO:-vtxdeo/vtx-cli}"
BIN_NAME="vtx"
INSTALL_DIR="${VTX_INSTALL_DIR:-$HOME/.vtx/bin}"
NO_PATH="${NO_PATH:-}"
QUIET="${QUIET:-}"
GITHUB_TOKEN="${GITHUB_TOKEN:-}"

usage() {
  cat <<'USAGE'
Usage: curl -fsSL https://raw.githubusercontent.com/vtxdeo/vtx-cli/main/install.sh | sh

Environment variables:
  VERSION=v1.2.3    Install a specific version (default: latest)
  REPO=owner/repo   Override GitHub repo (default: vtxdeo/vtx-cli)
  VTX_INSTALL_DIR   Override install dir (default: ~/.vtx/bin)
  NO_PATH=1         Skip PATH updates
  QUIET=1           Suppress non-error output
  VERBOSE=1         Enable verbose output
  GITHUB_TOKEN=...  GitHub token to avoid API rate limits
USAGE
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "Missing required command: $1" >&2
    exit 1
  fi
}

need_cmd uname
need_cmd mktemp
need_cmd tar
need_cmd curl
need_cmd sed

log() {
  if [ -z "$QUIET" ]; then
    echo "$@"
  fi
}

if [ "${1:-}" = "-h" ] || [ "${1:-}" = "--help" ]; then
  usage
  exit 0
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
  Linux) OS="linux" ;;
  Darwin) OS="darwin" ;;
  *)
    echo "Unsupported OS: $OS" >&2
    exit 1
    ;;
 esac

case "$ARCH" in
  x86_64|amd64) ARCH="amd64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *)
    echo "Unsupported architecture: $ARCH" >&2
    exit 1
    ;;
 esac

VERSION="${VERSION:-latest}"
if [ "$VERSION" = "latest" ]; then
  API_URL="https://api.github.com/repos/$REPO/releases/latest"
  if [ -n "$GITHUB_TOKEN" ]; then
    VERSION="$(curl -fsSL -H "Authorization: Bearer $GITHUB_TOKEN" "$API_URL" | sed -n 's/.*"tag_name":"\([^"]*\)".*/\1/p')"
  else
    VERSION="$(curl -fsSL "$API_URL" | sed -n 's/.*"tag_name":"\([^"]*\)".*/\1/p')"
  fi
  if [ -z "$VERSION" ]; then
    echo "Failed to resolve latest version" >&2
    echo "Hint: set VERSION=vX.Y.Z to install a specific version." >&2
    exit 1
  fi
else
  case "$VERSION" in
    v*) : ;;
    *) VERSION="v$VERSION" ;;
  esac
fi

ASSET_NAME="${BIN_NAME}-${OS}-${ARCH}.tar.gz"
CHECKSUM_NAME="${ASSET_NAME}.sha256"
BASE_URL="https://github.com/$REPO/releases/download/$VERSION"

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "$TMP_DIR"
}
trap cleanup EXIT

ARCHIVE_PATH="$TMP_DIR/$ASSET_NAME"
CHECKSUM_PATH="$TMP_DIR/$CHECKSUM_NAME"

curl -fL "$BASE_URL/$ASSET_NAME" -o "$ARCHIVE_PATH"
curl -fL "$BASE_URL/$CHECKSUM_NAME" -o "$CHECKSUM_PATH"

verify_checksum() {
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$TMP_DIR" && sha256sum -c "$CHECKSUM_NAME")
    return $?
  fi

  if command -v shasum >/dev/null 2>&1; then
    (cd "$TMP_DIR" && shasum -a 256 -c "$CHECKSUM_NAME")
    return $?
  fi

  echo "Missing required command: sha256sum or shasum" >&2
  return 1
}

log "Verifying checksum..."
verify_checksum

EXTRACT_DIR="$TMP_DIR/extract"
mkdir -p "$EXTRACT_DIR"
tar -xzf "$ARCHIVE_PATH" -C "$EXTRACT_DIR"

mkdir -p "$INSTALL_DIR"

if [ ! -f "$EXTRACT_DIR/$BIN_NAME" ]; then
  echo "Missing binary in archive: $BIN_NAME" >&2
  exit 1
fi

ACTION="installed"
if [ -f "$INSTALL_DIR/$BIN_NAME" ]; then
  ACTION="updated"
fi

if command -v install >/dev/null 2>&1; then
  install -m 0755 "$EXTRACT_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
else
  cp "$EXTRACT_DIR/$BIN_NAME" "$INSTALL_DIR/$BIN_NAME"
  chmod 0755 "$INSTALL_DIR/$BIN_NAME"
fi

ensure_path() {
  local shell_name="$1"
  local rc_file="$2"
  local line="$3"

  if [ -f "$rc_file" ] && grep -F "$INSTALL_DIR" "$rc_file" >/dev/null 2>&1; then
    return 0
  fi

  mkdir -p "$(dirname "$rc_file")"
  touch "$rc_file"
  printf '\n%s\n' "$line" >> "$rc_file"
  log "Updated $rc_file"
}

if [ -z "$NO_PATH" ]; then
  case "${SHELL:-}" in
    */zsh)
      ensure_path "zsh" "$HOME/.zshrc" "export PATH=\"$INSTALL_DIR:\$PATH\""
      ;;
    */bash)
      if [ -f "$HOME/.bashrc" ] || [ "$OS" = "linux" ]; then
        ensure_path "bash" "$HOME/.bashrc" "export PATH=\"$INSTALL_DIR:\$PATH\""
      else
        ensure_path "bash" "$HOME/.bash_profile" "export PATH=\"$INSTALL_DIR:\$PATH\""
      fi
      ;;
    */fish)
      ensure_path "fish" "$HOME/.config/fish/config.fish" "set -gx PATH \"$INSTALL_DIR\" \$PATH"
      ;;
    *)
      echo "Add $INSTALL_DIR to your PATH to use $BIN_NAME" >&2
      ;;
  esac
fi

log "$BIN_NAME $ACTION to $INSTALL_DIR ($VERSION)"
