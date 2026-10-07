# GoldSrc.rs Plugin SDK (`goldsrc-sdk`)

[![CI](https://github.com/goldsrc-rs/goldsrc-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/goldsrc-rs/goldsrc-sdk/actions)
[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/goldsrc.svg)](https://crates.io/crates/goldsrc)

The official, ultra-fast, pure Rust guest SDK and framework for developing WebAssembly (`wasm32-wasip1`) plugins for **GoldSrc.rs** (Half-Life 1, Counter-Strike 1.6).

## Highlights

- **Pure Guest SDK**: Zero dependency on Wasmtime, C++ compilers, SQLite, or engine host libraries.
- **Declarative Macros**: Ergonomic procedural macros (`#[plugin]`, `#[derive(PluginConfig)]`, `#[command]`).
- **Zero-Allocation Logging & Formatting**: Transparent guest logging bridging directly into host targets.
- **Rich Menus, HUD, Chat & ECS**: Out-of-the-box building blocks for interactive gameplay plugins.
- **Localization (i18n)**: Compile-time namespace translation lookups and parameter substitutions.

## Workspace Crates

- [`goldsrc`](goldsrc): High-level developer-facing framework, event handlers, and abstractions.
- [`goldsrc-api`](goldsrc-api): Abstract domain interfaces and WebAssembly component WIT bindings.
- [`goldsrc-spi`](goldsrc-spi): Service Provider Interface contracts (zero heavy dependencies).
- [`goldsrc-macros`](goldsrc-macros): Procedural macros for plugins, configs, commands, and events.
- [`examples/`](examples): Reference demo plugins (`test_chat`, `test_hud`, `test_i18n`, `test_menu`, `test_ecs`).

## Quick Start

Add `goldsrc` to your plugin's `Cargo.toml`:

```toml
[package]
name = "my_plugin"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
goldsrc = "0.20"
```

Write your plugin entry point in `src/lib.rs`:

```rust
use goldsrc::prelude::*;

#[plugin]
pub struct MyPlugin;

impl Plugin for MyPlugin {
    fn on_load(&mut self) {
        log_info!("MyPlugin loaded successfully!");
    }
}
```

Compile for WebAssembly:

```bash
cargo build --target wasm32-wasip1 --release
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for testing guidelines and development workflows.

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or [MIT License](LICENSE-MIT) at your option.
