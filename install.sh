#!/usr/bin/env bash
set -e

INSTALL_DIR="${HOME}/.local/bin"
BINARY="curlp"

echo ""
echo "  curlp — instalador"
echo "  ──────────────────"
echo ""

# Verificar Rust
if ! command -v cargo &>/dev/null; then
  echo "  ✗ Rust no está instalado."
  echo "    Instálalo con:"
  echo "    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
  echo ""
  exit 1
fi

echo "  → Compilando en modo release..."
cargo build --release --quiet

mkdir -p "$INSTALL_DIR"
cp "target/release/$BINARY" "$INSTALL_DIR/$BINARY"
chmod +x "$INSTALL_DIR/$BINARY"

echo "  ✓ Instalado en $INSTALL_DIR/$BINARY"
echo ""

# Verificar que está en PATH
if ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
  echo "  ⚠  $INSTALL_DIR no está en tu PATH."
  echo "     Agrega esto a tu .bashrc o .zshrc:"
  echo ""
  echo "     export PATH=\"\$HOME/.local/bin:\$PATH\""
  echo ""
else
  echo "  ✓ Listo! Prueba: curlp 'curl https://httpbin.org/get'"
  echo ""
fi
