# CHIP-8 Emulator
Simple CHIP-8 emulator written in Rust. Includes a bunch of public domain ROMs from [Zophar's Domain](https://www.zophar.net/pdroms/chip8.html).

![Screenshot of the MAZE game running on the emulator on Windows](screenshot.png)

## Getting started

### Windows
Download the .exe file from the relase page on Github and drag a ROM file on top of the executable to play it.

### Controls
The original controls:

|   |   |   |   |
|---|---|---|---|
| 1 | 2 | 3 | C |
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

Are translated as:

|   |   |   |   |
|---|---|---|---|
| 1 | 2 | 3 | 4 |
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |

### Building the project

Make sure you have [Rust installed](https://www.rust-lang.org/tools/install). You should be able to run `cargo` from the command line. Then run:

```
cargo install cargo-vcpkg
cargo vcpkg build
cargo run -r path/to/rom
```
