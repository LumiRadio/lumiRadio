use std::{io::Write, path::Path};

use audiotags::{AudioTag, AudioTagConfig, AudioTagEdit, AudioTagWrite, Id3v2Tag, ToAny, ToAnyTag};
use oggvorbismeta::{CommentHeader, VorbisComments};

pub struct OggInnerTag(CommentHeader);

impl OggInnerTag {
    pub fn new() -> Self {
        Self(CommentHeader::new())
    }
}

pub trait WavTag {
    fn read_from_wav_path(path: impl AsRef<Path>) -> Result<Self, id3::Error>
    where
        Self: Sized;
}

impl WavTag for Id3v2Tag {
    fn read_from_wav_path(path: impl AsRef<Path>) -> Result<Self, id3::Error>
    where
        Self: Sized,
    {
        let id_tag = id3::Tag::read_from_wav_path(path)?;

        Ok(id_tag.into())
    }
}
