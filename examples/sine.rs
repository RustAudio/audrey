//! Plays each of the `sine_440hz_stereo.*` audio files in the `samples/` directory in sequence.
//!
//! This example is adapted from the cpal `beep.rs` example, however rather than using an iterator
//! to generate samples, we read the samples from various file formats and convert them to the
//! target `device`'s default format.

use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

#[cfg(all(feature = "flac", feature = "ogg_vorbis", feature = "wav"))]
fn main() {
    // Use the audio crate to load the different audio formats and convert them to audio frames.
    let mut sine_flac = audrey::open("samples/sine_440hz_stereo.flac").unwrap();
    let mut sine_ogg_vorbis = audrey::open("samples/sine_440hz_stereo.ogg").unwrap();
    let mut sine_wav = audrey::open("samples/sine_440hz_stereo.wav").unwrap();

    // Chain together the frame-yielding iterators and collect them to create a cycling iterator.
    let sine_buffer = sine_flac
        .frames::<[i16; 2]>()
        .chain(sine_ogg_vorbis.frames::<[i16; 2]>())
        .chain(sine_wav.frames::<[i16; 2]>())
        .map(Result::unwrap)
        .map(|f| audrey::dasp_frame::Frame::scale_amp(f, 0.25)) // Scale down the amp to a friendly level.
        .collect::<Vec<_>>();
    let mut sine = sine_buffer.iter().cloned().cycle();

    // Setup the output device stream.
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");
    let format_range = device
        .supported_output_formats()
        .unwrap()
        .next()
        .expect("Failed to get endpoint format");
    let mut format = format_range.with_max_sample_rate();
    format.sample_rate = cpal::SampleRate(44_100);
    let event_loop = host.event_loop();
    let stream_id = event_loop
        .build_output_stream(&device, &format)
        .expect("Failed to create a voice");

    // A function for writing to the `cpal::Buffer`, whatever the default sample type may be.
    fn write_to_buffer<S, I>(mut buffer: cpal::OutputBuffer<S>, channels: usize, sine: &mut I)
    where
        S: cpal::Sample + audrey::dasp_sample::FromSample<i16>,
        I: Iterator<Item = [i16; 2]>,
    {
        match channels {
            // Mono
            1 => {
                for (frame, sine_frame) in buffer.chunks_mut(channels).zip(sine) {
                    let sum = sine_frame[0] + sine_frame[1];
                    frame[0] = audrey::dasp_sample::Sample::to_sample(sum);
                }
            }

            // Stereo
            2 => {
                for (frame, sine_frame) in buffer.chunks_mut(channels).zip(sine) {
                    for (sample, &sine_sample) in frame.iter_mut().zip(&sine_frame) {
                        *sample = audrey::dasp_sample::Sample::to_sample(sine_sample);
                    }
                }
            }

            _ => unimplemented!(),
        }
    }

    event_loop
        .play_stream(stream_id)
        .expect("failed to play_stream");

    event_loop.run(move |stream_id, buffer| {
        let stream_data = match buffer {
            Ok(data) => data,
            Err(err) => {
                eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
                return;
            }
        };

        match stream_data {
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::U16(buffer),
            } => write_to_buffer(buffer, usize::from(format.channels), &mut sine),
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::I16(buffer),
            } => write_to_buffer(buffer, usize::from(format.channels), &mut sine),
            cpal::StreamData::Output {
                buffer: cpal::UnknownTypeOutputBuffer::F32(buffer),
            } => write_to_buffer(buffer, usize::from(format.channels), &mut sine),
            _ => (),
        }
    });
}

#[cfg(not(all(feature = "flac", feature = "ogg_vorbis", feature = "wav")))]
fn main() {
    println!("This example requires all features to be enabled");
}
