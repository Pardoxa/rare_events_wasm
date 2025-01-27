# Git repository of WebAssembly-Application

This Application is for a book by Yannick Feld and Alexander K. Hartmann, which is currently Work in progress.
The current working title is "Simulation of rare events - From the foundations to efficient algorithms".

## Compiling

The following instructions have been tested under Linux

Install Rust via [rustup](https://rustup.rs/). You can find more information [here](https://doc.rust-lang.org/beta/book/ch01-01-installation.html).
Next install the webassembly target:

```bash
rustup target add wasm32-unknown-unknown
```

Then you can install trunk via 

```bash
cargo install trunk
cargo install wasm-bindgen-cli
```
Note: Tested with trunk v0.21.5

Afterwards for local testing use
```bash
export RUSTFLAGS=--cfg=web_sys_unstable_apis
trunk serve --release
```
Note: The export is required for copy and paste to work on the WebApp

To build the release version use
```bash
export RUSTFLAGS=--cfg=web_sys_unstable_apis
trunk build --release
```
Note: The export is required for copy and paste to work on the WebApp

The files will appear in the `dist` folder