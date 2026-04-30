#!/usr/bin/env bash
set -e

INSTALL_DIR="${HOME}/.local/bin"
BINARY="pcurl"
REPO="TincoNomad/pretty-curl"

print_banner() {
  echo -e "\033[90m┌───────────────────────────┐\033[0m"
  echo -e "\033[90m│\033[95m\033[1m █▀█ █▀▀ █▀▀ █ ▀▀█ ▀▀▀     \033[0m\033[90m  │\033[0m"
  echo -e "\033[90m│\033[95m\033[1m █▀▀ █▀  █▀▀ █  █   █      \033[0m\033[90m  │\033[0m"
  echo -e "\033[90m│\033[95m\033[1m ▀   ▀▀▀ ▀▀▀ ▀  ▀   ▀      \033[0m\033[90m  │\033[0m"
  echo -e "\033[90m│\033[96m\033[1m █▀▀ █ █ █▀▄ █             \033[0m\033[90m  │\033[0m"
  echo -e "\033[90m│\033[96m\033[1m █   █ █ █▀▄ █             \033[0m\033[90m  │\033[0m"
  echo -e "\033[90m│\033[96m\033[1m ▀▀▀ ▀▀▀ ▀ ▀ ▀▀▀           \033[0m\033[90m  │\033[0m"
  echo -e "\033[90m└───────────────────────────┘\033[0m"
  echo ""
}

print_banner
echo "  Make your HTTP requests beautiful 💅 ✦  v1.3.0"
echo "  ───────────────────────────────────────────────"
echo ""

# Detectar sistema operativo y arquitectura
detect_platform() {
  OS=$(uname -s | tr '[:upper:]' '[:lower:]')
  ARCH=$(uname -m)
  
  case $OS in
    linux)
      case $ARCH in
        x86_64) PLATFORM="linux-x64" ;;
        aarch64) PLATFORM="linux-arm64" ;;
        arm64) PLATFORM="linux-arm64" ;;
        *) echo "  ❌ Architecture $ARCH not supported for Linux"; exit 1 ;;
      esac
      ;;
    darwin)
      case $ARCH in
        x86_64) PLATFORM="macos-x64" ;;
        arm64) PLATFORM="macos-arm64" ;;
        *) echo "  ❌ Architecture $ARCH not supported for macOS"; exit 1 ;;
      esac
      ;;
    *)
      echo "  ❌ Operating system $OS not supported"
      exit 1
      ;;
  esac
}

# Descargar binario precompilado
download_binary() {
  local version=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  local asset_name="pcurl-$PLATFORM"
  
  if [[ "$PLATFORM" == *"windows"* ]]; then
    asset_name="$asset_name.exe"
  fi
  
  echo "  ➡️  Downloading pcurl $version for $PLATFORM..."
  
  local download_url="https://github.com/$REPO/releases/latest/download/$asset_name"
  
  if ! curl -fsSL "$download_url" -o "$INSTALL_DIR/$BINARY"; then
    echo "  ❌ Error downloading pcurl"
    echo "    Check: https://github.com/$REPO/releases"
    exit 1
  fi
  
  chmod +x "$INSTALL_DIR/$BINARY"
}

# Verificar si Rust está disponible (fallback)
try_rust_install() {
  if command -v cargo &>/dev/null; then
    echo "  ➡️ Rust detected, compiling from source..."
    cargo build --release --quiet
    cp "target/release/$BINARY" "$INSTALL_DIR/$BINARY"
    chmod +x "$INSTALL_DIR/$BINARY"
  else
    echo "  ❌ Could not download nor compile pcurl"
    echo "    Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
  fi
}

# Main
detect_platform
mkdir -p "$INSTALL_DIR"

echo "  ➡️  Platform detected: $PLATFORM"

# Intentar descargar binario, fallback a compilación
if ! download_binary; then
  echo "  ⚠️  Download failed, trying compilation..."
  try_rust_install
fi

echo "  ✅ Installed in $INSTALL_DIR/$BINARY"
echo ""

# Verificar PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo "  ⚠️  $INSTALL_DIR is not in your PATH."
  echo "     Add this to your .bashrc or .zshrc:"
  echo ""
  echo "     export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo ""
else
  echo "  ✅ Ready! Test: pcurl 'curl https://httpbin.org/get'"
  echo "  ✅ Or WebSocket: pcurl wss://echo.websocket.org"
  echo ""
fi
