# ultrabear/bfirs

A Rust port of [ultrabear/bfi](https://github.com/ultrabear/bfi).

This implementation is faster than bfi, and writing it served as a tool to better learn Rust as a language.
It uses the same algorithms from the Go version with some tweaking to work in the context of Rust.

# Structure

The project is structured as two crates: a library and a binary. The library provides the actual Brainfuck compilation and interpretation, while the binary provides a CLI interface.

# MSRV

The MSRV (Minimum Supported Rust Version) of this project is currently 1.61, but this is subject to increase so using "latest" as an MSRV is more appropriate.

# `release-lto` profile

A Cargo profile for building in release mode with LTO is provided. To use this profile pass `--profile release-lto` to Cargo commands.

# Differences from `bfi`

- Removed automatic compression. `+[]` will never halt in `bfirs`.
- Added support for 16- and 32-bit execution modes.
- (For the binary) Changed the CLI to require flag arguments, unlike `bfi` which takes argv as code by default.
