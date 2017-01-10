//! Items for reading and opening file formats from file.

use sample;
use std;
use Format;

#[cfg(feature="flac")]
use claxon;
#[cfg(feature="wav")]
use hound;
#[cfg(feature="ogg_vorbis")]
use lewton;


/// Types to which read samples may be converted via the `Reader::samples` method.
pub trait Sample: sample::Sample
    + sample::FromSample<i8>
    + sample::FromSample<i16>
    + sample::FromSample<i32>
    + sample::FromSample<f32> {}

impl<T> Sample for T
    where T: sample::Sample
           + sample::FromSample<i8>
           + sample::FromSample<i16>
           + sample::FromSample<i32>
           + sample::FromSample<f32> {}


/// Returned by the `read` function, enumerates the various supported readers.
pub enum Reader<R>
    where R: std::io::Read + std::io::Seek,
{
    #[cfg(feature="flac")]
    Flac(claxon::FlacReader<R>),
    #[cfg(feature="ogg_vorbis")]
    OggVorbis(lewton::inside_ogg::OggStreamReader<R>),
    #[cfg(feature="wav")]
    Wav(hound::WavReader<R>),
}


/// An iterator that reads samples from the underlying reader, converts them to the sample type `S`
/// if not already in that format and yields them.
pub struct Samples<'a, R, S>
    where R: 'a + std::io::Read + std::io::Seek,
{
    format: FormatSamples<'a, R>,
    sample: std::marker::PhantomData<S>,
}

// The inner part of the `Samples` iterator, specific to the format of the `Reader` used to produce
// the `Samples`.
enum FormatSamples<'a, R>
    where R: 'a + std::io::Read + std::io::Seek,
{
    #[cfg(feature="flac")]
    Flac(FlacSamples<'a, R>),

    #[cfg(feature="ogg_vorbis")]
    OggVorbis {
        reader: &'a mut lewton::inside_ogg::OggStreamReader<R>, 
        index: usize,
        buffer: Vec<i16>,
    },

    #[cfg(feature="wav")]
    Wav(WavSamples<'a, R>),
}

// The variants of flac's supported sample bit depths.
#[cfg(feature="flac")]
enum FlacSamples<'a, R: 'a>
    where R: std::io::Read,
{
    I8(claxon::FlacSamples<'a, R, i8>),
    I16(claxon::FlacSamples<'a, R, i16>),
    I32(claxon::FlacSamples<'a, R, i32>),
}

// The variants of hound's supported sample bit depths.
#[cfg(feature="wav")]
enum WavSamples<'a, R: 'a> {
    I8(hound::WavSamples<'a, R, i8>),
    I16(hound::WavSamples<'a, R, i16>),
    I32(hound::WavSamples<'a, R, i32>),
    F32(hound::WavSamples<'a, R, f32>),
}

/// An iterator that reads samples from the underlying reader, converts them to frames of type `F`
/// and yields them.
pub struct Frames<'a, R, F>
    where R: 'a + std::io::Read + std::io::Seek,
          F: sample::Frame,
{
    samples: Samples<'a, R, F::Sample>,
    frame: std::marker::PhantomData<F>,
}

/// An alias for the buffered, file `Reader` type returned from the `open` function.
pub type BufFileReader = Reader<std::io::BufReader<std::fs::File>>;

/// A description of the audio format that was read from file.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Description {
    channel_count: u32,
    sample_rate: u32,
}


/// Errors that might be returned from the `open` function.
#[derive(Debug)]
pub enum OpenError {
    Io(std::io::Error),
    Reader(FormatError),
    UnsupportedFormat { extension: String },
}

/// Format-specific errors that might occur when opening or reading from an audio file.
#[derive(Debug)]
pub enum FormatError {
    #[cfg(feature="flac")]
    Flac(claxon::Error),
    #[cfg(feature="ogg_vorbis")]
    OggVorbis(lewton::VorbisError),
    #[cfg(feature="wav")]
    Wav(hound::Error),
}


/// Attempts to open an audio `Reader` from the file at the specified `Path`.
///
/// The format is determined from the path's file extension.
pub fn open<P>(file_path: P) -> Result<BufFileReader, OpenError>
    where P: AsRef<std::path::Path>,
{
    BufFileReader::open(file_path)
}


impl Description {

    /// The number of channels of audio.
    ///
    /// E.g. For audio stored in stereo this should return `2`. Mono audio will return `1`.
    pub fn channel_count(&self) -> u32 {
        self.channel_count
    }

    /// The rate in Hertz at which each channel of the stored audio is sampled.
    ///
    /// E.g. A `sample_rate` of 44_100 indicates that the audio is sampled 44_100 times per second
    /// per channel.
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

}


impl BufFileReader {

    /// Attempts to open an audio `Reader` from the file at the specified `Path`.
    ///
    /// The format is determined from the path's file extension.
    pub fn open<P>(file_path: P) -> Result<Self, OpenError>
        where P: AsRef<std::path::Path>,
    {
        let path = file_path.as_ref();
        let file = try!(std::fs::File::open(path));
        let reader = std::io::BufReader::new(file);
        let extension = path.extension()
            .and_then(|s| s.to_str())
            .map_or_else(String::new, std::ascii::AsciiExt::to_ascii_lowercase);

        let format = match Format::from_extension(&extension) {
            Some(format) => format,
            None => return Err(OpenError::UnsupportedFormat { extension: extension }),
        };

        let reader = match format {
            #[cfg(feature="wav")]
            Format::Wav => {
                let reader = try!(hound::WavReader::new(reader));
                Reader::Wav(reader)
            },
            #[cfg(feature="ogg_vorbis")]
            Format::OggVorbis => {
                let reader = try!(lewton::inside_ogg::OggStreamReader::new(reader));
                Reader::OggVorbis(reader)
            },
            #[cfg(feature="flac")]
            Format::Flac => {
                let reader = try!(claxon::FlacReader::new(reader));
                Reader::Flac(reader)
            },
        };

        Ok(reader)
    }

}

impl<R> Reader<R>
    where R: std::io::Read + std::io::Seek,
{

    /// The format from which the audio will be read.
    pub fn format(&self) -> Format {
        match *self {
            #[cfg(feature="flac")]
            Reader::Flac(_) => Format::Flac,
            #[cfg(feature="ogg_vorbis")]
            Reader::OggVorbis(_) => Format::OggVorbis,
            #[cfg(feature="wav")]
            Reader::Wav(_) => Format::Wav,
        }
    }

    /// A basic description of the audio being read.
    pub fn description(&self) -> Description {
        match *self {

            #[cfg(feature="flac")]
            Reader::Flac(ref reader) => {
                let info = reader.streaminfo();
                Description {
                    channel_count: info.channels as u32,
                    sample_rate: info.sample_rate,
                }
            },

            #[cfg(feature="ogg_vorbis")]
            Reader::OggVorbis(ref reader) => {
                Description {
                    channel_count: reader.ident_hdr.audio_channels as u32,
                    sample_rate: reader.ident_hdr.audio_sample_rate as u32,
                }
            },

            #[cfg(feature="wav")]
            Reader::Wav(ref reader) => {
                let spec = reader.spec();
                Description {
                    channel_count: spec.channels as u32,
                    sample_rate: spec.sample_rate,
                }
            },

        }
    }

    /// Produce an iterator that reads samples from the underlying reader, converts them to the
    /// sample type `S` if not already in that format and yields them.
    ///
    /// When reading from multiple channels, samples are **interleaved**.
    pub fn samples<S>(&mut self) -> Samples<R, S>
        where S: Sample,
    {
        let format = match *self {

            #[cfg(feature="flac")]
            Reader::Flac(ref mut reader) => {
                let info = reader.streaminfo();
                match info.bits_per_sample {
                    8 => FormatSamples::Flac(FlacSamples::I8(reader.samples())),
                    16 => FormatSamples::Flac(FlacSamples::I16(reader.samples())),
                    _ => FormatSamples::Flac(FlacSamples::I32(reader.samples())),
                }
            },

            #[cfg(feature="ogg_vorbis")]
            Reader::OggVorbis(ref mut reader) => FormatSamples::OggVorbis {
                reader: reader,
                index: 0,
                buffer: Vec::new(),
            },

            #[cfg(feature="wav")]
            Reader::Wav(ref mut reader) => {
                let spec = reader.spec();
                match spec.sample_format {
                    hound::SampleFormat::Int => match spec.bits_per_sample {
                        8 => FormatSamples::Wav(WavSamples::I8(reader.samples())),
                        16 => FormatSamples::Wav(WavSamples::I16(reader.samples())),
                        _ => FormatSamples::Wav(WavSamples::I32(reader.samples())),
                    },
                    hound::SampleFormat::Float =>
                        FormatSamples::Wav(WavSamples::F32(reader.samples())),
                }
            },

        };

        Samples {
            format: format,
            sample: std::marker::PhantomData,
        }
    }

    /// Produce an iterator that yields read frames from the underlying `Reader`.
    ///
    /// This method currently expects that the frame type `F` has the same number of channels as
    /// stored in the underlying audio format.
    ///
    /// TODO: Should consider changing this behaviour to check the audio file's actual number of
    /// channels and automatically convert to `F`'s number of channels while reading.
    pub fn frames<F>(&mut self) -> Frames<R, F>
        where F: sample::Frame,
              F::Sample: Sample,
    {
        Frames {
            samples: self.samples(),
            frame: std::marker::PhantomData,
        }
    }

}


impl<'a, R, S> Iterator for Samples<'a, R, S>
    where R: std::io::Read + std::io::Seek,
          S: Sample,
{
    type Item = Result<S, FormatError>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.format {

            #[cfg(feature="flac")]
            FormatSamples::Flac(ref mut flac_samples) => {

                macro_rules! next_sample {
                    ($samples:expr) => {{
                        $samples.next().map(|sample| {
                            sample.map_err(FormatError::Flac).map(sample::Sample::to_sample)
                        })
                    }};
                }

                match *flac_samples {
                    FlacSamples::I8(ref mut samples) => next_sample!(samples),
                    FlacSamples::I16(ref mut samples) => next_sample!(samples),
                    FlacSamples::I32(ref mut samples) => next_sample!(samples),
                }
            },

            #[cfg(feature="ogg_vorbis")]
            FormatSamples::OggVorbis { ref mut reader, ref mut index, ref mut buffer } => loop {

                // Convert and return any pending samples.
                if *index < buffer.len() {
                    let sample = sample::Sample::to_sample(buffer[*index]);
                    *index += 1;
                    return Some(Ok(sample));
                }

                // If there are no samples left in the buffer, refill the buffer.
                match reader.read_dec_packet_itl() {
                    Ok(Some(packet)) => {
                        std::mem::replace(buffer, packet);
                        *index = 0;
                    },
                    Ok(None) => return None,
                    Err(err) => return Some(Err(err.into())),
                }
            },

            #[cfg(feature="wav")]
            FormatSamples::Wav(ref mut wav_samples) => {

                macro_rules! next_sample {
                    ($samples:expr) => {{
                        $samples.next().map(|sample| {
                            sample.map_err(FormatError::Wav).map(sample::Sample::to_sample)
                        })
                    }};
                }

                match *wav_samples {
                    WavSamples::I8(ref mut samples) => next_sample!(samples),
                    WavSamples::I16(ref mut samples) => next_sample!(samples),
                    WavSamples::I32(ref mut samples) => next_sample!(samples),
                    WavSamples::F32(ref mut samples) => next_sample!(samples),
                }
            },

        }
    }
}


impl<'a, R, F> Iterator for Frames<'a, R, F>
    where R: std::io::Read + std::io::Seek,
          F: sample::Frame,
          F::Sample: Sample,
{
    type Item = Result<F, FormatError>;
    fn next(&mut self) -> Option<Self::Item> {

        enum FrameConstruction {
            NotEnoughSamples,
            Ok,
            Err(FormatError),
        }

        let mut result = FrameConstruction::Ok;
        let frame = F::from_fn(|_| match self.samples.next() {
            Some(Ok(sample)) => sample,
            Some(Err(error)) => {
                result = FrameConstruction::Err(error.into());
                <F::Sample as sample::Sample>::equilibrium()
            },
            None => {
                result = FrameConstruction::NotEnoughSamples;
                <F::Sample as sample::Sample>::equilibrium()
            },
        });

        match result {
            FrameConstruction::Ok => Some(Ok(frame)),
            FrameConstruction::Err(error) => Some(Err(error)),
            FrameConstruction::NotEnoughSamples => None,
        }
    }
}


#[cfg(feature="flac")]
impl From<claxon::Error> for FormatError {
    fn from(err: claxon::Error) -> Self {
        FormatError::Flac(err)
    }
}

#[cfg(feature="ogg_vorbis")]
impl From<lewton::VorbisError> for FormatError {
    fn from(err: lewton::VorbisError) -> Self {
        FormatError::OggVorbis(err)
    }
}

#[cfg(feature="wav")]
impl From<hound::Error> for FormatError {
    fn from(err: hound::Error) -> Self {
        FormatError::Wav(err)
    }
}

impl<T> From<T> for OpenError
    where T: Into<FormatError>,
{
    fn from(err: T) -> Self {
        OpenError::Reader(err.into())
    }
}

impl From<std::io::Error> for OpenError {
    fn from(err: std::io::Error) -> Self {
        OpenError::Io(err)
    }
}


impl std::error::Error for FormatError {
    fn description(&self) -> &str {
        match *self {
            #[cfg(feature="flac")]
            FormatError::Flac(ref err) => std::error::Error::description(err),
            #[cfg(feature="ogg_vorbis")]
            FormatError::OggVorbis(ref err) => std::error::Error::description(err),
            #[cfg(feature="wav")]
            FormatError::Wav(ref err) => std::error::Error::description(err),
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            #[cfg(feature="flac")]
            FormatError::Flac(ref err) => Some(err),
            #[cfg(feature="ogg_vorbis")]
            FormatError::OggVorbis(ref err) => Some(err),
            #[cfg(feature="wav")]
            FormatError::Wav(ref err) => Some(err),
        }
    }
}

impl std::error::Error for OpenError {
    fn description(&self) -> &str {
        match *self {
            OpenError::Io(ref err) => std::error::Error::description(err),
            OpenError::Reader(ref err) => std::error::Error::description(err),
            OpenError::UnsupportedFormat { .. } => "no supported format was detected",
        }
    }
    fn cause(&self) -> Option<&std::error::Error> {
        match *self {
            OpenError::Io(ref err) => Some(err),
            OpenError::Reader(ref err) => Some(err),
            OpenError::UnsupportedFormat { .. } => None,
        }
    }
}


impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            #[cfg(feature="flac")]
            FormatError::Flac(ref err) => err.fmt(f),
            #[cfg(feature="ogg_vorbis")]
            FormatError::OggVorbis(ref err) => err.fmt(f),
            #[cfg(feature="wav")]
            FormatError::Wav(ref err) => err.fmt(f),
        }
    }
}

impl std::fmt::Display for OpenError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            OpenError::Io(ref err) => err.fmt(f),
            OpenError::Reader(ref err) => err.fmt(f),
            OpenError::UnsupportedFormat { ref extension } =>
                write!(f, "{}: {}", std::error::Error::description(self), extension),
        }
    }
}
