#![cfg(all(feature="flac", feature="ogg_vorbis", feature="wav"))]

extern crate audio;

const FLAC: &'static str = "samples/sine_440hz_stereo.flac";
const OGG_VORBIS: &'static str = "samples/sine_440hz_stereo.ogg";
const WAV: &'static str = "samples/sine_440hz_stereo.wav";

#[test]
fn read() {
    let flac = std::io::BufReader::new(std::fs::File::open(FLAC).unwrap());
    match audio::Reader::new(flac).unwrap() {
        audio::Reader::Flac(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    let wav = std::io::BufReader::new(std::fs::File::open(WAV).unwrap());
    match audio::Reader::new(wav).unwrap() {
        audio::Reader::Wav(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    let ogg_vorbis = std::io::BufReader::new(std::fs::File::open(OGG_VORBIS).unwrap());
    match audio::Reader::new(ogg_vorbis).unwrap() {
        audio::Reader::OggVorbis(_) => (),
        _ => panic!("Incorrect audio format"),
    }
}

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
fn open_and_read_samples() {
    fn read_samples<P>(path: P) -> usize
        where P: AsRef<std::path::Path>,
    {
        let mut reader = audio::open(path).unwrap();
        reader.samples::<i16>().map(Result::unwrap).count()
    }

    // The original sample.
    let num_wav_samples = read_samples(WAV);
    // FLAC should be lossless.
    assert_eq!(num_wav_samples, read_samples(FLAC));
    // Ogg Vorbis is lossy.
    read_samples(OGG_VORBIS);
}
