# Desktop frontend

From this directory, build the Wasm guest and launch the native host:

```sh
./build.sh
```

Release mode is required for practical Cranelift performance.
The launcher expects to run from `host/desktop` so its component and ROM paths resolve correctly.

The pinned wasi-gfx runtime's WebGPU package is patched locally to recover when acquisition returns no texture after a surface resize, loss, or timeout.
See [vendor/README.md](vendor/README.md) for the source revision and patch scope.
The Linux audio backend tries PipeWire/PulseAudio devices if ALSA's default output cannot open.
This fallback does not modify the machine's ALSA configuration.

Run the native graphics and shutdown regression tests from this directory:

```sh
cargo test --locked --release -p foldiboy-desktop -p wasi-graphics-context-wasmtime -p wasi-surface-wasmtime -p wasi-webgpu-wasmtime
```

To check real audio-device access without playing a test tone:

```sh
cargo test --locked --release opens_desktop_audio_output -- --ignored --nocapture
```

The optional Vulkan `VK_EXT_physical_device_drm` warning does not prevent startup.

Serial/link-cable bytes are not printed by default.
For Blargg ROMs or serial debugging, enable them explicitly:

```sh
FOLDIBOY_SERIAL_LOG=1 ./build.sh
```

The host forwards only `FOLDIBOY_SERIAL_LOG=1` and `FOLDIBOY_INTEGER_SCALE=1` into the Wasm guest.
Ordinary game traffic can contain repeated `0x55` bytes, which appeared as `U` with the previous unconditional serial logging.

Window close now cancels the guest, releases audio/GPU resources while the event loop is alive, and joins the guest and frame threads before exiting normally.
The acquisition retry loop observes shutdown, and a worker panic requests final UI exit after guest resources unwind.
The regression suite covers surface recovery, cancellation during persistent timeout/loss, graphics-context error propagation, worker success/error/panic cleanup, and late events after receiver shutdown.

For integer scaling inside a tiled window, launch with:

```sh
FOLDIBOY_INTEGER_SCALE=1 ./build.sh
```

The renderer centers the canvas at the largest whole-number scale that fits the window and leaves unused space black.
Windows smaller than the logical 480×240 canvas shrink to fit without cropping.
Pointer hit testing follows the same scaled layout.
The default remains fractional scaling to fit the available space.
