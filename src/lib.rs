pub extern crate sample;

#[cfg(feature = "alac")]
pub extern crate alac;
#[cfg(feature = "caf")]
pub extern crate caf;
#[cfg(feature = "flac")]
pub extern crate claxon; // flac
#[cfg(feature = "wav")]
pub extern crate hound; // wav
#[cfg(feature = "ogg_vorbis")]
pub extern crate lewton; // ogg vorbis

#[cfg(feature = "caf_alac")]
mod caf_alac;

pub mod read;
pub mod write;

pub use read::{open, Reader};

/// Enumerates the various formats supported by the crate.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum Format {
    #[cfg(feature = "flac")]
    Flac,
    #[cfg(feature = "ogg_vorbis")]
    OggVorbis,
    #[cfg(feature = "wav")]
    Wav,
    #[cfg(feature = "caf_alac")]
    CafAlac,
}

impl Format {
    /// Read a `Format` from the given `extension`.
    ///
    /// This function expects that the `extension` is lowercase ASCII, e.g "wav" or "ogg".
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension {
            #[cfg(feature = "flac")]
            "flac" => Some(Format::Flac),
            #[cfg(feature = "ogg_vorbis")]
            "ogg" | "oga" => Some(Format::OggVorbis),
            #[cfg(feature = "wav")]
            "wav" | "wave" => Some(Format::Wav),
            #[cfg(feature = "caf")]
            "caf" => Some(Format::CafAlac),
            _ => None,
        }
    }

    /// Return the most commonly used file extension associated with the `Format`.
    pub fn extension(self) -> &'static str {
        match self {
            #[cfg(feature = "flac")]
            Format::Flac => "flac",
            #[cfg(feature = "wav")]
            Format::Wav => "wav",
            #[cfg(feature = "ogg_vorbis")]
            Format::OggVorbis => "ogg",
            #[cfg(feature = "caf_alac")]
            Format::CafAlac => "caf",
        }
    }
}
