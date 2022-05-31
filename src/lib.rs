//! Loading and playing sounds.

#![allow(warnings)]

mod error;

pub use error::Error;

#[cfg(target_os = "android")]
#[path = "opensles_snd.rs"]
mod snd;

#[cfg(any(target_os = "linux", target_os = "dragonfly", target_os = "freebsd"))]
#[path = "alsa_snd.rs"]
mod snd;

#[cfg(any(target_os = "macos", target_os = "ios"))]
#[path = "coreaudio_snd.rs"]
mod snd;

#[cfg(target_os = "windows")]
#[path = "wasapi_snd.rs"]
mod snd;

#[cfg(target_arch = "wasm32")]
#[path = "web_snd.rs"]
mod snd;

#[cfg(not(target_arch = "wasm32"))]
mod mixer;

pub use snd::{AudioContext, Playback, Sound};

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
