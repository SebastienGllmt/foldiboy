Place any ROM in this directory: the emulator's ROM browser lists it (`lib/crates/render/src/rom_browser.rs`). Nothing here is tracked but this file.

A game built from source keeps its source outside the repository, in the workspace's `roms/games/<game>`, and its setup script builds it there and copies the ROM here (`roms/games/setup-tetris.sh` writes `tetris.gb`).
The test suites are elsewhere: their checkouts are under `FOLDIBOY_TEST_ROOT`, and the harnesses that run them on the semantics are in `krs/oracles/`.
