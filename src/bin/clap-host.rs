use aaw::APP_VERSION;
use clack_host::events::event_types::*;
use clack_host::prelude::*;

// Prepare our (extremely basic) host implementation

struct MyHostShared;

impl<'a> SharedHandler<'a> for MyHostShared {
    /* ... */
    fn request_restart(&self) { /* ... */
    }
    fn request_process(&self) { /* ... */
    }
    fn request_callback(&self) { /* ... */
    }
}

struct MyHost;

impl HostHandlers for MyHost {
    type Shared<'a> = MyHostShared;

    type MainThread<'a> = ();
    type AudioProcessor<'a> = ();
}

fn main() -> anyhow::Result<()> {
    // Information about our totally legit host.
    let host_info = HostInfo::new("Arri's Audio Workstation", "Arri", "", APP_VERSION)?;

    let entry = unsafe {
        PluginEntry::load(
            "/home/stravnicek/Dev/Python/hurdy-gurdy/aaw-hurdy-gurdy-plugin/target/release/aaw-hurdy-gurdy.clap",
        )?
    };
    let plugin_factory = entry.get_plugin_factory().unwrap();

    for descriptor in plugin_factory.plugin_descriptors() {
        println!("- {}", descriptor.id().unwrap().to_string_lossy());
    }

    let plugin_descriptor = plugin_factory
        .plugin_descriptors()
        .find(|d| d.id().unwrap().to_bytes() == b"audio.aaw.hurdy-gurdy")
        .unwrap();

    println!("desc: {}", plugin_descriptor.description().unwrap().to_string_lossy());

    let mut plugin_instance = PluginInstance::<MyHost>::new(
        |_| MyHostShared,
        |_| (),
        &entry,
        plugin_descriptor.id().unwrap(),
        &host_info,
    )?;

    let audio_configuration = PluginAudioConfiguration {
        sample_rate: 48_000.0,
        min_frames_count: 4,
        max_frames_count: 4,
    };
    let audio_processor = plugin_instance.activate(|_, _| (), audio_configuration)?;

    let note_on_event = NoteOnEvent::new(0, Pckn::new(0u16, 0u16, 12u16, 60u32), 4.2);
    let input_events_buffer = [note_on_event];
    let mut output_events_buffer = EventBuffer::new();

    let mut input_audio_buffers = [[0.0f32; 4]; 2]; // 2 channels (stereo), 1 port
    let mut output_audio_buffers = [[0.0f32; 4]; 2];

    let mut input_ports = AudioPorts::with_capacity(2, 1); // 2 channels (stereo), 1 port
    let mut output_ports = AudioPorts::with_capacity(2, 1);

    // Let's send the audio processor to a dedicated audio processing thread.
    let audio_processor = std::thread::scope(|s| {
        s.spawn(|| {
            let mut audio_processor = audio_processor.start_processing().unwrap();

            let input_events = InputEvents::from_buffer(&input_events_buffer);
            let mut output_events = OutputEvents::from_buffer(&mut output_events_buffer);

            let input_audio = input_ports.with_input_buffers([AudioPortBuffer {
                latency: 0,
                channels: AudioPortBufferType::f32_input_only(
                    input_audio_buffers.iter_mut().map(InputChannel::constant),
                ),
            }]);

            let mut output_audio = output_ports.with_output_buffers([AudioPortBuffer {
                latency: 0,
                channels: AudioPortBufferType::f32_output_only(
                    output_audio_buffers.iter_mut().map(|b| b.as_mut_slice()),
                ),
            }]);

            // Finally do the processing itself.
            let _ = audio_processor
                .process(
                    &input_audio,
                    &mut output_audio,
                    &input_events,
                    &mut output_events,
                    None,
                    None,
                )
                .unwrap();

            // Send the audio processor back to be deallocated by the main thread.
            audio_processor.stop_processing()
        })
        .join()
        .unwrap()
    });

    plugin_instance.deactivate(audio_processor);
    Ok(())
}
