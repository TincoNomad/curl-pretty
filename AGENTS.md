# AGENTS.md

## Build & Run

- `cargo build --release` - builds binary to `target/release/curlp`
- `cargo run --bin curlp -- [args]` - run with curl arguments (recommended)
- `curl -si <url> | cargo run --bin curlp --` - pipe mode

## Testing

- `cargo test` - runs integration tests
- Tests are in `tests/integration_tests.rs` and spawn actual binary via `cargo run --bin curlp --`

## Release

- Push a tag `v*` to trigger GitHub Actions release workflow
- Builds for: Linux x64/ARM64, macOS x64/ARM64, Windows x64
- Artifacts uploaded to GitHub Releases automatically

## WebSocket Support

- Built-in: `curlp wss://<url>` or `curlp ws://<url>`
- Also: `curlp wscat -c wss://<url>`

## Project Structure

- `src/main.rs` - HTTP response parsing, display logic, WebSocket detection
- `src/ws_client.rs` - WebSocket client implementation
- `src/curl_parser.rs` - curl command tokenization and reconstruction
- `tests/integration_tests.rs` - End-to-end tests
- `install.sh` - Universal installer script
- `.github/workflows/release.yml` - Multi-platform builds and releases

## Development Guidelines

- **New HTTP features**: Add to `src/main.rs` in display functions
- **New WebSocket features**: Add to `src/ws_client.rs`
- **Curl parsing issues**: Fix in `src/curl_parser.rs`
- **Always test**: Add integration tests for new features
- **Update help**: Modify `print_help()` in `src/main.rs`

## Common Tasks

### Add new output format
1. Add detection in `display_body()` function
2. Create formatting function (e.g., `print_yaml()`)
3. Add integration test in `tests/integration_tests.rs`

### Fix curl parsing
1. Check tokenization in `curl_parser.rs`
2. Test with problematic command
3. Update parser logic if needed

### Update version
1. Change version in `Cargo.toml`
2. Tag with `git tag v1.0.1`
3. Push: `git push --tags`