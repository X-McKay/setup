#!/usr/bin/env bash
# Install the released setup CLI from GitHub Releases.
#
# Usage:
#   curl -fsSL https://github.com/X-McKay/setup/releases/latest/download/install.sh | bash
#   curl -fsSL https://github.com/X-McKay/setup/releases/download/v0.3.0/install.sh | \
#     bash -s -- --version v0.3.0

set -euo pipefail

REPO="${SETUP_GITHUB_REPO:-X-McKay/setup}"
INSTALL_DIR="${SETUP_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${SETUP_VERSION:-latest}"
VERIFY_CHECKSUMS=1
RELEASE_BASE_URL="${SETUP_RELEASE_BASE_URL:-}"

info() { printf '==> %s\n' "$1"; }
warn() { printf 'warning: %s\n' "$1" >&2; }
error() { printf 'error: %s\n' "$1" >&2; }

usage() {
  cat <<'EOF'
Install the released setup CLI from GitHub Releases.

Options:
  --version <tag>      Install a specific release tag (for example: v0.3.0)
  --install-dir <dir>  Install into this directory (default: ~/.local/bin)
  --repo <owner/name>  Override the GitHub repo (default: X-McKay/setup)
  --no-verify          Skip SHA256 verification
  -h, --help           Show this help

Environment overrides:
  SETUP_VERSION
  SETUP_INSTALL_DIR
  SETUP_GITHUB_REPO
  SETUP_RELEASE_BASE_URL   Override the release asset base URL for testing
EOF
}

require_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    error "required command not found: $1"
    exit 1
  fi
}

normalize_version() {
  if [[ "$1" == "latest" ]]; then
    printf '%s\n' "$1"
  elif [[ "$1" == v* ]]; then
    printf '%s\n' "$1"
  else
    printf 'v%s\n' "$1"
  fi
}

detect_platform() {
  local os arch

  case "$(uname -s)" in
    Linux) os="linux" ;;
    *)
      error "unsupported operating system: $(uname -s)"
      exit 1
      ;;
  esac

  case "$(uname -m)" in
    x86_64 | amd64) arch="x86_64" ;;
    aarch64 | arm64) arch="aarch64" ;;
    *)
      error "unsupported architecture: $(uname -m)"
      exit 1
      ;;
  esac

  printf '%s %s\n' "$os" "$arch"
}

build_base_url() {
  if [[ -n "$RELEASE_BASE_URL" ]]; then
    printf '%s\n' "${RELEASE_BASE_URL%/}"
    return
  fi

  local normalized
  normalized="$(normalize_version "$VERSION")"

  if [[ "$normalized" == "latest" ]]; then
    printf 'https://github.com/%s/releases/latest/download\n' "$REPO"
  else
    printf 'https://github.com/%s/releases/download/%s\n' "$REPO" "$normalized"
  fi
}

download_asset() {
  local url=$1
  local output=$2

  if ! curl -fsSL -o "$output" "$url"; then
    error "failed to download $url"
    exit 1
  fi
}

verify_checksum() {
  local tmpdir=$1
  local asset_name=$2
  local checksum_file=$3
  local match_file=$tmpdir/checksum-match.txt

  if ! grep -E "[[:space:]]$asset_name$" "$checksum_file" >"$match_file"; then
    error "checksum entry for $asset_name not found in $(basename "$checksum_file")"
    exit 1
  fi

  (
    cd "$tmpdir"
    sha256sum -c "$match_file"
  )
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version)
      VERSION=${2:?missing value for --version}
      shift 2
      ;;
    --install-dir)
      INSTALL_DIR=${2:?missing value for --install-dir}
      shift 2
      ;;
    --repo)
      REPO=${2:?missing value for --repo}
      shift 2
      ;;
    --no-verify)
      VERIFY_CHECKSUMS=0
      shift
      ;;
    -h | --help)
      usage
      exit 0
      ;;
    *)
      error "unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

INSTALL_DIR="${INSTALL_DIR/#\~/$HOME}"

require_cmd curl
require_cmd tar
require_cmd install
if [[ "$VERIFY_CHECKSUMS" -eq 1 ]]; then
  require_cmd sha256sum
fi

read -r os arch <<<"$(detect_platform)"
asset_name="setup-${os}-${arch}.tar.gz"
checksum_name="checksums.txt"
base_url="$(build_base_url)"

tmpdir="$(mktemp -d)"
trap 'rm -rf "$tmpdir"' EXIT

tarball="$tmpdir/$asset_name"
checksums="$tmpdir/$checksum_name"

info "Downloading $asset_name"
download_asset "$base_url/$asset_name" "$tarball"

if [[ "$VERIFY_CHECKSUMS" -eq 1 ]]; then
  info "Verifying SHA256 checksum"
  download_asset "$base_url/$checksum_name" "$checksums"
  verify_checksum "$tmpdir" "$asset_name" "$checksums"
else
  warn "skipping checksum verification"
fi

info "Installing setup into $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
tar -xzf "$tarball" -C "$tmpdir"

if [[ ! -f "$tmpdir/setup" ]]; then
  error "release archive did not contain a setup binary"
  exit 1
fi

install -m 0755 "$tmpdir/setup" "$INSTALL_DIR/setup"

info "Installed: $INSTALL_DIR/setup"
"$INSTALL_DIR/setup" --version || true

case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    warn "$INSTALL_DIR is not on PATH"
    printf 'Add this to your shell config:\n  export PATH="%s:$PATH"\n' "$INSTALL_DIR"
    ;;
esac
