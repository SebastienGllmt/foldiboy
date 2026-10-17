mod audio;
mod worker;

use anyhow::Context;
use clap::Parser;
use std::sync::Arc;
use wasi_frame_buffer_wasmtime::WasiFrameBufferView;
use wasi_graphics_context_wasmtime::WasiGraphicsContextView;
use wasi_surface_wasmtime::{Surface, SurfaceDesc, WasiSurfaceView};
use wasi_webgpu_wasmtime::WasiWebGpuView;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};

use wasmtime_wasi::{ResourceTable, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_io::IoView;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {}

struct HostState {
    closing: Arc<std::sync::atomic::AtomicBool>,
    pub table: ResourceTable,
    pub ctx: WasiCtx,
    pub instance: Arc<wasi_webgpu_wasmtime::reexports::wgpu_core::global::Global>,
    pub main_thread_proxy: wasi_surface_wasmtime::WasiWinitEventLoopProxy,
}

impl HostState {
    fn new(
        main_thread_proxy: wasi_surface_wasmtime::WasiWinitEventLoopProxy,
        closing: Arc<std::sync::atomic::AtomicBool>,
    ) -> Self {
        let mut wasi = WasiCtxBuilder::new();
        wasi.inherit_stdio();
        if std::env::var("FOLDIBOY_SERIAL_LOG").as_deref() == Ok("1") {
            wasi.env("FOLDIBOY_SERIAL_LOG", "1");
        }
        wasi.preopened_dir(
            "../../roms",
            "roms",
            wasmtime_wasi::DirPerms::READ,
            wasmtime_wasi::FilePerms::READ,
        )
        .expect("failed to preopen roms directory");
        Self {
            closing,
            table: ResourceTable::new(),
            ctx: wasi.build(),
            instance: Arc::new(wasi_webgpu_wasmtime::reexports::wgpu_core::global::Global::new(
                "webgpu",
                &wasi_webgpu_wasmtime::reexports::wgpu_types::InstanceDescriptor {
                    backends: wasi_webgpu_wasmtime::reexports::wgpu_types::Backends::all(),
                    flags: wasi_webgpu_wasmtime::reexports::wgpu_types::InstanceFlags::from_build_config(),
                    backend_options: wasi_webgpu_wasmtime::reexports::wgpu_types::BackendOptions::default(),
                    memory_budget_thresholds: wasi_webgpu_wasmtime::reexports::wgpu_types::MemoryBudgetThresholds::default(),
                },
            )),
            main_thread_proxy,
        }
    }
}

impl IoView for HostState {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
}

impl WasiView for HostState {
    fn ctx(&mut self) -> wasmtime_wasi::WasiCtxView<'_> {
        wasmtime_wasi::WasiCtxView {
            ctx: &mut self.ctx,
            table: &mut self.table,
        }
    }
}

impl WasiGraphicsContextView for HostState {}
impl WasiFrameBufferView for HostState {}

impl audio::AudioView for HostState {}

struct UiThreadSpawner(wasi_surface_wasmtime::WasiWinitEventLoopProxy);

impl wasi_webgpu_wasmtime::MainThreadSpawner for UiThreadSpawner {
    async fn spawn<F, T>(&self, f: F) -> T
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static,
    {
        self.0.spawn(f).await
    }
}

impl WasiWebGpuView for HostState {
    fn shutdown_signal(&self) -> Option<Arc<std::sync::atomic::AtomicBool>> {
        Some(Arc::clone(&self.closing))
    }
    fn instance(&self) -> Arc<wasi_webgpu_wasmtime::reexports::wgpu_core::global::Global> {
        Arc::clone(&self.instance)
    }

    fn ui_thread_spawner(&self) -> Box<impl wasi_webgpu_wasmtime::MainThreadSpawner + 'static> {
        Box::new(UiThreadSpawner(self.main_thread_proxy.clone()))
    }
}

impl WasiSurfaceView for HostState {
    fn create_canvas(&self, desc: SurfaceDesc) -> Surface {
        pollster::block_on(self.main_thread_proxy.create_window(desc))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .init();

    // let args = Args::parse();

    let mut config = Config::default();
    config.wasm_component_model(true);
    config.async_support(true);
    config.epoch_interruption(true);
    let engine = Engine::new(&config)?;
    let mut linker: Linker<HostState> = Linker::new(&engine);

    wasi_webgpu_wasmtime::add_to_linker(&mut linker)?;
    wasi_frame_buffer_wasmtime::add_to_linker(&mut linker)?;
    wasi_graphics_context_wasmtime::add_to_linker(&mut linker)?;
    wasi_surface_wasmtime::add_only_surface_to_linker(&mut linker)?;
    wasmtime_wasi::p2::add_to_linker_sync(&mut linker)?;
    audio::add_to_linker(&mut linker)?;

    let (mut main_thread_loop, main_thread_proxy) =
        wasi_surface_wasmtime::create_wasi_winit_event_loop();
    let exit_proxy = main_thread_proxy.clone();
    let closing = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let host_state = HostState::new(main_thread_proxy, Arc::clone(&closing));

    let mut store = Store::new(&engine, host_state);
    store.set_epoch_deadline(1);

    let component = Component::from_file(&engine, "../../lib/target/foldiboy-lib.wasm")
        .context("Component file not found")?;

    let command =
        wasmtime_wasi::p2::bindings::Command::instantiate_async(&mut store, &component, &linker)
            .await
            .context("failed to instantiate guest")?;

    let (shutdown, stopped) = tokio::sync::oneshot::channel::<()>();
    let worker_closing = Arc::clone(&closing);
    let worker = std::thread::spawn(move || {
        worker::run(
            move || {
                let mut result = pollster::block_on(async {
                    tokio::select! {
                        biased;
                        _ = stopped => Ok(()),
                        result = command.wasi_cli_run().call_run(&mut store) => {
                            result.context("failed to invoke guest")?
                                .map_err(|()| anyhow::anyhow!("guest returned an error"))
                        }
                    }
                });
                // An executing Wasm call may finish with the requested epoch interrupt
                // before select can observe the shutdown channel.
                if worker_closing.load(std::sync::atomic::Ordering::Acquire)
                    && result.as_ref().err().is_some_and(|error| {
                        matches!(
                            error.downcast_ref::<wasmtime::Trap>(),
                            Some(wasmtime::Trap::Interrupt)
                        )
                    })
                {
                    result = Ok(());
                }
                // Guest resources (including audio and GPU objects) are dropped here,
                // while the main thread still owns the native event loop.
                drop(store);
                result
            },
            || exit_proxy.request_exit(),
        )
    });

    let shutdown = std::sync::Mutex::new(Some(shutdown));
    main_thread_loop.run_with_close_handler(Some(Box::new(move || {
        if let Some(shutdown) = shutdown.lock().unwrap().take() {
            closing.store(true, std::sync::atomic::Ordering::Release);
            let _ = shutdown.send(());
            // Keep the UI alive while the guest cancels and releases its GPU
            // and audio resources. The worker then requests final UI exit.
            engine.increment_epoch();
        }
    })));
    let result = worker
        .join()
        .map_err(|_| anyhow::anyhow!("emulator thread panicked"))?;

    result
}
