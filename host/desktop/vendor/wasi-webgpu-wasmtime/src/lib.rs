// TODO: in this file:
// - Remove all calls to `Default::default()`. Instead, manually set them, and link to the spec stating what defaults should be used.
// - Implement all todos.
// - Remove all unwraps.
// - Implement all the drop handlers.
// - Remove clippy allows, and either fix the code, or add comments explaining why it's okay to leave it.

#![allow(clippy::unwrap_or_default)]
#![allow(clippy::new_without_default)]

use std::{future::Future, sync::Arc};

use wasi_graphics_context_wasmtime::{AbstractBuffer, DisplayApi, DrawApi};
use wasmtime::component::HasData;
use wasmtime_wasi_io::IoView;
use wgpu_core::id::SurfaceId;

// ToCore trait used for resources, records, and variants.
// Into trait used for enums, since they never need table access.
mod enum_conversions;
mod to_core_conversions;

mod trait_impls;
mod wrapper_types;

/// Re-export of `wgpu_core` and `wgpu_types` so that runtime implementors don't need to keep track of what version of wgpu this crate is using.
pub mod reexports {
    pub use wgpu_core;
    pub use wgpu_types;
}

#[cfg(all(
    not(target_os = "linux"),
    not(target_os = "android"),
    not(target_os = "windows"),
    not(target_os = "macos"),
    not(target_os = "ios"),
))]
pub(crate) type Backend = wgpu_core::api::Gl;

// needed for wasmtime::component::bindgen! as it only looks in the current crate.

wasmtime::component::bindgen!({
    path: "../wasi-gfx-wit/",
    world: "example",
    imports: {
        "wasi:webgpu/webgpu.[method]gpu-buffer.map-async": async
    },
    with: {
        "wasi:io": wasmtime_wasi_io::bindings::wasi::io,
        "wasi:webgpu/webgpu.gpu-adapter": wgpu_core::id::AdapterId,
        "wasi:webgpu/webgpu.gpu-device": wrapper_types::Device,
        "wasi:webgpu/webgpu.gpu-queue": wgpu_core::id::QueueId,
        "wasi:webgpu/webgpu.gpu-command-encoder": wgpu_core::id::CommandEncoderId,
        "wasi:webgpu/webgpu.gpu-render-pass-encoder": wrapper_types::RenderPassEncoder,
        "wasi:webgpu/webgpu.gpu-compute-pass-encoder": wrapper_types::ComputePassEncoder,
        "wasi:webgpu/webgpu.gpu-shader-module": wgpu_core::id::ShaderModuleId,
        "wasi:webgpu/webgpu.gpu-render-pipeline": wgpu_core::id::RenderPipelineId,
        "wasi:webgpu/webgpu.gpu-render-bundle-encoder": wrapper_types::RenderBundleEncoder,
        "wasi:webgpu/webgpu.gpu-render-bundle": wgpu_core::id::RenderBundleId,
        "wasi:webgpu/webgpu.gpu-command-buffer": wgpu_core::id::CommandBufferId,
        "wasi:webgpu/webgpu.gpu-buffer": wrapper_types::Buffer,
        // "wasi:webgpu/webgpu.non-standard-buffer": wrapper_types::BufferPtr,
        "wasi:webgpu/webgpu.gpu-pipeline-layout": wgpu_core::id::PipelineLayoutId,
        "wasi:webgpu/webgpu.gpu-bind-group-layout": wgpu_core::id::BindGroupLayoutId,
        "wasi:webgpu/webgpu.gpu-sampler": wgpu_core::id::SamplerId,
        "wasi:webgpu/webgpu.gpu-supported-features": wgpu_types::Features,
        "wasi:webgpu/webgpu.gpu-texture": wgpu_core::id::TextureId,
        "wasi:webgpu/webgpu.gpu-compute-pipeline": wgpu_core::id::ComputePipelineId,
        "wasi:webgpu/webgpu.gpu-bind-group": wgpu_core::id::BindGroupId,
        "wasi:webgpu/webgpu.gpu-texture-view": wgpu_core::id::TextureViewId,
        "wasi:webgpu/webgpu.gpu-adapter-info": wgpu_types::AdapterInfo,
        "wasi:webgpu/webgpu.gpu-query-set": wgpu_core::id::QuerySetId,
        "wasi:webgpu/webgpu.gpu-supported-limits": wgpu_types::Limits,
        "wasi:webgpu/webgpu.record-gpu-pipeline-constant-value": wrapper_types::RecordGpuPipelineConstantValue,
        "wasi:webgpu/webgpu.record-option-gpu-size64": wrapper_types::RecordOptionGpuSize64,
        "wasi:graphics-context/graphics-context": wasi_graphics_context_wasmtime::wasi::graphics_context::graphics_context,
    },
});

struct WasiWebGpu<T: Send>(T);
impl<T: Send + 'static> HasData for WasiWebGpu<T> {
    type Data<'a> = WasiWebGpuImpl<&'a mut T>;
}
pub fn add_to_linker<T>(l: &mut wasmtime::component::Linker<T>) -> wasmtime::Result<()>
where
    T: WasiWebGpuView,
{
    wasi::webgpu::webgpu::add_to_linker::<_, WasiWebGpu<T>>(l, |x| WasiWebGpuImpl(x))?;
    Ok(())
}

pub trait WasiWebGpuView: IoView + Send {
    fn instance(&self) -> Arc<wgpu_core::global::Global>;

    /// Optional host shutdown flag, checked between surface acquisition attempts.
    fn shutdown_signal(&self) -> Option<Arc<std::sync::atomic::AtomicBool>> {
        None
    }

    /// Provide the ability to run closure on the UI thread.
    /// On platforms that don't require UI to run on the UI thread, this can just execute in place.
    fn ui_thread_spawner(&self) -> Box<impl MainThreadSpawner>;
}

#[repr(transparent)]
pub struct WasiWebGpuImpl<T>(pub T);
impl<T: WasiWebGpuView> wasmtime_wasi_io::IoView for WasiWebGpuImpl<T> {
    fn table(&mut self) -> &mut wasmtime_wasi::ResourceTable {
        T::table(&mut self.0)
    }
}

impl<T: ?Sized + WasiWebGpuView> WasiWebGpuView for &mut T {
    fn instance(&self) -> Arc<wgpu_core::global::Global> {
        T::instance(self)
    }

    fn shutdown_signal(&self) -> Option<Arc<std::sync::atomic::AtomicBool>> {
        T::shutdown_signal(self)
    }

    fn ui_thread_spawner(&self) -> Box<impl MainThreadSpawner> {
        T::ui_thread_spawner(self)
    }
}
impl<T: ?Sized + WasiWebGpuView> WasiWebGpuView for Box<T> {
    fn instance(&self) -> Arc<wgpu_core::global::Global> {
        T::instance(self)
    }

    fn shutdown_signal(&self) -> Option<Arc<std::sync::atomic::AtomicBool>> {
        T::shutdown_signal(self)
    }

    fn ui_thread_spawner(&self) -> Box<impl MainThreadSpawner> {
        T::ui_thread_spawner(self)
    }
}
impl<T: WasiWebGpuView> WasiWebGpuView for WasiWebGpuImpl<T> {
    fn instance(&self) -> Arc<wgpu_core::global::Global> {
        T::instance(&self.0)
    }

    fn shutdown_signal(&self) -> Option<Arc<std::sync::atomic::AtomicBool>> {
        T::shutdown_signal(&self.0)
    }

    fn ui_thread_spawner(&self) -> Box<impl MainThreadSpawner> {
        T::ui_thread_spawner(&self.0)
    }
}

pub trait MainThreadSpawner: Send + Sync + 'static {
    fn spawn<F, T>(&self, f: F) -> impl Future<Output = T>
    where
        F: FnOnce() -> T + Send + 'static,
        T: Send + 'static;
}

struct WebGpuSurface<GI, CS, I>
where
    I: AsRef<wgpu_core::global::Global>,
    GI: Fn() -> I,
    CS: Fn(&Arc<dyn DisplayApi + Send + Sync>) -> SurfaceId,
{
    get_instance: GI,
    create_surface: CS,
    device_id: wgpu_core::id::DeviceId,
    // Might be needed one day for surface configuration.
    _adapter_id: wgpu_core::id::AdapterId,
    surface_id: Option<wgpu_core::id::SurfaceId>,
    display_api: Option<Arc<dyn DisplayApi + Send + Sync>>,
    needs_reconfigure: bool,
    shutdown: Option<Arc<std::sync::atomic::AtomicBool>>,
}

impl<GI, CS, I> Drop for WebGpuSurface<GI, CS, I>
where
    I: AsRef<wgpu_core::global::Global>,
    GI: Fn() -> I,
    CS: Fn(&Arc<dyn DisplayApi + Send + Sync>) -> SurfaceId,
{
    fn drop(&mut self) {
        if let Some(surface) = self.surface_id.take() {
            let instance = (self.get_instance)();
            // Cancellation can occur between acquisition and presentation.
            // Finish GPU work and release that frame before unconfiguring the
            // swapchain, while display_api still keeps the window alive.
            let _ = instance.as_ref().poll_all_devices(true);
            let _ = instance.as_ref().surface_texture_discard(surface);
            instance.as_ref().surface_drop(surface);
        }
    }
}

/// A recoverable acquisition can legitimately contain no texture.
fn acquire_surface_texture<T>(
    mut acquire: impl FnMut() -> wasmtime::Result<wgpu_core::present::SurfaceOutput<T>>,
    mut reconfigure: impl FnMut() -> wasmtime::Result<()>,
    mut wait: impl FnMut(),
    mut cancelled: impl FnMut() -> bool,
) -> wasmtime::Result<(T, bool)> {
    use wgpu_types::SurfaceStatus;
    loop {
        if cancelled() {
            return Err(wasmtime::Trap::Interrupt.into());
        }
        let output = acquire()?;
        if let Some(texture) = output.texture {
            return Ok((texture, matches!(output.status, SurfaceStatus::Suboptimal)));
        }
        match output.status {
            SurfaceStatus::Timeout => {}
            SurfaceStatus::Outdated | SurfaceStatus::Lost => reconfigure()?,
            status => {
                return Err(wasmtime::Error::msg(format!(
                    "surface returned no texture: {status:?}"
                )))
            }
        }
        // The UI thread remains free to process resize/minimize events.
        wait();
    }
}

impl<GI, CS, I> WebGpuSurface<GI, CS, I>
where
    I: AsRef<wgpu_core::global::Global>,
    GI: Fn() -> I,
    CS: Fn(&Arc<dyn DisplayApi + Send + Sync>) -> SurfaceId,
{
    fn configure_surface(&self) -> wasmtime::Result<()> {
        let display = self
            .display_api
            .as_ref()
            .ok_or_else(|| wasmtime::Error::msg("graphics context has no display"))?;
        let surface_id = self
            .surface_id
            .ok_or_else(|| wasmtime::Error::msg("graphics context has no surface"))?;
        #[cfg(target_os = "android")]
        let format = wgpu_types::TextureFormat::Rgba8Unorm;
        #[cfg(not(target_os = "android"))]
        let format = wgpu_types::TextureFormat::Bgra8Unorm;
        let config = wgpu_types::SurfaceConfiguration {
            format,
            usage: wgpu_types::TextureUsages::RENDER_ATTACHMENT,
            view_formats: vec![],
            alpha_mode: wgpu_types::CompositeAlphaMode::Opaque,
            width: display.width().max(1),
            height: display.height().max(1),
            present_mode: wgpu_types::PresentMode::Fifo,
            desired_maximum_frame_latency: 2,
        };
        if let Some(error) =
            (self.get_instance)()
                .as_ref()
                .surface_configure(surface_id, self.device_id, &config)
        {
            return Err(error.into());
        }
        Ok(())
    }
}

impl<GI, CS, I> DrawApi for WebGpuSurface<GI, CS, I>
where
    I: AsRef<wgpu_core::global::Global>,
    GI: Fn() -> I,
    CS: Fn(&Arc<dyn DisplayApi + Send + Sync>) -> SurfaceId,
{
    fn get_current_buffer(&mut self) -> wasmtime::Result<AbstractBuffer> {
        if self.needs_reconfigure {
            self.configure_surface()?;
            self.needs_reconfigure = false;
        }
        let surface_id = self
            .surface_id
            .ok_or_else(|| wasmtime::Error::msg("graphics context has no surface"))?;
        let instance = (self.get_instance)();
        let (texture, suboptimal) = acquire_surface_texture(
            || {
                Ok(instance
                    .as_ref()
                    .surface_get_current_texture(surface_id, None)?)
            },
            || self.configure_surface(),
            || std::thread::sleep(std::time::Duration::from_millis(16)),
            || {
                self.shutdown
                    .as_ref()
                    .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::Acquire))
            },
        )?;
        self.needs_reconfigure = suboptimal;
        Ok(Box::new(texture).into())
    }

    fn present(&mut self) -> wasmtime::Result<()> {
        let surface_id = self
            .surface_id
            .ok_or_else(|| wasmtime::Error::msg("graphics context has no surface"))?;
        let status = (self.get_instance)().as_ref().surface_present(surface_id)?;
        self.needs_reconfigure |= matches!(
            status,
            wgpu_types::SurfaceStatus::Outdated
                | wgpu_types::SurfaceStatus::Lost
                | wgpu_types::SurfaceStatus::Suboptimal
        );
        Ok(())
    }

    fn display_api_ready(&mut self, display: &Arc<dyn DisplayApi + Send + Sync>) {
        self.surface_id = Some((self.create_surface)(display));
        self.display_api = Some(Arc::clone(display));
        // Configure on acquisition so errors can be returned to the caller.
        self.needs_reconfigure = true;
    }
}

#[cfg(test)]
mod surface_tests {
    use super::acquire_surface_texture;
    use wgpu_core::present::SurfaceOutput;
    use wgpu_types::SurfaceStatus;

    #[test]
    fn recovers_from_resize_loss_and_timeout() {
        let mut outputs = [
            SurfaceOutput {
                status: SurfaceStatus::Outdated,
                texture: None,
            },
            SurfaceOutput {
                status: SurfaceStatus::Timeout,
                texture: None,
            },
            SurfaceOutput {
                status: SurfaceStatus::Lost,
                texture: None,
            },
            SurfaceOutput {
                status: SurfaceStatus::Good,
                texture: Some(42),
            },
        ]
        .into_iter();
        let mut configurations = 0;
        let mut waits = 0;
        let result = acquire_surface_texture(
            || Ok(outputs.next().expect("unexpected extra acquisition")),
            || {
                configurations += 1;
                Ok(())
            },
            || waits += 1,
            || false,
        )
        .unwrap();
        assert_eq!(result, (42, false));
        assert_eq!(configurations, 2);
        assert_eq!(waits, 3);
    }

    #[test]
    fn keeps_suboptimal_frame_and_requests_later_reconfiguration() {
        let result = acquire_surface_texture(
            || {
                Ok(SurfaceOutput {
                    status: SurfaceStatus::Suboptimal,
                    texture: Some(42),
                })
            },
            || panic!("must present the acquired texture before reconfiguring"),
            || panic!("valid frame must not wait"),
            || false,
        )
        .unwrap();
        assert_eq!(result, (42, true));
    }

    #[test]
    fn propagates_acquisition_and_reconfiguration_errors() {
        let result = acquire_surface_texture::<u8>(
            || Err(wasmtime::Error::msg("device lost")),
            || panic!("device error is not a surface resize"),
            || panic!("device error must not loop"),
            || false,
        );
        assert!(result.unwrap_err().to_string().contains("device lost"));
        let result = acquire_surface_texture::<u8>(
            || {
                Ok(SurfaceOutput {
                    status: SurfaceStatus::Lost,
                    texture: None,
                })
            },
            || Err(wasmtime::Error::msg("configuration failed")),
            || panic!("configuration failure must not loop"),
            || false,
        );
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("configuration failed"));
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::acquire_surface_texture;
    use std::cell::Cell;
    use wgpu_core::present::SurfaceOutput;
    use wgpu_types::SurfaceStatus;

    #[test]
    fn shutdown_prevents_acquisition() {
        let result = acquire_surface_texture::<u8>(
            || panic!("must not acquire after shutdown"),
            || panic!("must not reconfigure after shutdown"),
            || panic!("must not wait after shutdown"),
            || true,
        );
        assert!(matches!(
            result.unwrap_err().downcast_ref::<wasmtime::Trap>(),
            Some(wasmtime::Trap::Interrupt)
        ));
    }

    #[test]
    fn shutdown_interrupts_persistent_timeout_or_loss() {
        for lost in [false, true] {
            let cancelled = Cell::new(false);
            let mut acquisitions = 0;
            let result = acquire_surface_texture::<u8>(
                || {
                    acquisitions += 1;
                    Ok(SurfaceOutput {
                        status: if lost {
                            SurfaceStatus::Lost
                        } else {
                            SurfaceStatus::Timeout
                        },
                        texture: None,
                    })
                },
                || Ok(()),
                || cancelled.set(true),
                || cancelled.get(),
            );
            assert!(matches!(
                result.unwrap_err().downcast_ref::<wasmtime::Trap>(),
                Some(wasmtime::Trap::Interrupt)
            ));
            assert_eq!(acquisitions, 1);
        }
    }
}
