#![cfg(all(feature = "flac", feature = "ogg_vorbis", feature = "wav"))]

extern crate audrey;

const FLAC: &'static str = "samples/sine_440hz_stereo.flac";
const OGG_VORBIS: &'static str = "samples/sine_440hz_stereo.ogg";
const WAV: &'static str = "samples/sine_440hz_stereo.wav";
const CAF_ALAC: &'static str = "samples/sine_440hz_stereo.caf";

#[test]
fn read() {
    let flac = std::io::BufReader::new(std::fs::File::open(FLAC).unwrap());
    match audrey::Reader::new(flac).unwrap() {
        audrey::Reader::Flac(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    let wav = std::io::BufReader::new(std::fs::File::open(WAV).unwrap());
    match audrey::Reader::new(wav).unwrap() {
        audrey::Reader::Wav(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    let ogg_vorbis = std::io::BufReader::new(std::fs::File::open(OGG_VORBIS).unwrap());
    match audrey::Reader::new(ogg_vorbis).unwrap() {
        audrey::Reader::OggVorbis(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    let caf_alac = std::io::BufReader::new(std::fs::File::open(CAF_ALAC).unwrap());
    match audrey::Reader::new(caf_alac).unwrap() {
        audrey::Reader::CafAlac(_) => (),
        _ => panic!("Incorrect audio format"),
    }
}

#[test]
fn open() {
    match audrey::open(FLAC).unwrap() {
        audrey::Reader::Flac(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    match audrey::open(WAV).unwrap() {
        audrey::Reader::Wav(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    match audrey::open(OGG_VORBIS).unwrap() {
        audrey::Reader::OggVorbis(_) => (),
        _ => panic!("Incorrect audio format"),
    }
    match audrey::open(CAF_ALAC).unwrap() {
        audrey::Reader::CafAlac(_) => (),
        _ => panic!("Incorrect audio format"),
    }
}

#[test]
fn open_and_read_samples() {
    fn read_samples<P>(path: P) -> usize
    where
        P: AsRef<std::path::Path>,
    {
        let mut reader = audrey::open(path).unwrap();
        reader.samples::<i16>().map(Result::unwrap).count()
    }

    // The original sample.
    let num_wav_samples = read_samples(WAV);
    // FLAC should be lossless.
    assert_eq!(num_wav_samples, read_samples(FLAC));
    // Ogg Vorbis is lossy.
    read_samples(OGG_VORBIS);
}
