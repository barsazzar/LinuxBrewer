#!/usr/bin/env sh
set -e

REPO="barsazzar/LinuxBrewer"
BIN_NAME="LinuxBrewer"
INSTALL_DIR="$HOME/.local/bin"

# ── Colors ─────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GRN='\033[0;32m'
YLW='\033[1;33m'
NC='\033[0m'

info()  { printf "${GRN}[LinuxBrewer]${NC} %s\n" "$1"; }
warn()  { printf "${YLW}[LinuxBrewer]${NC} %s\n" "$1"; }
error() { printf "${RED}[LinuxBrewer]${NC} %s\n" "$1" >&2; exit 1; }

# ── OS check ───────────────────────────────────────────────────────────────────
OS="$(uname -s)"
if [ "$OS" != "Linux" ]; then
  error "macOS support is not yet available. Linux only for now."
fi

# ── Architecture ───────────────────────────────────────────────────────────────
ARCH="$(uname -m)"
case "$ARCH" in
  x86_64)  APPIMAGE_ARCH="amd64" ;;
  aarch64) APPIMAGE_ARCH="arm64" ;;
  *)       error "Unsupported architecture: $ARCH" ;;
esac

# ── WebKitGTK check ────────────────────────────────────────────────────────────
check_webkit() {
  # Check via ldconfig or pkg-config
  if ldconfig -p 2>/dev/null | grep -q 'libwebkit2gtk'; then
    return 0
  fi
  if pkg-config --exists webkit2gtk-4.1 2>/dev/null; then
    return 0
  fi
  return 1
}

if ! check_webkit; then
  warn "libwebkit2gtk-4.1 is required but not found."
  warn "Install it with:"
  if command -v apt-get >/dev/null 2>&1; then
    warn "  sudo apt-get install libwebkit2gtk-4.1-0"
  elif command -v dnf >/dev/null 2>&1; then
    warn "  sudo dnf install webkit2gtk4.1"
  elif command -v pacman >/dev/null 2>&1; then
    warn "  sudo pacman -S webkit2gtk-4.1"
  elif command -v zypper >/dev/null 2>&1; then
    warn "  sudo zypper install libwebkit2gtk-4_1-0"
  else
    warn "  Please install webkit2gtk-4.1 using your system package manager."
  fi
  error "Please install the dependency above, then re-run this script."
fi

# ── FUSE check ─────────────────────────────────────────────────────────────────
if ! (modinfo fuse >/dev/null 2>&1 || ls /dev/fuse >/dev/null 2>&1); then
  warn "FUSE not detected — will use --appimage-extract-and-run mode."
  USE_EXTRACT_RUN=1
fi

# ── Fetch latest release version ───────────────────────────────────────────────
info "Fetching latest release..."

if command -v curl >/dev/null 2>&1; then
  API_RESPONSE="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest")"
elif command -v wget >/dev/null 2>&1; then
  API_RESPONSE="$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest")"
else
  error "curl or wget is required."
fi

VERSION="$(printf '%s' "$API_RESPONSE" | grep '"tag_name"' | sed 's/.*"tag_name": *"\(.*\)".*/\1/')"
if [ -z "$VERSION" ]; then
  error "Could not determine latest version. Check your internet connection."
fi

# Strip leading 'v' for filename matching
VER="${VERSION#v}"
FILENAME="${BIN_NAME}_${VER}_${APPIMAGE_ARCH}.AppImage"
DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/${FILENAME}"

info "Installing ${BIN_NAME} ${VERSION} (${APPIMAGE_ARCH})..."

# ── Download ───────────────────────────────────────────────────────────────────
TMP_FILE="$(mktemp /tmp/${BIN_NAME}.XXXXXX.AppImage)"

if command -v curl >/dev/null 2>&1; then
  curl -fsSL --progress-bar "$DOWNLOAD_URL" -o "$TMP_FILE"
else
  wget -q --show-progress "$DOWNLOAD_URL" -O "$TMP_FILE"
fi

chmod +x "$TMP_FILE"

# ── Install ────────────────────────────────────────────────────────────────────
mkdir -p "$INSTALL_DIR"
mv "$TMP_FILE" "${INSTALL_DIR}/${BIN_NAME}"

# ── Create wrapper if FUSE not available ───────────────────────────────────────
if [ "${USE_EXTRACT_RUN:-0}" = "1" ]; then
  APPIMAGE_PATH="${INSTALL_DIR}/${BIN_NAME}.appimage"
  mv "${INSTALL_DIR}/${BIN_NAME}" "$APPIMAGE_PATH"
  cat > "${INSTALL_DIR}/${BIN_NAME}" <<EOF
#!/bin/sh
APPIMAGE_EXTRACT_AND_RUN=1 exec "${APPIMAGE_PATH}" "\$@"
EOF
  chmod +x "${INSTALL_DIR}/${BIN_NAME}"
fi

# ── PATH check ─────────────────────────────────────────────────────────────────
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *)
    warn "${INSTALL_DIR} is not in your PATH."
    warn "Add the following line to your ~/.bashrc or ~/.zshrc:"
    warn "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    warn "Then run: source ~/.bashrc"
    ;;
esac

info "Done! Run '${BIN_NAME}' to launch Brew Manager."
