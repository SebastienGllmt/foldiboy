# wasi-gfx runtime patch

`wasi-webgpu-wasmtime`, `wasi-surface-wasmtime`, `wasi-graphics-context-wasmtime`, and `wasi-gfx-wit` are copied from https://github.com/wasi-gfx/wasi-gfx-runtime at revision `067d92d0eabd264e15486b9cecbb9f7020257b7b`.
Upstream attribution and source comments are retained.

The local package replaces workspace dependency inheritance with the same explicit versions and adjusts the WIT path.
The functional change retries missing swapchain textures after timeouts, surface loss, and window resizing.
Surface acquisition checks the host shutdown flag between retries and returns a Wasmtime interrupt when closing.
The graphics-context bindings make acquisition and presentation trappable so `DrawApi` errors propagate to the host without panicking; the guest WIT ABI is unchanged.
Surface destruction waits for GPU work, discards an acquired frame, and releases the swapchain before releasing its window.
The surface package adds a close callback that leaves the native event loop active until the guest finishes teardown, stops and joins the frame ticker, and tolerates events whose receivers have closed.
The host uses Wasmtime epoch interruption plus asynchronous cancellation to stop the guest and joins its worker before returning from `main`.
The worker catches panics after unwinding guest resources and requests final UI exit on success, error, or panic.
The frame-buffer package remains pinned to the original Git revision.
