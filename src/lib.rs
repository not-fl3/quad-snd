#[cfg(target_arch = "wasm32")]
#[path = "web_snd.rs"]
pub mod snd;

#[cfg(not(target_arch = "wasm32"))]
extern crate cpal;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native_snd.rs"]
pub mod snd;

pub mod decoder;
pub mod mixer;

pub use self::snd::*;

#[derive(Debug, Clone, Copy)]
/// error produced when creating the [`SoundDriver`]
pub enum SoundError {
    /// sound initialization was a success
    NoError,
    /// no sound device was found
    NoDevice,
    /// could not create an output stream
    OutputStream,
    /// unsupported output stream format
    UnknownStreamFormat,
}

/// You must provide a struct implementing this trait to the driver.
///
/// This is what generates the samples to be send to the audio output.
pub trait SoundGenerator<T>: Send {
    /// the sound driver calls this function during initialization to provide the audio interface sample rate.
    fn init(&mut self, sample_rate: f32);
    /// Because the sound generator runs in a separate thread on native target,
    /// you can only communicate with it through events using [`SoundDriver::send_event`].
    /// This is where you should handle those events.
    fn handle_event(&mut self, evt: T);
    /// This is the function generating the samples.
    /// Remember this is stereo output, you have to generate samples alternatively for the left and right channels.
    /// Sample values should be between -1.0 and 1.0.
    fn next_value(&mut self) -> f32;
}
