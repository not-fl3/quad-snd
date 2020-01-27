use super::{SoundError, SoundGenerator};

extern "C" {
    fn audio_init(buffer_size: u32) -> bool;
    fn audio_current_time() -> f64;
    fn audio_samples(buffer: *mut f32, audio_start: f64) -> f64;
    fn audio_sample_rate() -> f64;
    fn audio_pause_state() -> f64;
}

pub struct SoundDriver<T> {
    generator: Option<Box<dyn SoundGenerator<T>>>,
    start_audio: f64,
    buffer: [f32; BUFFER_SIZE as usize * 2],
    err: SoundError,
}

const BUFFER_SIZE: u32 = 2048;
const AUDIO_LATENCY: f64 = 0.1;

enum GameStatus {
    Running,
    Paused,
    Resumed(f64),
}

impl<T> SoundDriver<T> {
    pub fn get_error(&self) -> SoundError {
        self.err
    }

    pub fn new(generator: Box<dyn SoundGenerator<T>>) -> Self {
        let success = unsafe { audio_init(BUFFER_SIZE) };

        let err = if success == false {
            SoundError::NoDevice
        } else {
            SoundError::NoError
        };
        Self {
            generator: Some(generator),
            start_audio: 0.0,
            buffer: [0.0; BUFFER_SIZE as usize * 2],
            err,
        }
    }
    // -1 => game paused
    // >0 => pause duration
    fn get_pause_status(&self) -> GameStatus {
        let value: f64 = unsafe { audio_pause_state() };

        if value == 0.0 {
            GameStatus::Running
        } else {
            if value == -1.0 {
                GameStatus::Paused
            } else {
                GameStatus::Resumed(value)
            }
        }
    }
    pub fn send_event(&mut self, event: T) {
        if let Some(ref mut gen) = self.generator {
            gen.handle_event(event);
        }
    }
    pub fn frame(&mut self) {
        match self.get_pause_status() {
            GameStatus::Paused => {
                return;
            }
            GameStatus::Resumed(duration) => self.start_audio += duration,
            GameStatus::Running => (),
        }
        let now: f64 = unsafe { audio_current_time() };
        let now_latency = now + AUDIO_LATENCY;
        if self.start_audio == 0.0 {
            self.start_audio = now_latency;
        }
        if now >= self.start_audio - AUDIO_LATENCY {
            if let Some(ref mut gen) = self.generator {
                for i in 0..BUFFER_SIZE as usize * 2 {
                    self.buffer[i] = gen.next_value();
                }
            }
            let buffer_ptr = self.buffer.as_mut_ptr();
            let start_audio = self.start_audio;
            let samples: f64 = unsafe { audio_samples(buffer_ptr, start_audio) };
            self.start_audio += samples;
        }
    }
    pub fn start(&mut self) {
        if let Some(ref mut gen) = self.generator {
            let sample_rate: f64 = unsafe { audio_sample_rate() };
            gen.init(sample_rate as f32);
        }
    }
}
