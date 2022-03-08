# 🦔 vxl-parser

[![vxl-parser](https://github.com/vektor-finance/vxl-parser/actions/workflows/build.yml/badge.svg)](https://github.com/vektor-finance/vxl-parser/actions/workflows/build.yml)
[![vxl-parser: security audit (nightly)](https://github.com/vektor-finance/vxl-parser/actions/workflows/security-audit-nightly.yml/badge.svg)](https://github.com/vektor-finance/vxl-parser/actions/workflows/security-audit-nightly.yml)

[![Minimum rustc version](https://img.shields.io/badge/rustc-1.49.0+-lightgray.svg)](#rust-version-requirements)

## Overview

- **Purpose**: VXL parser library
- **Technologies**: [Rust](https://www.rust-lang.org/) with [nom](https://github.com/Geal/nom)
- **Deployment**: Compiled at build-time by other projects
- **Where**: [app](https://github.com/vektor-finance/app) and [vektor backend](https://github.com/vektor-finance/vektor)

## Background

This library start off as a fork of [https://github.com/jlindsey/hcl2](https://github.com/jlindsey/hcl2).

Vektor Execution Language (VXL) is a chain-agnostic highly flexible grammar to interact with smart contracts and blockchains.

The `vxl-parser` project is responsible for:

- Converting a string input into objects e.g. `Function`, `Argument`, `Optional Argument`.
- Validating the input is syntactically correct.
- Having some light `domain` knowledge such as addresses start with `0x`.

It is **not** responsible for aspects such as:

- Knowing about assets in the crypto ecosystem.
- Determing if a command can work across 2 blockchains.
- Knowing if a user has enough funds to execute a command.

## Requirements

| Name                                               | Purpose        | Install                                                           | Version                         |
| :------------------------------------------------- | :------------- | :---------------------------------------------------------------- | :------------------------------ |
| [Rust](https://www.rust-lang.org/tools/install)    | Rust toolchain | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` | rustc `1.49.0`, rustup `1.23.1` |
| [wasm-pack](https://github.com/rustwasm/wasm-pack) | WASM builder   | `cargo install wasm-pack`                                         | `0.10.0`                        |

## Getting Started

1. Build the workspace with `cargo build`

## Repo Structure

- `/core` - parsing library
- `/cli` - CLI app e.g. `cat test.vxl | ./vxl`
- `/js` - JavaScript wrapper library
- `/elixir` - Elixir NIF

## Building crates

### CLI app

```bash
cargo build
```

### JavaScript

```bash
wasm-pack build --dev --scope vektor js
```

The [app](https://github.com/vektor-finance/app) project takes care of preparing the package for embedding in the app.

### Elixir

The compilation and loading is handled by `mix` at build or runtime.

## Testing

### Rust Libraries and Binaries

```bash
cargo test
```

### JavaScript Binding

#### Browser testing

Omit `--headless` to test interactively

```bash
wasm-pack test --chrome --safari --firefox --headless js
```

### Node testing

```bash
wasm-pack test --node js
```

### Benchmarking

```bash
cargo bench
```

## TODO

See [TODO](TODO.md)

## Known issues

### vxl-parser WASM module packing process

Currently, we compile and package the `vxl-parser` using `wasm-pack` which works but has some brittle steps that could be improved.

We rename the package in the outputted `package.json` during a publish step (for convenience sake but necessarily required).

### Optimisations

- [ ] Use [wasm-snip](https://rustwasm.github.io/book/reference/code-size.html#use-the-wasm-snip-tool)
- [ ] Time profile
- [ ] Compare [wee_alloc](https://github.com/rustwasm/wee_alloc)
- [ ] See about switching logging to [console_log](https://github.com/iamcodemaker/console_log)
- [ ] Test [compilation sizes](https://rustwasm.github.io/book/reference/code-size.html#optimizing-builds-for-code-size)

## Learn More

- [Learn Rust](https://www.rust-lang.org/learn)
- [nom](https://github.com/Geal/nom)
- [Parsing in Rust with nom](https://blog.logrocket.com/parsing-in-rust-with-nom/)
