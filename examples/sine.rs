//! Plays each of the `sine_440hz_stereo.*` audio files in the `samples/` directory in sequence.
//!
//! This example is adapted from the cpal `beep.rs` example, however rather than using an iterator
//! to generate samples, we read the samples from various file formats and convert them to the
//! target `endpoint`'s default format.

extern crate audio;
extern crate cpal;
extern crate futures;

#[cfg(all(feature="wav", feature="ogg_vorbis"))]
fn main() {
    use futures::stream::Stream;

    // Use the audio crate to load the different audio formats and convert them to audio frames.
    let mut sine_flac = audio::open("samples/sine_440hz_stereo.flac").unwrap();
    let mut sine_ogg_vorbis = audio::open("samples/sine_440hz_stereo.ogg").unwrap();
    let mut sine_wav = audio::open("samples/sine_440hz_stereo.wav").unwrap();
    let sine_frames: Vec<[i16; 2]> = sine_flac.frames()
        .chain(sine_ogg_vorbis.frames())
        .chain(sine_wav.frames())
        .map(Result::unwrap)
        .collect();
    let mut sine = sine_frames.into_iter().cycle();

    let endpoint = cpal::get_default_endpoint().expect("Failed to get endpoint");
    let format = endpoint.get_supported_formats_list().unwrap().next().expect("Failed to get endpoint format");

    struct Executor;
    impl futures::task::Executor for Executor {
        fn execute(&self, r: futures::task::Run) {
            r.run();
        }
    }

    let executor = std::sync::Arc::new(Executor);
    let event_loop = cpal::EventLoop::new();
    let (mut voice, stream) = cpal::Voice::new(&endpoint, &format, &event_loop).expect("Failed to create a voice");

    // A function for writing to the `cpal::Buffer`, whatever the default sample type may be.
    fn write_to_buffer<S, I>(mut buffer: cpal::Buffer<S>, channels: usize, sine: &mut I)
        where S: cpal::Sample + audio::sample::FromSample<i16>,
              I: Iterator<Item=[i16; 2]>,
    {
        match channels {

            // Mono
            1 => for (frame, sine_frame) in buffer.chunks_mut(channels).zip(sine) {
                let sum = sine_frame[0] + sine_frame[1];
                frame[0] = audio::sample::Sample::to_sample(sum);
            },

            // Stereo
            2 => for (frame, sine_frame) in buffer.chunks_mut(channels).zip(sine) {
                for (sample, &sine_sample) in frame.iter_mut().zip(&sine_frame) {
                    *sample = audio::sample::Sample::to_sample(sine_sample);
                }
            },

            _ => unimplemented!(),
        }
    }

    futures::task::spawn(stream.for_each(move |buffer| -> Result<_, ()> {
        match buffer {
            cpal::UnknownTypeBuffer::U16(buffer) => write_to_buffer(buffer, format.channels.len(), &mut sine),
            cpal::UnknownTypeBuffer::I16(buffer) => write_to_buffer(buffer, format.channels.len(), &mut sine),
            cpal::UnknownTypeBuffer::F32(buffer) => write_to_buffer(buffer, format.channels.len(), &mut sine),
        };
        Ok(())
    })).execute(executor);

    std::thread::spawn(move || {
        loop {
            voice.play();
            std::thread::sleep(std::time::Duration::from_secs(1));
            voice.pause();
            std::thread::sleep(std::time::Duration::from_millis(500));
        }
    });

    event_loop.run();
}

#[cfg(not(all(feature="wav", feature="ogg_vorbis")))]
fn main() {
    println!("This example requires all features to be enabled");
}
