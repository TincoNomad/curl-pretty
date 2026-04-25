#!/usr/bin/env bash
set -e

INSTALL_DIR="${HOME}/.local/bin"
BINARY="curlp"
REPO="tu-usuario/curlp"  # ← Cambia esto por tu usuario de GitHub

echo ""
echo "  curlp — instalador universal"
echo "  ──────────────────────────"
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
        *) echo "  ✗ Arquitectura $ARCH no soportada para Linux"; exit 1 ;;
      esac
      ;;
    darwin)
      case $ARCH in
        x86_64) PLATFORM="macos-x64" ;;
        arm64) PLATFORM="macos-arm64" ;;
        *) echo "  ✗ Arquitectura $ARCH no soportada para macOS"; exit 1 ;;
      esac
      ;;
    *)
      echo "  ✗ Sistema operativo $OS no soportado"
      exit 1
      ;;
  esac
}

# Descargar binario precompilado
download_binary() {
  local version=$(curl -s "https://api.github.com/repos/$REPO/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  local asset_name="curlp-$PLATFORM"
  
  if [[ "$PLATFORM" == *"windows"* ]]; then
    asset_name="$asset_name.exe"
  fi
  
  echo "  → Descargando curlp $version para $PLATFORM..."
  
  local download_url="https://github.com/$REPO/releases/latest/download/$asset_name"
  
  if ! curl -fsSL "$download_url" -o "$INSTALL_DIR/$BINARY"; then
    echo "  ✗ Error descargando curlp"
    echo "    Verifica: https://github.com/$REPO/releases"
    exit 1
  fi
  
  chmod +x "$INSTALL_DIR/$BINARY"
}

# Verificar si Rust está disponible (fallback)
try_rust_install() {
  if command -v cargo &>/dev/null; then
    echo "  → Rust detectado, compilando desde source..."
    cargo build --release --quiet
    cp "target/release/$BINARY" "$INSTALL_DIR/$BINARY"
    chmod +x "$INSTALL_DIR/$BINARY"
  else
    echo "  ✗ No se pudo descargar ni compilar curlp"
    echo "    Instala Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
  fi
}

# Main
detect_platform
mkdir -p "$INSTALL_DIR"

echo "  → Plataforma detectada: $PLATFORM"

# Intentar descargar binario, fallback a compilación
if ! download_binary; then
  echo "  ⚠  Falló descarga, intentando compilación..."
  try_rust_install
fi

echo "  ✓ Instalado en $INSTALL_DIR/$BINARY"
echo ""

# Verificar PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo "  ⚠  $INSTALL_DIR no está en tu PATH."
  echo "     Agrega esto a tu .bashrc o .zshrc:"
  echo ""
  echo "     export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo ""
else
  echo "  ✓ Listo! Prueba: curlp 'curl https://httpbin.org/get'"
  echo "  ✓ O WebSocket: curlp wss://echo.websocket.org"
  echo ""
fi
