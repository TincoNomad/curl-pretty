# pcurl

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
curl -sSL https://raw.githubusercontent.com/TincoNomad/pretty-curl/main/install.sh | bash
```

The installer automatically detects:
- Linux (x64, ARM64)
- macOS (Intel, Apple Silicon)
- Downloads correct binary from GitHub Releases
- Fallback to compilation if no binary available

### Option 2 — Manual Download

Go to [GitHub Releases](https://github.com/TincoNomad/pretty-curl/releases) and download:

- `pcurl-linux-x64` - Linux 64-bit
- `pcurl-linux-arm64` - Linux ARM64
- `pcurl-macos-x64` - macOS Intel
- `pcurl-macos-arm64` - macOS Apple Silicon
- `pcurl-windows-x64.exe` - Windows 64-bit

Then:
```bash
# Linux/macOS
chmod +x pcurl-*
sudo cp pcurl-* /usr/local/bin/pcurl

# Windows
# Move pcurl.exe to a directory in your PATH
```

### Option 3 — Compile from Source

If you have Rust installed:

```bash
git clone https://github.com/TincoNomad/pretty-curl
cd pretty-curl
cargo build --release
sudo cp target/release/pcurl /usr/local/bin/
```

---

## Usage

### Mode 1 — Argument Mode (recommended)

`pcurl` executes `curl` for you and prettifies the response:

```bash
# Simple GET
pcurl 'curl https://api.example.com/users/1'

# POST with JSON
pcurl 'curl -X POST https://api.example.com/users \
  -H "Authorization: Bearer <token>" \
  -d '"'{"name":"Juan","role":"admin"}'"'"''

# With extra flags
pcurl 'curl -L -k https://api.internal.com/health'
pcurl 'curl -u user:password https://api.example.com/private'
```

### Mode 2 — Pipe

If you prefer to execute `curl` yourself, use `-si` and pipe:

```bash
curl -si https://api.example.com/users/1 | pcurl
curl -si -X DELETE https://api.example.com/users/42 | pcurl
```

> `-s` silences progress bar, `-i` includes headers in stdout.

### Recommended Aliases

Add this to your `.bashrc` / `.zshrc`:

```bash
# Prettify any curl automatically
pcurl() { command pcurl "$@"; }

# Or a shorter alias
alias cget='pcurl curl'
alias cpost='pcurl curl -X POST'
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
pcurl wss://echo.websocket.org
pcurl ws://localhost:8080/chat

# wscat-style commands also work
pcurl wscat -c wss://echo.websocket.org
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

## Self-Update

`pcurl` checks for newer versions automatically:

- **On `--version`**: Shows current version and alerts if a newer one exists
- **On every HTTP request**: Silent background check; prints a notice if outdated
- **`--update`**: Runs the installer to update to the latest release
- **`--doctor`**: Diagnoses installation issues (binary location, PATH, curl dependency, connectivity)

---

## Security Notice

⚠️ **This tool is designed for local development and testing only.**

- **Auto-update**: Downloads and executes code from GitHub. If you don't trust this, use manual download or compile from source.
- **Pipe mode**: `pcurl` only reads from stdin and displays output — it does not execute arbitrary code.
- **Argument mode**: Passes arguments directly to `curl`. Be careful with untrusted input.
- **For production use**: Review the code and consider security implications before use in sensitive environments.

### Safe Usage Guidelines

✅ Safe: Testing your own APIs locally  
✅ Safe: Connecting to trusted internal services  
⚠️ Caution: Using with `-k` / `--insecure` in production  
❌ Avoid: Piping untrusted network data directly to `pcurl`

---

## Contributing

PRs welcome. Code is in three modules:
- `src/main.rs` — HTTP response parser, rendering, version checking, self-update, doctor diagnostic
- `src/curl_parser.rs` — curl command tokenization and reconstruction
- `src/ws_client.rs` — WebSocket client implementation
