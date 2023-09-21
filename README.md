# CHIP-8 Emulator
Simple CHIP-8 emulator written in Rust

## Getting started

### Windows
Download the .exe file from the relase page on Github and drag a ROM file on top of the executable to play it.

### Building the project

Make sure you have [Rust installed](https://www.rust-lang.org/tools/install). You should be able to run `cargo` from the command line. Then run:

```
cargo install cargo-vcpkg
cargo vcpkg build
cargo run -r path/to/rom
```
