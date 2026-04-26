# curlp

HTTP pretty-printer for your terminal. Takes raw `curl` output and makes it readable — like Postman or Bruno, but without leaving your terminal.

```
  ✓  HTTP/2 200 OK    142ms

  HEADERS
  ──────────────────────────────────────────────────────
  Content-Type                   application/json
  X-Request-Id                   abc-def-123

  BODY
  ──────────────────────────────────────────────────────
{
  "id": 42,
  "nombre": "Juan Pérez",
  "permisos": [
    "read",
    "write",
    "delete"
  ]
}
```

## Installation

### Option 1 — Universal Installer (recommended)

**No Rust needed. Downloads precompiled binary for your system:**

```bash
curl -sSL https://raw.githubusercontent.com/tinconomad/curl-pretty/main/install.sh | bash
```

The installer automatically detects:
- Linux (x64, ARM64)
- macOS (Intel, Apple Silicon)
- Downloads correct binary from GitHub Releases
- Fallback to compilation if no binary available

### Option 2 — Manual Download

Go to [GitHub Releases](https://github.com/tinconomad/curl-pretty/releases) and download:

- `curlp-linux-x64` - Linux 64-bit
- `curlp-linux-arm64` - Linux ARM64
- `curlp-macos-x64` - macOS Intel
- `curlp-macos-arm64` - macOS Apple Silicon
- `curlp-windows-x64.exe` - Windows 64-bit

Then:
```bash
# Linux/macOS
chmod +x curlp-*
sudo cp curlp-* /usr/local/bin/curlp

# Windows
# Move curlp.exe to a directory in your PATH
```

### Option 3 — Compile from Source

If you have Rust installed:

```bash
git clone https://github.com/tinconomad/curl-pretty
cd curl-pretty
cargo build --release
sudo cp target/release/curlp /usr/local/bin/
```

---

## Usage

### Mode 1 — Argument Mode (recommended)

`curlp` executes `curl` for you and prettifies the response:

```bash
# Simple GET
curlp 'curl https://api.example.com/users/1'

# POST with JSON
curlp 'curl -X POST https://api.example.com/users \
  -H "Authorization: Bearer <token>" \
  -d '"'{"name":"Juan","role":"admin"}'"'"''

# With extra flags
curlp 'curl -L -k https://api.internal.com/health'
curlp 'curl -u user:password https://api.example.com/private'
```

### Mode 2 — Pipe

If you prefer to execute `curl` yourself, use `-si` and pipe:

```bash
curl -si https://api.example.com/users/1 | curlp
curl -si -X DELETE https://api.example.com/users/42 | curlp
```

> `-s` silences progress bar, `-i` includes headers in stdout.

### Recommended Aliases

Add this to your `.bashrc` / `.zshrc`:

```bash
# Prettify any curl automatically
curlp() { command curlp "$@"; }

# Or a shorter alias
alias cget='curlp curl'
alias cpost='curlp curl -X POST'
```

---

## What it shows

| Element | Description |
|---|---|
| **Status** | Code + text, colored: 🟢 2xx · 🟡 3xx · 🔴 4xx/5xx |
| **Time** | Request milliseconds (argument mode) |
| **Headers** | Aligned key + value, with key in cyan |
| **Body JSON** | Indented with colors: strings green, numbers yellow, booleans magenta, null red |
| **Body XML** | Tree indented with cyan tags |
| **Body text** | Plain, unmodified |

---

## WebSocket

**New! Integrated WebSocket support:**

```bash
# Connect to WebSocket URL
curlp wss://echo.websocket.org
curlp ws://localhost:8080/chat

# wscat-style commands also work
curlp wscat -c wss://echo.websocket.org
```

Features:
- **Automatic JSON prettifier** for messages
- **Colored prefixes**: `←` incoming (green), `→` outgoing (cyan)
- **Interactive**: Type messages and press Enter
- **`/quit` command** to close connection
- **Connection status** on startup

Example session:
```
↔ wss://echo.websocket.org
────────────────────────────────────────────────────────────
✓ Connected! (HTTP 101)
────────────────────────────────────────────────────────────
Type messages and press Enter. /quit to exit.

> hello world
← "hello world"

> {"type":"ping","timestamp":123456}
← {
     "type": "ping",
     "timestamp": 123456
  }
```

---

## Contributing

PRs welcome. Code is in two modules:
- `src/main.rs` — HTTP response parser and rendering
- `src/curl_parser.rs` — curl command tokenization and reconstruction
