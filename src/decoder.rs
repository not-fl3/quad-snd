use crate::mixer::{PlaybackStyle, Sound};

#[derive(Debug)]
pub enum Error {
    HoundError(hound::Error),
    LewtonError(lewton::VorbisError),
    ReadError(String),
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

    if spec.sample_rate % 11025 != 0 {
        return Err(Error::ReadError(String::from(format!(
            "Sample rate {} is unsupported.",
            spec.sample_rate
        ))));
    }

    if spec.bits_per_sample != 16 {
        return Err(Error::ReadError(String::from(format!(
            "Bits per sample {} is unsupported.",
            spec.bits_per_sample
        ))));
    }

    if spec.sample_format != hound::SampleFormat::Int {
        return Err(Error::ReadError(String::from(
            "Sample format is unsupported.",
        )));
    }

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
    read_wav(bytes).map(|sound| Sound {
        playback_style,
        ..sound
    })
}

pub fn read_ogg(data: &[u8]) -> Result<Sound, Error> {
    use lewton::inside_ogg::*;
    use std::io::Cursor;

    let mut reader = OggStreamReader::new(Cursor::new(data))?;
    let sample_rate = reader.ident_hdr.audio_sample_rate as i32;
    let channels = reader.ident_hdr.audio_channels;

    if sample_rate % 11025 != 0 {
        return Err(Error::ReadError(String::from(format!(
            "Sample rate {} is unsupported.",
            sample_rate
        ))));
    }

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
    read_ogg(data).map(|sound| Sound {
        playback_style,
        ..sound
    })
}
