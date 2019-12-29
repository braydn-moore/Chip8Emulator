## Chip8 Emulator
An emulator for the chip8 chip, written to learn Rust. It follows the [specifications for Chip8.](http://www.cs.columbia.edu/~sedwards/classes/2016/4840-spring/designs/Chip8.pdf) The emulator uses SDL2 for graphics and user input.

## Getting Started
The application can be built by running

    cargo build --release

in the root directory. Precompiled binaries for **x86_64-unknown-linux-gnu** can be found in the "binaries" folder and are run as follows:

    ./binary <path to rom file>
