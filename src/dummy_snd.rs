use crate::PlaySoundParams;

pub struct AudioContext;

impl AudioContext {
    pub fn new() -> AudioContext {
        AudioContext
    }
}

pub struct Playback;

impl Playback {
    pub fn stop(self, _ctx: &mut AudioContext) {}

    pub fn set_volume(&mut self, _ctx: &mut AudioContext) {}
}

pub struct Sound;

impl Sound {
    pub fn load(_data: &[u8]) -> Sound {
        Sound
    }

    pub fn play(&mut self, _ctx: &mut AudioContext, _params: PlaySoundParams) -> Playback {
        Playback
    }

    pub fn stop(&mut self, _ctx: &mut AudioContext) {}

    pub fn set_volume(&mut self, _ctx: &mut AudioContext, _volume: f32) {}
}
