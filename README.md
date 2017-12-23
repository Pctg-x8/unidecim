Unidecim
---

Cheap Decimator plugin for Unity

## Supported Environment

- Unity: 2017.2.1f and later
- OS: Windows/Mac

## Building

### Windows

> rustup run nightly cargo build [--release]

copy `target/(debug|release)/AudioPlugin_Unidecim.dll` into Plugin folder

### Mac

> ./build-darwin-universal.sh

copy `target/release/AudioPlugin_Unidecim.bundle` into Plugin folder
