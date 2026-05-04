# Mac Health Monitor Rust

A lightweight native macOS dashboard for checking the health of a Mac without opening Activity Monitor.

It is built in Rust with a tiny local HTTP server and a native WKWebView window through `wry`/`tao`. The app focuses on fast startup, low overhead, and readable explanations for opaque macOS processes like `WindowServer`, `fileproviderd`, `cloudd`, `com.apple.WebKit.GPU`, Rosetta, and local LLM tooling.

## Features

- Native macOS app window, not a browser tab.
- CPU, memory pressure, disk, battery, network, uptime, and system health cards.
- Rolling 5-minute Process Watch for CPU, RAM, thermal impact, sleeping-heavy apps, and Rosetta suspects.
- Human-readable process explanations for common macOS daemons and hidden app helpers.
- Best-effort File Provider attribution for iCloud Drive, Adobe Creative Cloud, OneDrive, Dropbox, Google Drive, Box, Nextcloud, Synology Drive, and Proton Drive.
- Local LLM activity monitor for Claude, Codex, and Gemini processes.
- Optional OpenUsage cache reading for quota/token summaries when OpenUsage is installed locally.
- Light, warm UI designed for quick scanning on a MacBook display.

## Privacy Model

The app runs locally on `127.0.0.1` and does not send telemetry to a remote server.

It reads macOS command-line tools and local files that are already available to the current user. For LLM usage, it can read the OpenUsage cache at:

```text
~/Library/Application Support/com.sunstory.openusage/usage-api-cache.json
```

It does not read Claude/Codex/Gemini conversation transcripts.

## Limitations

- Apple Silicon does not expose precise CPU temperature in Celsius to normal sandbox-free apps. The app shows a thermal index and macOS thermal state unless privileged tools expose more.
- Safari/WebKit does not expose reliable CPU/RAM attribution per tab through public APIs. WebKit processes are explained as Safari/webview activity, but not mapped to exact tabs.
- Process explanations are best-effort heuristics. They are designed to be useful, not to pretend macOS exposes every private owner relationship.
- The generated app is not notarized. If distributed as a binary, users may need to approve it in macOS Gatekeeper settings.

## Requirements

- macOS 13 or later.
- Rust stable with edition 2024 support.
- Xcode Command Line Tools.
- `sips` and `iconutil` for app icon generation, both available on macOS.

## Run From Source

```bash
git clone https://github.com/EauZz/mac-health-monitor-rust.git
cd mac-health-monitor-rust
CARGO_TARGET_DIR=/tmp/mac-health-monitor-rust-target cargo run --release
```

The internal server uses port `8767` by default and automatically falls back to a free port if needed.

## Build The macOS App

```bash
./build-app.sh
open "dist/Mac Health Monitor Rust.app"
```

The build script creates:

```text
dist/Mac Health Monitor Rust.app
```

You can customize the output:

```bash
APP_NAME="Mac Health Monitor" \
BUNDLE_ID="dev.yourname.MacHealthMonitor" \
OUT_DIR="$PWD/dist" \
./build-app.sh
```

## Development Checks

```bash
cargo fmt --check
cargo check --locked
cargo test --locked
node --check public/app.js
./build-app.sh
```

## Repository Layout

```text
src/main.rs          Rust app, local server, macOS metrics collectors
public/             UI assets served inside the native window
assets/             App icon source assets
build-app.sh        Portable macOS .app bundle builder
PRODUCT.md          Product notes
DESIGN.md           Design direction
```

## License

MIT. See [LICENSE](LICENSE).
