# Contributing

Contributions are welcome if they keep the app lightweight, local-first, and clear for non-expert Mac users.

## Local Setup

```bash
git clone https://github.com/EauZz/mac-health-monitor-rust.git
cd mac-health-monitor-rust
CARGO_TARGET_DIR=/tmp/mac-health-monitor-rust-target cargo run --release
```

## Before Opening A Pull Request

```bash
cargo fmt --check
cargo check --locked
cargo test --locked
node --check public/app.js
./build-app.sh
```

## Design Principles

- Prefer native macOS public APIs and normal user permissions.
- Do not add telemetry or remote analytics.
- Do not read LLM transcripts or private conversation files.
- Keep process explanations honest when attribution is heuristic.
- Avoid heavy frontend dependencies unless there is a strong reason.

## Good First Issues

- Add explanations for more macOS daemons.
- Improve File Provider attribution.
- Improve accessibility and responsive layout.
- Add screenshots and release packaging.
