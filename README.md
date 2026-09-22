# DSRV Language Server

`dsrv-lsp` is the language server for the DSRV langauge. It implements the Language Server Protocol using the [`tower-lsp-server`](https://crates.io/crates/tower-lsp-server) crate, and is the component behind the language features of the [DSRV VS Code extension](https://github.com/INTO-CPS-Association/dsrv-vscode).

For how to install and configure the extension, see [Editor Support](https://into-cps-association.github.io/robosapiens-trustworthiness-checker/features/editor-support.html) in the Trustworthiness Checker documentation.

## What it provides

| Feature | Notes |
|---|---|
| Completion | Keywords and built-in functions; not yet context-aware |
| Diagnostics | Syntax and type errors, reported as the document is edited |
| Hover | Information about the symbol under the cursor |

## Relationship to the Trustworthiness Checker

The server depends on the [Trustworthiness Checker](https://github.com/INTO-CPS-Association/robosapiens-trustworthiness-checker) as a library and reuses its parser and type checker, so the diagnostics shown in an editor come from the same language definition the checker evaluates.

That dependency is pinned to a specific revision in `Cargo.toml`, and the pin is moved deliberately rather than automatically, so that each update can be tested against the server before it is released. Two consequences follow:

- the server does not need rebuilding for every checker change; only for changes to the language definition or the type checker;
- between updates, the checker may accept a program the server flags, or the reverse. When diagnostics and a run disagree, the pin is the first thing to check.

## Build

Requires Rust 1.95 or newer, inherited from the checker's minimum supported version.

```sh
# development
cargo build

# release
cargo build --release
```

The executable is written to `target/debug/dsrv-lsp` or `target/release/dsrv-lsp`. Point the extension's `DSRV.lspPath` setting at it, or place it on `PATH`.

The crate also builds a library (`dsrv_lsp`), so the analysis layer can be used outside the server.

### A self-contained library

For distribution, build against musl to produce an executable with no runtime dependency on system libraries:

```sh
rustup target add x86_64-unknown-linux-musl
cargo build --release --target x86_64-unknown-linux-musl
```

`ldd target/x86_64-unknown-linux-musl/release/dsrv-lsp` reports `statically linked` for such a build. No C toolchain is requried.

## Tests

```sh
cargo test
```

## License

This project is licensed under GPL-3.0-only. See the `Cargo.toml` package metadata for details.