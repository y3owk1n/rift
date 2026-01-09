# AGENTS.md

This document provides guidelines for AI agents working on the **Rift** window manager (Rust/macOS).

**Binary**: `rift` (main daemon with CLI via clap) | **Edition**: Rust 2024

## Build Commands

```bash
cargo build                      # Debug build
cargo build --release            # Release build with optimizations
cargo check --locked             # Check compilation without building
```

## Testing

```bash
cargo test --lib -- --test-threads=1  # Run lib tests serially (required on macOS)
cargo test --lib -- --test-threads=1 test_name  # Run single test by name
cargo bench                          # Run benchmarks
cargo test --doc                     # Doc tests
```

**Note:** Tests must run with `--test-threads=1` to avoid macOS ObjC runtime race conditions when accessing window server APIs.

## Linting and Formatting

```bash
cargo fmt --all                      # Format code
cargo fmt --all --check              # Check formatting
cargo clippy --all-targets --all-features  # Lint
cargo clippy --fix --allow-dirty     # Auto-fix some lints
```

## Code Style

### Import Grouping

Group imports in this order: std → external crates (alphabetical) → local modules.

```rust
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::bail;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::actor::wm_controller::WmCommand;
use crate::sys::hotkey::{Hotkey, HotkeySpec};
```

### Naming Conventions

| Type                 | Convention           | Example                             |
| -------------------- | -------------------- | ----------------------------------- |
| Structs/Enums/Traits | PascalCase           | `ConfigActor`, `LayoutMode`         |
| Functions/methods    | snake_case           | `ensure_accessibility_permission()` |
| Variables            | snake_case           | `window_tx`, `config_path`          |
| Constants            | SCREAMING_SNAKE_CASE | `MAX_WORKSPACES`                    |
| Modules              | snake_case           | `actor`, `layout_engine`            |
| Acronyms             | Mixed case           | `CGSEventType`, `WmCommand`         |

### Error Handling

- Use `anyhow::Result<T>` for fallible functions, `thiserror` for library errors
- Use `bail!("message")` for early returns, `context()` for error context

```rust
pub fn read(path: &Path) -> anyhow::Result<Config> {
    let buf = std::fs::read_to_string(path)?;
    Self::parse(&buf).context("failed to parse config")
}
```

### Concurrency

- Use `tokio::sync::mpsc` for actor message passing
- Use `Sender<T>` wrapper types (see `actor.rs`)
- Use `DashMap` for concurrent hash maps, `Arc<AtomicBool>` for shutdown signals

```rust
pub struct Sender<Event>(UnboundedSender<(Span, Event)>);
impl<Event> Sender<Event> {
    pub fn send(&self, event: Event) { _ = self.try_send(event) }
}
```

## macOS Patterns

- Use `objc2` crates for Objective-C interop (`objc2-app-kit`, `objc2-foundation`)
- Mark main thread usage with `MainThreadMarker`
- Use `NSApplication::finishLaunching()` early in `main()`
- Handle `unsafe` blocks for FFI to SkyLight/CoreGraphics

## Project Structure

```
rift/
├── src/
│   ├── actor/              # Actor implementations
│   ├── bin/rift.rs         # Main daemon entry point
│   ├── common/config.rs    # Config parsing/validation
│   ├── layout_engine/      # Tiling layout algorithms
│   ├── model/              # Data models
│   ├── sys/                # FFI wrappers for macOS APIs
│   └── ui/                 # Menu bar, Mission Control UI
├── assets/Info.plist       # Embedded plist
├── Cargo.toml
├── rustfmt.toml
└── rift.default.toml       # Default configuration
```

## Actor Pattern

```rust
let (broadcast_tx, broadcast_rx) = rift_wm::actor::channel();
let events_tx = Reactor::spawn(config.clone(), layout, broadcast_tx.clone());
let _ = events_tx.send(reactor::Event::Command(reactor::Command::SaveAndExit));
```

Each actor runs an async loop processing messages from its receiver.

## Configuration & State

- **Config**: `~/.config/rift/config.toml` (TOML format)
- **State**: `~/.rift/layout.ron` (RON serialization)
- **Validation**: `Config::validate()` returns `Vec<String>` of issues

## Key Dependencies

- `tokio` - Async runtime
- `anyhow` / `thiserror` - Error handling
- `serde` / `ron` - Serialization
- `objc2-*` - Objective-C interop
- `dashmap` - Concurrent hash map
- `tracing` - Structured logging
