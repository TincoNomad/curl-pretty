# AGENTS.md

## Build & Run

- `cargo build --release` - builds binary to `target/release/pcurl`
- `cargo run --bin pcurl -- [args]` - run with curl arguments (recommended)
- `curl -si <url> | cargo run --bin pcurl --` - pipe mode

## CLI Options

- `--help` / `-h` - Show help
- `--version` / `-V` - Show version (also checks for updates)
- `--doctor` / `--check` - Diagnose installation and PATH
- `--update` - Update to latest version via install.sh

## Testing

- `cargo test` - runs integration tests
- Tests are in `tests/integration_tests.rs` and spawn actual binary via `cargo run --bin pcurl --`

## Release

- Push a tag `v*` to trigger GitHub Actions release workflow
- Builds for: Linux x64/ARM64, macOS x64/ARM64, Windows x64
- Artifacts uploaded to GitHub Releases automatically

## WebSocket Support

- Built-in: `pcurl wss://<url>` or `pcurl ws://<url>`
- Also: `pcurl wscat -c wss://<url>`

## Project Structure

- `src/main.rs` - Entry point, CLI dispatch, curl execution
- `src/display.rs` - HTTP response parsing and display (status, headers, body, JSON, XML)
- `src/help.rs` - Help text and doctor diagnostic
- `src/version.rs` - Version checking and self-update
- `src/ws.rs` - WebSocket URL extraction
- `src/ws_client.rs` - WebSocket client implementation
- `src/curl_parser.rs` - curl command tokenization and reconstruction
- `tests/integration_tests.rs` - End-to-end tests
- `install.sh` - Universal installer script
- `.github/workflows/release.yml` - Multi-platform builds and releases
- `.github/workflows/quality_checks.yml` - CI: fmt, clippy, test, build on push/PR

## Dependencies

- `serde_json` - JSON parsing and formatting
- `colored` - Terminal colors
- `atty` - Stdin detection (pipe mode)
- `tokio` + `tokio-tungstenite` + `futures-util` - WebSocket async runtime
- `url` - URL parsing for WebSocket
- `openssl` (vendored) - TLS support
- `ureq` - HTTP client for version checking (GitHub API)

## Development Guidelines

- **New HTTP features**: Add to `src/display.rs` in display functions
- **New WebSocket features**: Add to `src/ws_client.rs`
- **Curl parsing issues**: Fix in `src/curl_parser.rs`
- **New CLI flags**: Add match arm in `main()`, update `print_help()`, add function
- **Always test**: Add integration tests for new features
- **Update help**: Modify `print_help()` in `src/help.rs`
- **Version checking**: Uses `ureq` to query GitHub Releases API (`check_latest_version()` in `src/version.rs`)
- **Silent update notification**: `check_for_update_notification()` in `src/version.rs` runs on every HTTP request

## Common Tasks

### Add new output format
1. Add detection in `display_body()` function in `src/display.rs`
2. Create formatting function (e.g., `print_yaml()`)
3. Add integration test in `tests/integration_tests.rs`

### Fix curl parsing
1. Check tokenization in `curl_parser.rs`
2. Test with problematic command
3. Update parser logic if needed

### Add new CLI flag
1. Add match arm in `main()` function
2. Create handler function (e.g., `print_doctor()` in `src/help.rs`)
3. Update `print_help()` in `src/help.rs` with new option
4. Add integration test if applicable

### Update version
1. Change version in `Cargo.toml`
2. Update version in `install.sh` (hardcoded in banner)
3. Tag with `git tag v1.0.1`
4. Push: `git push --tags`