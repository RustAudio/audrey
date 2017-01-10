#![cfg(all(feature="flac", feature="ogg_vorbis", feature="wav"))]

extern crate audio;

const FLAC: &'static str = "samples/sine_440hz_stereo.flac";
const OGG_VORBIS: &'static str = "samples/sine_440hz_stereo.ogg";
const WAV: &'static str = "samples/sine_440hz_stereo.wav";

#[test]
fn open() {
    match audio::open(FLAC).unwrap() {
        audio::Reader::Flac(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    match audio::open(WAV).unwrap() {
        audio::Reader::Wav(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    match audio::open(OGG_VORBIS).unwrap() {
        audio::Reader::OggVorbis(_) => (),
        _ => panic!("Incorrect audio format"),
    }
}

#[test]
fn read_samples() {
    fn read_all_samples<P>(path: P) -> usize
        where P: AsRef<std::path::Path>,
    {
        let mut reader = audio::open(path).unwrap();
        reader.samples::<i16>().map(Result::unwrap).count()
    }

    // The original sample.
    let num_wav_samples = read_all_samples(WAV);
    // FLAC should be lossless.
    assert_eq!(num_wav_samples, read_all_samples(FLAC));
    // Ogg Vorbis is lossy.
    read_all_samples(OGG_VORBIS);
}
