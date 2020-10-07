use crate::mixer::{Sound, PlaybackStyle};

#[derive(Debug)]
pub enum Error {
    HoundError(hound::Error),
    LewtonError(lewton::VorbisError),
}

impl std::convert::From<hound::Error> for Error {
    fn from(error: hound::Error) -> Error {
        Error::HoundError(error)
    }
}

impl std::convert::From<lewton::VorbisError> for Error {
    fn from(error: lewton::VorbisError) -> Error {
        Error::LewtonError(error)
    }
}

pub fn read_wav(bytes: &[u8]) -> Result<Sound, Error> {
    let reader = hound::WavReader::new(bytes)?;
    let spec = reader.spec();

    assert_eq!(spec.sample_rate % 11025, 0, "format unsupported");
    assert_eq!(spec.bits_per_sample, 16, "format unsupported");
    assert_eq!(
        spec.sample_format,
        hound::SampleFormat::Int,
        "format unsupported"
    );

    let samples = reader
        .into_samples::<i16>()
        .map(|sample| sample.unwrap() as f32 / std::i16::MAX as f32)
        .collect::<Vec<f32>>();

    Ok(Sound {
        sample_rate: spec.sample_rate as f32,
        channels: spec.channels,
        samples,
        playback_style: PlaybackStyle::Once,
    })
}

pub fn read_wav_ext(bytes: &[u8], playback_style: PlaybackStyle) -> Result<Sound, Error> {
    read_wav(bytes).map(|sound| Sound { playback_style, ..sound })
}

pub fn read_ogg(data: &[u8]) -> Result<Sound, Error> {
    use lewton::inside_ogg::*;
    use std::io::Cursor;

    let mut reader = OggStreamReader::new(Cursor::new(data))?;
    let sample_rate = reader.ident_hdr.audio_sample_rate as i32;
    let channels = reader.ident_hdr.audio_channels;

    assert_eq!(sample_rate % 11025, 0, "format unsupported");

    let mut samples: Vec<f32> = vec![];

    while let Ok(Some(data)) = reader.read_dec_packet_itl() {
        for sample in data {
            samples.push(sample as f32 / std::i16::MAX as f32);
        }
    }

    Ok(Sound {
        sample_rate: sample_rate as _,
        channels: channels as _,
        samples,
        playback_style: PlaybackStyle::Once,
    })
}

pub fn read_ogg_ext(data: &[u8], playback_style: PlaybackStyle) -> Result<Sound, Error> {
    read_ogg(data).map(|sound| Sound {playback_style, ..sound})
}
