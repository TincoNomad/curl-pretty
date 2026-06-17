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

### HTTP Mode

**Argument mode** — `pcurl` executes `curl` for you and prettifies the response:

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

# Direct URL (adds -si automatically)
pcurl https://api.example.com/users/1
```

**Pipe mode** — execute `curl` yourself and pipe output:

```bash
curl -si https://api.example.com/users/1 | pcurl
curl -si -X DELETE https://api.example.com/users/42 | pcurl
```

> `-s` silences progress bar, `-i` includes headers in stdout.

**Global flags** (work in any HTTP mode):

```bash
pcurl --body-only 'curl https://api.example.com/users/1'   # Only JSON body
pcurl --headers-only 'curl https://api.example.com/users/1' # Only headers + status
pcurl --no-color 'curl https://api.example.com/users/1'    # No colors (also respects NO_COLOR env)
```

---

### WebSocket Mode

```bash
# Native WebSocket subcommand
pcurl ws wss://echo.websocket.org
pcurl ws ws://localhost:8080/chat --verbose

# wscat-compatible alias
pcurl wscat -c wss://echo.websocket.org
pcurl wscat -c wss://echo.websocket.org --verbose
```

Features:
- **Automatic JSON prettifier** for messages
- **Colored prefixes**: `←` incoming (green), `→` outgoing (cyan)
- **Interactive**: Type messages and press Enter
- **`/quit` command** to close connection
- **Connection status** on startup
- `--verbose` shows ping/pong frames

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

### MCP Mode (Model Context Protocol)

```bash
# Basic call (session auto-created)
pcurl mcp http://localhost:8080/mcp/message tools/call '{"name":"test"}'

# With explicit session
pcurl mcp --session-id my-session http://localhost:8080/mcp/message resources/list '{}'

# See the constructed curl command
pcurl mcp --verbose http://localhost:8080/mcp/message tools/call '{"name":"test"}'
```

Features:
- **JSON-RPC body** constructed automatically (`jsonrpc`, `id`, `method`, `params`)
- **Session persistence**: `session_id` saved to `~/.config/pcurl/mcp_session`
- **SSE streaming**: MCP responses rendered in real-time with `←` prefix
- **No globbing issues**: JSON params passed as single argument, safe from zsh expansion
- `--session-id <id>` for explicit session control
- `--verbose` shows the curl command being executed

---

## CLI Reference

| Command / Flag | Description |
|---|---|
| `pcurl [FLAGS] [curl_command]` | HTTP mode (arg or pipe) |
| `pcurl ws <url> [--verbose]` | WebSocket native mode |
| `pcurl wscat -c <url> [--verbose]` | wscat-compatible alias |
| `pcurl mcp <url> <method> <params> [--session-id <id>] [--verbose]` | MCP JSON-RPC call |
| `--body-only` | Show only response body |
| `--headers-only` | Show only headers + status |
| `--no-color` | Disable colors (also via `NO_COLOR` env) |
| `--doctor` | Diagnose installation and PATH |
| `--update` | Update to latest version via install.sh |
| `--version` / `-V` | Show version (checks for updates) |
| `--help` / `-h` | Show help |

---

## Why pcurl vs alternatives

| | pcurl | xh | HTTPie | curlie |
|---|---|---|---|---|
| Accepts literal curl commands | ✓ | ✗ | ✗ | partial |
| WebSocket support | ✓ | ✗ | ✓ | ✗ |
| Single binary, no runtime | ✓ | ✓ | ✗ | ✓ |
| Written in | Rust | Rust | Python | Go |

If you already know curl and just want readable output, pcurl requires
zero new syntax. If you're comparing with xh: xh is great but doesn't
support WebSocket and requires learning its own request format.

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

PRs welcome. Code structure:

- `src/main.rs` — Entry point, CLI dispatch (clap), curl execution
- `src/cli.rs` — CLI definitions (clap derive structs)
- `src/curl_parser.rs` — curl command tokenization and reconstruction
- `src/display.rs` — HTTP response parsing and display (status, headers, body, JSON, XML)
- `src/mcp.rs` — MCP session management and JSON-RPC payload construction
- `src/ws_client.rs` — WebSocket client implementation
- `src/version.rs` — Version checking and self-update
- `src/help.rs` — Doctor diagnostic
