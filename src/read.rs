//! Items for reading and opening file formats from file.

use crate::Format;

#[cfg(feature = "caf")]
use caf::{self, CafError};
#[cfg(feature = "flac")]
use claxon;
#[cfg(feature = "wav")]
use hound;
#[cfg(feature = "ogg_vorbis")]
use lewton;

/// Types to which read samples may be converted via the `Reader::samples` method.
pub trait Sample:
    sample::Sample
    + sample::FromSample<i8>
    + sample::FromSample<i16>
    + sample::FromSample<sample::I24>
    + sample::FromSample<i32>
    + sample::FromSample<f32>
{
}

impl<T> Sample for T where
    T: sample::Sample
        + sample::FromSample<i8>
        + sample::FromSample<i16>
        + sample::FromSample<sample::I24>
        + sample::FromSample<i32>
        + sample::FromSample<f32>
{
}

/// Returned by the `read` function, enumerates the various supported readers.
pub enum Reader<R>
where
    R: std::io::Read + std::io::Seek,
{
    #[cfg(feature = "flac")]
    Flac(claxon::FlacReader<R>),
    #[cfg(feature = "ogg_vorbis")]
    OggVorbis(lewton::inside_ogg::OggStreamReader<R>),
    #[cfg(feature = "wav")]
    Wav(hound::WavReader<R>),
    #[cfg(feature = "caf_alac")]
    CafAlac(super::caf_alac::AlacReader<R>),
}

/// An iterator that reads samples from the underlying reader, converts them to the sample type `S`
/// if not already in that format and yields them.
pub struct Samples<'a, R, S>
where
    R: 'a + std::io::Read + std::io::Seek,
{
    format: FormatSamples<'a, R>,
    sample: std::marker::PhantomData<S>,
}

// The inner part of the `Samples` iterator, specific to the format of the `Reader` used to produce
// the `Samples`.
enum FormatSamples<'a, R>
where
    R: 'a + std::io::Read + std::io::Seek,
{
    #[cfg(feature = "flac")]
    Flac(claxon::FlacSamples<&'a mut claxon::input::BufferedReader<R>>),

    #[cfg(feature = "ogg_vorbis")]
    OggVorbis {
        reader: &'a mut lewton::inside_ogg::OggStreamReader<R>,
        index: usize,
        buffer: Vec<i16>,
    },

    #[cfg(feature = "wav")]
    Wav(WavSamples<'a, R>),

    #[cfg(feature = "caf_alac")]
    CafAlac {
        reader: &'a mut super::caf_alac::AlacReader<R>,
        index: usize,
        buffer: Vec<i32>,
    },
}

// The variants of hound's supported sample bit depths.
#[cfg(feature = "wav")]
enum WavSamples<'a, R: 'a> {
    I8(hound::WavSamples<'a, R, i8>),
    I16(hound::WavSamples<'a, R, i16>),
    I24(hound::WavSamples<'a, R, i32>),
    I32(hound::WavSamples<'a, R, i32>),
    F32(hound::WavSamples<'a, R, f32>),
}

/// An iterator that reads samples from the underlying reader, converts them to frames of type `F`
/// and yields them.
pub struct Frames<'a, R, F>
where
    R: 'a + std::io::Read + std::io::Seek,
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
    format: Format,
    channel_count: u32,
    sample_rate: u32,
}

/// Errors that might be returned from the `Reader::new` function.
#[derive(Debug)]
pub enum ReadError {
    Io(std::io::Error),
    Reader(FormatError),
    UnsupportedFormat,
}

/// Format-specific errors that might occur when opening or reading from an audio file.
#[derive(Debug)]
pub enum FormatError {
    #[cfg(feature = "flac")]
    Flac(claxon::Error),
    #[cfg(feature = "ogg_vorbis")]
    OggVorbis(lewton::VorbisError),
    #[cfg(feature = "wav")]
    Wav(hound::Error),
    #[cfg(feature = "caf")]
    Caf(caf::CafError),
    #[cfg(feature = "alac")]
    Alac(()),
}

/// Attempts to open an audio `Reader` from the file at the specified `Path`.
///
/// The format is determined from the path's file extension.
pub fn open<P>(file_path: P) -> Result<BufFileReader, ReadError>
where
    P: AsRef<std::path::Path>,
{
    BufFileReader::open(file_path)
}

impl Description {
    /// The format from which the audio will be read.
    pub fn format(&self) -> Format {
        self.format
    }

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
    /// This function is a convenience wrapper around the `Reader::new` function.
    ///
    /// This function pays no attention to the `file_path`'s extension and instead attempts to read
    /// a supported `Format` via the file header.
    pub fn open<P>(file_path: P) -> Result<Self, ReadError>
    where
        P: AsRef<std::path::Path>,
    {
        let path = file_path.as_ref();
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);
        Reader::new(reader)
    }
}

impl<R> Reader<R>
where
    R: std::io::Read + std::io::Seek,
{
    /// Attempts to read the format of the audio read by the given `reader` and returns the associated
    /// `Reader` variant.
    ///
    /// The format is determined by attempting to construct each specific format reader until one
    /// is successful.
    pub fn new(mut reader: R) -> Result<Self, ReadError> {
        #[cfg(feature = "wav")]
        {
            let is_wav = match hound::WavReader::new(&mut reader) {
                Err(hound::Error::FormatError(_)) => false,
                Err(err) => return Err(err.into()),
                Ok(_) => true,
            };
            reader.seek(std::io::SeekFrom::Start(0))?;
            if is_wav {
                return Ok(Reader::Wav(hound::WavReader::new(reader)?));
            }
        }

        #[cfg(feature = "flac")]
        {
            let is_flac = match claxon::FlacReader::new(&mut reader) {
                Err(claxon::Error::FormatError(_)) => false,
                Err(err) => return Err(err.into()),
                Ok(_) => true,
            };
            reader.seek(std::io::SeekFrom::Start(0))?;
            if is_flac {
                return Ok(Reader::Flac(claxon::FlacReader::new(reader)?));
            }
        }

        #[cfg(feature = "ogg_vorbis")]
        {
            let is_ogg_vorbis = match lewton::inside_ogg::OggStreamReader::new(&mut reader) {
                Err(lewton::VorbisError::OggError(_))
                | Err(lewton::VorbisError::BadHeader(
                    lewton::header::HeaderReadError::NotVorbisHeader,
                )) => false,
                Err(err) => return Err(err.into()),
                Ok(_) => true,
            };
            reader.seek(std::io::SeekFrom::Start(0))?;
            if is_ogg_vorbis {
                return Ok(Reader::OggVorbis(lewton::inside_ogg::OggStreamReader::new(
                    reader,
                )?));
            }
        }

        #[cfg(feature = "caf_alac")]
        {
            let is_caf_alac = match super::caf_alac::AlacReader::new(&mut reader) {
                Err(FormatError::Caf(CafError::NotCaf)) => false,
                Err(err) => return Err(err.into()),
                // There is a CAF container, but no ALAC inside
                Ok(None) => false,
                // Everything is fine!
                Ok(Some(_)) => true,
            };
            reader.seek(std::io::SeekFrom::Start(0))?;
            if is_caf_alac {
                return Ok(Reader::CafAlac(
                    super::caf_alac::AlacReader::new(reader)?.unwrap(),
                ));
            }
        }

        Err(ReadError::UnsupportedFormat)
    }

    /// The format from which the audio will be read.
    pub fn format(&self) -> Format {
        match *self {
            #[cfg(feature = "flac")]
            Reader::Flac(_) => Format::Flac,
            #[cfg(feature = "ogg_vorbis")]
            Reader::OggVorbis(_) => Format::OggVorbis,
            #[cfg(feature = "wav")]
            Reader::Wav(_) => Format::Wav,
            #[cfg(feature = "caf_alac")]
            Reader::CafAlac(_) => Format::CafAlac,
        }
    }

    /// A basic description of the audio being read.
    pub fn description(&self) -> Description {
        match *self {
            #[cfg(feature = "flac")]
            Reader::Flac(ref reader) => {
                let info = reader.streaminfo();
                Description {
                    format: Format::Flac,
                    channel_count: info.channels as u32,
                    sample_rate: info.sample_rate,
                }
            }

            #[cfg(feature = "ogg_vorbis")]
            Reader::OggVorbis(ref reader) => Description {
                format: Format::OggVorbis,
                channel_count: u32::from(reader.ident_hdr.audio_channels),
                sample_rate: reader.ident_hdr.audio_sample_rate as u32,
            },

            #[cfg(feature = "wav")]
            Reader::Wav(ref reader) => {
                let spec = reader.spec();
                Description {
                    format: Format::Wav,
                    channel_count: u32::from(spec.channels),
                    sample_rate: spec.sample_rate,
                }
            }

            #[cfg(feature = "caf_alac")]
            Reader::CafAlac(ref reader) => {
                let desc = &reader.caf_reader.audio_desc;
                Description {
                    format: Format::CafAlac,
                    channel_count: desc.channels_per_frame as u32,
                    sample_rate: (1.0 / desc.sample_rate) as u32,
                }
            }
        }
    }

    /// Produce an iterator that reads samples from the underlying reader, converts them to the
    /// sample type `S` if not already in that format and yields them.
    ///
    /// When reading from multiple channels, samples are **interleaved**.
    pub fn samples<S>(&mut self) -> Samples<R, S>
    where
        S: Sample,
    {
        let format = match *self {
            #[cfg(feature = "flac")]
            Reader::Flac(ref mut reader) => FormatSamples::Flac(reader.samples()),

            #[cfg(feature = "ogg_vorbis")]
            Reader::OggVorbis(ref mut reader) => FormatSamples::OggVorbis {
                reader,
                index: 0,
                buffer: Vec::new(),
            },

            #[cfg(feature = "wav")]
            Reader::Wav(ref mut reader) => {
                let spec = reader.spec();
                match spec.sample_format {
                    hound::SampleFormat::Int => match spec.bits_per_sample {
                        8 => FormatSamples::Wav(WavSamples::I8(reader.samples())),
                        16 => FormatSamples::Wav(WavSamples::I16(reader.samples())),
                        24 => FormatSamples::Wav(WavSamples::I24(reader.samples())),
                        32 => FormatSamples::Wav(WavSamples::I32(reader.samples())),
                        // Should there be an error here?
                        _ => FormatSamples::Wav(WavSamples::I32(reader.samples())),
                    },
                    hound::SampleFormat::Float => {
                        FormatSamples::Wav(WavSamples::F32(reader.samples()))
                    }
                }
            }

            #[cfg(feature = "caf_alac")]
            Reader::CafAlac(ref mut reader) => FormatSamples::CafAlac {
                reader,
                index: 0,
                buffer: Vec::new(),
            },
        };

        Samples {
            format,
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
    where
        F: sample::Frame,
        F::Sample: Sample,
    {
        Frames {
            samples: self.samples(),
            frame: std::marker::PhantomData,
        }
    }
}

impl<'a, R, S> Iterator for Samples<'a, R, S>
where
    R: std::io::Read + std::io::Seek,
    S: Sample,
{
    type Item = Result<S, FormatError>;
    fn next(&mut self) -> Option<Self::Item> {
        match self.format {
            #[cfg(feature = "flac")]
            FormatSamples::Flac(ref mut flac_samples) => flac_samples.next().map(|sample| {
                sample
                    .map_err(FormatError::Flac)
                    .map(sample::Sample::to_sample)
            }),

            #[cfg(feature = "ogg_vorbis")]
            FormatSamples::OggVorbis {
                ref mut reader,
                ref mut index,
                ref mut buffer,
            } => loop {
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
                    }
                    Ok(None) => return None,
                    Err(err) => return Some(Err(err.into())),
                }
            },

            #[cfg(feature = "wav")]
            FormatSamples::Wav(ref mut wav_samples) => {
                macro_rules! next_sample {
                    ($samples:expr) => {{
                        $samples.next().map(|sample| {
                            sample
                                .map_err(FormatError::Wav)
                                .map(sample::Sample::to_sample)
                        })
                    }};
                }

                match *wav_samples {
                    WavSamples::I8(ref mut samples) => next_sample!(samples),
                    WavSamples::I16(ref mut samples) => next_sample!(samples),
                    WavSamples::I24(ref mut samples) => samples.next().map(|sample| {
                        sample
                            .map_err(FormatError::Wav)
                            .map(sample::I24::new_unchecked)
                            .map(sample::Sample::to_sample)
                    }),
                    WavSamples::I32(ref mut samples) => next_sample!(samples),
                    WavSamples::F32(ref mut samples) => next_sample!(samples),
                }
            }

            #[cfg(feature = "caf_alac")]
            FormatSamples::CafAlac {
                ref mut reader,
                ref mut index,
                ref mut buffer,
            } => loop {
                // Convert and return any pending samples.
                if *index < buffer.len() {
                    let sample = sample::Sample::to_sample(buffer[*index]);
                    *index += 1;
                    return Some(Ok(sample));
                }

                // If there are no samples left in the buffer, refill the buffer.
                match reader.read_packet() {
                    Ok(Some(packet)) => {
                        std::mem::replace(buffer, packet);
                        *index = 0;
                    }
                    Ok(None) => return None,
                    Err(err) => return Some(Err(err)),
                }
            },
        }
    }
}

impl<'a, R, F> Iterator for Frames<'a, R, F>
where
    R: std::io::Read + std::io::Seek,
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
                result = FrameConstruction::Err(error);
                <F::Sample as sample::Sample>::equilibrium()
            }
            None => {
                result = FrameConstruction::NotEnoughSamples;
                <F::Sample as sample::Sample>::equilibrium()
            }
        });

        match result {
            FrameConstruction::Ok => Some(Ok(frame)),
            FrameConstruction::Err(error) => Some(Err(error)),
            FrameConstruction::NotEnoughSamples => None,
        }
    }
}

#[cfg(feature = "flac")]
impl From<claxon::Error> for FormatError {
    fn from(err: claxon::Error) -> Self {
        FormatError::Flac(err)
    }
}

#[cfg(feature = "ogg_vorbis")]
impl From<lewton::VorbisError> for FormatError {
    fn from(err: lewton::VorbisError) -> Self {
        FormatError::OggVorbis(err)
    }
}

#[cfg(feature = "wav")]
impl From<hound::Error> for FormatError {
    fn from(err: hound::Error) -> Self {
        FormatError::Wav(err)
    }
}

#[cfg(feature = "caf")]
impl From<CafError> for FormatError {
    fn from(err: CafError) -> Self {
        FormatError::Caf(err)
    }
}

impl<T> From<T> for ReadError
where
    T: Into<FormatError>,
{
    fn from(err: T) -> Self {
        ReadError::Reader(err.into())
    }
}

impl From<std::io::Error> for ReadError {
    fn from(err: std::io::Error) -> Self {
        ReadError::Io(err)
    }
}

impl std::error::Error for FormatError {
    fn description(&self) -> &str {
        match *self {
            #[cfg(feature = "flac")]
            FormatError::Flac(ref err) => std::error::Error::description(err),
            #[cfg(feature = "ogg_vorbis")]
            FormatError::OggVorbis(ref err) => std::error::Error::description(err),
            #[cfg(feature = "wav")]
            FormatError::Wav(ref err) => std::error::Error::description(err),
            #[cfg(feature = "caf")]
            FormatError::Caf(ref err) => std::error::Error::description(err),
            #[cfg(feature = "alac")]
            FormatError::Alac(_) => "Alac decode error",
        }
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            #[cfg(feature = "flac")]
            FormatError::Flac(ref err) => Some(err),
            #[cfg(feature = "ogg_vorbis")]
            FormatError::OggVorbis(ref err) => Some(err),
            #[cfg(feature = "wav")]
            FormatError::Wav(ref err) => Some(err),
            #[cfg(feature = "caf")]
            FormatError::Caf(ref err) => Some(err),
            #[cfg(feature = "alac")]
            FormatError::Alac(_) => None,
        }
    }
}

impl std::error::Error for ReadError {
    fn description(&self) -> &str {
        match *self {
            ReadError::Io(ref err) => std::error::Error::description(err),
            ReadError::Reader(ref err) => std::error::Error::description(err),
            ReadError::UnsupportedFormat => "no supported format was detected",
        }
    }
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match *self {
            ReadError::Io(ref err) => Some(err),
            ReadError::Reader(ref err) => Some(err),
            ReadError::UnsupportedFormat => None,
        }
    }
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            #[cfg(feature = "flac")]
            FormatError::Flac(ref err) => err.fmt(f),
            #[cfg(feature = "ogg_vorbis")]
            FormatError::OggVorbis(ref err) => err.fmt(f),
            #[cfg(feature = "wav")]
            FormatError::Wav(ref err) => err.fmt(f),
            #[cfg(feature = "caf")]
            FormatError::Caf(ref err) => err.fmt(f),
            #[cfg(feature = "alac")]
            FormatError::Alac(_) => write!(f, "{}", std::error::Error::description(self)),
        }
    }
}

impl std::fmt::Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match *self {
            ReadError::Io(ref err) => err.fmt(f),
            ReadError::Reader(ref err) => err.fmt(f),
            ReadError::UnsupportedFormat => write!(f, "{}", std::error::Error::description(self)),
        }
    }
}
