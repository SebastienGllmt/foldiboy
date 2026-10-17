# Testing

The default suite passes 350 tests, with 6 ignored.
It includes unit tests and the Blargg/SameSuite ROM targets below; it does not run every external test ROM repository.

## Setup and commands

Clone [retrio/gb-test-roms](https://github.com/retrio/gb-test-roms) and [LIJI32/SameSuite](https://github.com/LIJI32/SameSuite) in the workspace’s `roms/tests/` directory, or set `FOLDIBOY_TEST_ROOT` to their parent directory.
`gb-test-roms` contains compiled ROMs; SameSuite must be built with RGBDS.
The helpers use the logical workspace path, including when its repositories are symlinks:

```sh
./scripts/setup-tests.sh
./scripts/test.sh
```

The setup helper uses RGBDS from PATH, falling back to a pinned project-scoped Nix package.
Cargo restores locked WIT dependencies through the upstream `wit-deps` build dependency.
The project-owned audio interface is tracked in `lib/interfaces/nightstream-audio/`.
The external test ROM checkout revisions are not currently pinned by Cargo's lockfile.

For direct Cargo invocation:

```sh
cd lib
FOLDIBOY_TEST_ROOT=/path/to/test-suites cargo test --locked --workspace --no-fail-fast
```

Missing suite ROMs are failures with their attempted paths in the diagnostic.
Without the environment override, the CPU harness looks in `../roms/tests/` relative to a normal, non-symlinked Foldiboy checkout.

## Default coverage

| Target | Passing | Ignored | Scope |
| --- | ---: | ---: | --- |
| APU unit tests | 75 | 0 | Channels, frame sequencer, mixer, registers |
| CPU unit tests | 101 | 0 | Opcodes, flags, interrupts, HALT, EI delay |
| Blargg CPU integration | 14 | 0 | 11 individual CPU ROMs, instruction timing, HALT bug, and a HALT diagnostic |
| Blargg DMG sound | 9 | 3 | 01–08 and 11 enabled; 09, 10, and 12 ignored for wave RAM timing |
| APU diagnostics | 3 | 0 | Two local diagnostics and one using sound ROM 08 |
| SameSuite | 1 | 2 | `ei_delay_halt` enabled; two channel 3 wave RAM tests ignored |
| Optional game profiler | 0 | 1 | Requires an explicit ROM path |
| Memory unit tests | 3 | 0 | Bus trait and flat test memory |
| PPU unit tests | 65 | 0 | Mode transitions, STAT IRQ, scanlines, OAM |
| Render unit tests | 79 | 0 | Debug UI, text, ROM browser, memory and trace viewers, viewport scaling and hit testing |
| Total | 350 | 6 | |

The CPU ROM harness uses flat memory, a basic timer/VBlank model, and the APU.
It does not exercise the production memory bus and PPU together.
Passing these targets therefore does not establish full-system DMA, mapper, or PPU timing accuracy.
The HALT and sound diagnostics are not all independent conformance assertions.

Game profiling is optional and is not a conformance test:

```sh
FOLDIBOY_PROFILE_ROM=/absolute/path/to/game.gb ./scripts/test.sh -p cpu --test opcode_profile -- --ignored
```

Native graphics/shutdown regression tests are separate from the guest workspace:

```sh
cd host/desktop
cargo test --locked --release -p foldiboy-desktop -p wasi-graphics-context-wasmtime -p wasi-surface-wasmtime -p wasi-webgpu-wasmtime
```

There are ten native regression tests; the real audio-device check is ignored by default.
See [the desktop documentation](../host/desktop/README.md) for real-device audio checks and launch instructions.

## External suites and tools

| Project | Purpose | Integration status |
| --- | --- | --- |
| [gb-test-roms](https://github.com/retrio/gb-test-roms) | Blargg CPU, sound, memory timing, and OAM tests | CPU, instruction timing, HALT, and DMG sound subsets have runners |
| [SameSuite](https://github.com/LIJI32/SameSuite) | Interrupt and APU behavior, including CGB tests | One enabled interrupt test and two ignored wave RAM tests |
| [mooneye-test-suite](https://github.com/Gekkio/mooneye-test-suite) | CPU, interrupts, timer, PPU, DMA, and mapper acceptance | Historical results below; no surviving ROM runner |
| [dmg-acid2](https://github.com/mattcurrie/dmg-acid2) | DMG rendering correctness against a reference image | Referenced by a sprite-priority unit test; no framebuffer comparison runner |
| [mealybug-tearoom-tests](https://github.com/mattcurrie/mealybug-tearoom-tests) | Precise PPU register-write and rendering timing | No Foldiboy runner |
| [mbc30test](https://github.com/ZoomTen/mbc30test) | MBC3/MBC30 ROM and SRAM banking | No Foldiboy runner |
| [rtc3test](https://github.com/aaaaaa123456789/rtc3test) | MBC3 real-time clock | No Foldiboy runner; RTC is currently stubbed |
| [gameboy-doctor](https://github.com/robert/gameboy-doctor) | Compare CPU traces against Blargg reference logs | No compatible trace exporter or invocation integrated |

Mooneye currently builds with WLA-DX, acid2 and Mealybug with RGBDS, and MBC30 with ASMotor; some also publish prebuilt ROMs.
Acid2 can pass with a line-based renderer, while Mealybug tests mode-3 timing behavior that conflicts with Foldiboy's documented simplifications.
Image-based tests require model-specific reference images and DMG grayscale values `0x00`, `0x55`, `0xAA`, and `0xFF`.

## Expected accuracy limitations

The following expectations come from the documented architecture and historical results, not a current run of the missing external runners.

**CPU / Bus:**
- All `boot_*` tests — no boot ROM, execution starts at PC=$0100
- 14 sub-instruction timing tests — atomic instruction execution
- 3 OAM DMA timing tests — instantaneous DMA transfer
- `halt_ime0_nointr_timing` — requires PPU-dependent halt wake timing

**PPU:**
- 7 Mooneye dot-precise timing tests — fixed mode 3 duration (no SCX/sprite penalties)

**Timer:**
- `rapid_toggle` — sub-instruction counter offset (8T boundary shift)
- `tima_write_reloading` — writing TIMA during 4-cycle reload delay
- `tma_write_reloading` — writing TMA during 4-cycle reload delay

**APU:**
- Blargg dmg_sound 09, 10, 12 — simplified wave RAM access timing

**Historical Mooneye passes (runner not present and results not rerun):** `ie_push`, `oam_dma/reg_read`, `oam_dma/sources-GS`, `stat_irq_blocking`, `stat_lyc_onoff`, `vblank_stat_intr-GS`, `timer/div_write`, `timer/tim00`, `timer/tim01`, `timer/tim10`, `timer/tim11`, `timer/tima_reload`, `ei_sequence`
