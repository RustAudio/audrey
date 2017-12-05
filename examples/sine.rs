//! Plays each of the `sine_440hz_stereo.*` audio files in the `samples/` directory in sequence.
//!
//! This example is adapted from the cpal `beep.rs` example, however rather than using an iterator
//! to generate samples, we read the samples from various file formats and convert them to the
//! target `endpoint`'s default format.

extern crate audrey;
extern crate cpal;

#[cfg(all(feature="flac", feature="ogg_vorbis", feature="wav"))]
fn main() {

    // Use the audio crate to load the different audio formats and convert them to audio frames.
    let mut sine_flac = audrey::open("samples/sine_440hz_stereo.flac").unwrap();
    let mut sine_ogg_vorbis = audrey::open("samples/sine_440hz_stereo.ogg").unwrap();
    let mut sine_wav = audrey::open("samples/sine_440hz_stereo.wav").unwrap();
    let sine_frames: Vec<[i16; 2]> = sine_flac.frames()
        .chain(sine_ogg_vorbis.frames())
        .chain(sine_wav.frames())
        .map(Result::unwrap)
        .collect();
    let mut sine = sine_frames.into_iter().cycle();

    let endpoint = cpal::default_endpoint().expect("Failed to get endpoint");
    let format_range = endpoint.supported_formats().unwrap().next().expect("Failed to get endpoint format");
    let format = format_range.with_max_samples_rate();

    let event_loop = cpal::EventLoop::new();
    let voice_id = event_loop.build_voice(&endpoint, &format).expect("Failed to create a voice");

    // A function for writing to the `cpal::Buffer`, whatever the default sample type may be.
    fn write_to_buffer<S, I>(mut buffer: cpal::Buffer<S>, channels: usize, sine: &mut I)
        where S: cpal::Sample + audrey::sample::FromSample<i16>,
              I: Iterator<Item=[i16; 2]>,
    {
        match channels {

            // Mono
            1 => for (frame, sine_frame) in buffer.chunks_mut(channels).zip(sine) {
                let sum = sine_frame[0] + sine_frame[1];
                frame[0] = audrey::sample::Sample::to_sample(sum);
            },

            // Stereo
            2 => for (frame, sine_frame) in buffer.chunks_mut(channels).zip(sine) {
                for (sample, &sine_sample) in frame.iter_mut().zip(&sine_frame) {
                    *sample = audrey::sample::Sample::to_sample(sine_sample);
                }
            },

            _ => unimplemented!(),
        }
    }

    event_loop.play(voice_id);

    event_loop.run(move |_voice_id, buffer| {
        match buffer {
            cpal::UnknownTypeBuffer::U16(buffer) => write_to_buffer(buffer, format.channels.len(), &mut sine),
            cpal::UnknownTypeBuffer::I16(buffer) => write_to_buffer(buffer, format.channels.len(), &mut sine),
            cpal::UnknownTypeBuffer::F32(buffer) => write_to_buffer(buffer, format.channels.len(), &mut sine),
        };
    });
}

#[cfg(not(all(feature="flac", feature="ogg_vorbis", feature="wav")))]
fn main() {
    println!("This example requires all features to be enabled");
}
