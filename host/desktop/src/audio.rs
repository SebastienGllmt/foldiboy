use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::traits::*;
use ringbuf::HeapRb;
use wasmtime::component::{HasData, Linker, Resource};
use wasmtime_wasi::ResourceTable;
use wasmtime_wasi_io::IoView;

// ---------------------------------------------------------------------------
// WIT bindings (host side)
// ---------------------------------------------------------------------------

wasmtime::component::bindgen!({
    path: "../../lib/wit/deps/nightstream-audio",
    world: "imports",
    with: {
        "nightstream:audio/audio.audio-output": AudioStream,
    },
});

use nightstream::audio::audio;

// ---------------------------------------------------------------------------
// Audio stream (resource backing type)
// ---------------------------------------------------------------------------

/// Host-side data for each `audio-output` resource instance.
/// Holds the producer half of a lock-free ring buffer and the cpal stream.
pub struct AudioStream {
    producer: ringbuf::HeapProd<i16>,
    /// Kept alive so audio keeps playing. Dropped when resource is dropped.
    _stream: Option<cpal::Stream>,
}

// cpal::Stream is Send on ALSA (the only backend we target).
// The HeapProd is already Send.
unsafe impl Send for AudioStream {}

impl AudioStream {
    fn new_with_device(sample_rate: u32, channels: u32) -> Self {
        let host = cpal::default_host();

        if let Some(device) = host.default_output_device() {
            match Self::open_device(&device, sample_rate, channels) {
                Ok(stream) => return stream,
                Err(error) => {
                    log::warn!("default audio output failed: {error}; trying a sound server")
                }
            }
        }

        // ALSA's default can point at an absent hardware PCM even when the
        // desktop sound server is available. Keep this fallback local to the app.
        #[cfg(target_os = "linux")]
        if let Ok(devices) = host.devices() {
            for device in devices {
                let name = device.name().unwrap_or_default();
                if name != "pipewire" && name != "pulse" {
                    continue;
                }
                match Self::open_device(&device, sample_rate, channels) {
                    Ok(stream) => return stream,
                    Err(error) => log::warn!("audio output {name} failed: {error}"),
                }
            }
        }
        log::warn!("no usable audio output — audio disabled");
        Self::silent()
    }

    fn open_device(device: &cpal::Device, sample_rate: u32, channels: u32) -> Result<Self, String> {
        let config = cpal::StreamConfig {
            channels: channels as u16,
            sample_rate: cpal::SampleRate(sample_rate),
            buffer_size: cpal::BufferSize::Default,
        };

        // ~100ms ring buffer at 48kHz stereo
        let capacity = (sample_rate as usize) * (channels as usize) / 10;
        let rb = HeapRb::<i16>::new(capacity);
        let (producer, mut consumer) = rb.split();

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                    let filled = consumer.pop_slice(data);
                    for sample in &mut data[filled..] {
                        *sample = 0;
                    }
                },
                |err| log::error!("audio stream error: {err}"),
                None,
            )
            .map_err(|error| error.to_string())?;
        stream.play().map_err(|error| error.to_string())?;
        log::info!(
            "audio: opened {}Hz {}ch output on {}",
            sample_rate,
            channels,
            device.name().unwrap_or_default()
        );
        Ok(AudioStream {
            producer,
            _stream: Some(stream),
        })
    }

    fn silent() -> Self {
        let rb = HeapRb::<i16>::new(8192);
        let (producer, _consumer) = rb.split();
        AudioStream {
            producer,
            _stream: None,
        }
    }
}

// ---------------------------------------------------------------------------
// View trait + wrapper (follows wasi-gfx pattern)
// ---------------------------------------------------------------------------

pub trait AudioView: IoView + Send {}

impl<T: ?Sized + AudioView> AudioView for &mut T {}

#[repr(transparent)]
struct AudioImpl<T>(T);

impl<T: AudioView> IoView for AudioImpl<T> {
    fn table(&mut self) -> &mut ResourceTable {
        T::table(&mut self.0)
    }
}

impl<T: AudioView> AudioView for AudioImpl<T> {}

struct AudioHasData<T: Send>(T);
impl<T: Send + 'static> HasData for AudioHasData<T> {
    type Data<'a> = AudioImpl<&'a mut T>;
}

// ---------------------------------------------------------------------------
// WIT trait implementations (on wrapper to avoid blanket impl conflicts)
// ---------------------------------------------------------------------------

impl<T: AudioView> audio::Host for AudioImpl<T> {}

impl<T: AudioView> audio::HostAudioOutput for AudioImpl<T> {
    fn new(&mut self, config: audio::AudioConfig) -> Resource<AudioStream> {
        let stream = AudioStream::new_with_device(config.sample_rate, config.channels);
        self.table().push(stream).unwrap()
    }

    fn write(&mut self, self_: Resource<AudioStream>, samples: Vec<i16>) -> u32 {
        let stream = self.table().get_mut(&self_).unwrap();
        stream.producer.push_slice(&samples) as u32
    }

    fn available(&mut self, self_: Resource<AudioStream>) -> u32 {
        let stream = self.table().get(&self_).unwrap();
        stream.producer.vacant_len() as u32
    }

    fn drop(&mut self, resource: Resource<AudioStream>) -> wasmtime::Result<()> {
        self.table().delete(resource)?;
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Linker registration
// ---------------------------------------------------------------------------

pub fn add_to_linker<T: AudioView + 'static>(linker: &mut Linker<T>) -> wasmtime::Result<()> {
    audio::add_to_linker::<_, AudioHasData<T>>(linker, |x| AudioImpl(x))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::AudioStream;

    #[test]
    #[ignore = "requires a desktop audio server or output device"]
    fn opens_desktop_audio_output() {
        let stream = AudioStream::new_with_device(48000, 2);
        assert!(
            stream._stream.is_some(),
            "audio unexpectedly fell back to silence"
        );
    }
}
