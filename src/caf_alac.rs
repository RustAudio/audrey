use super::read::FormatError;
use alac::Decoder;
use caf::chunks::CafChunk;
use caf::{CafPacketReader, ChunkType, FormatType};
use std::io::{Read, Seek};

pub struct AlacReader<T>
where
    T: Read + Seek,
{
    pub caf_reader: CafPacketReader<T>,
    pub alac_decoder: Decoder,
}

impl<T> AlacReader<T>
where
    T: Read + Seek,
{
    /// Creates a new AlacReader
    ///
    /// Returns Err(..) on IO errors, or if the stream is not CAF.
    /// Returns Ok(Some(..)) if the format inside is ALAC,
    /// None if its not ALAC.
    pub fn new(rdr: T) -> Result<Option<Self>, FormatError> {
        let caf_reader = r#try!(CafPacketReader::new(rdr, vec![ChunkType::MagicCookie]));
        if caf_reader.audio_desc.format_id != FormatType::AppleLossless {
            return Ok(None);
        }
        let cookie = caf_reader
            .chunks
            .iter()
            .filter_map(|c| match c {
                CafChunk::MagicCookie(ref d) => Some(d.clone()),
                _ => None,
            })
            .next()
            .unwrap();
        let decoder = r#try!(Decoder::from_cookie(&cookie).map_err(|_| FormatError::Alac(())));
        Ok(Some(AlacReader {
            caf_reader,
            alac_decoder: decoder,
        }))
    }
    pub fn read_packet(&mut self) -> Result<Option<Vec<i32>>, FormatError> {
        let mut output_buf: Vec<i32> = vec![
            0;
            (self.caf_reader.audio_desc.frames_per_packet
                * self.caf_reader.audio_desc.channels_per_frame)
                as usize
        ];
        let packet = match r#try!(self.caf_reader.next_packet()) {
            Some(pck) => pck,
            None => return Ok(None),
        };
        r#try!(self
            .alac_decoder
            .decode_packet(&packet, &mut output_buf)
            .map_err(|_| FormatError::Alac(())));
        Ok(Some(output_buf))
    }
}
