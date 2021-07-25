//! Loading and playing sounds.

#![allow(warnings)]

mod error;

pub use error::Error;

#[cfg(feature = "dummy_snd")]
#[path = "dummy_snd.rs"]
mod snd;

#[cfg(target_os = "android")]
#[cfg(not(feature = "dummy_snd"))]
#[path = "opensles_snd.rs"]
mod snd;

#[cfg(target_os = "linux")]
#[cfg(not(feature = "dummy_snd"))]
#[path = "alsa_snd.rs"]
mod snd;

#[cfg(target_arch = "wasm32")]
#[cfg(not(feature = "dummy_snd"))]
#[path = "web_snd.rs"]
mod snd;

mod mixer;

pub use snd::{AudioContext, Sound};

pub struct PlaySoundParams {
    pub looped: bool,
    pub volume: f32,
}

impl Default for PlaySoundParams {
    fn default() -> PlaySoundParams {
        PlaySoundParams {
            looped: false,
            volume: 1.,
        }
    }
}

