use crate::PlaySoundParams;

extern "C" {
    fn audio_init();
    fn audio_add_buffer(content: *const u8, content_len: u32) -> u32;
    fn audio_play_buffer(buffer: u32, volume: f32, pitch: f32, repeat: bool) -> u32;
    fn audio_source_is_loaded(buffer: u32) -> bool;
    fn audio_source_set_volume(buffer: u32, volume: f32);
    fn audio_source_stop(buffer: u32);
    fn audio_source_delete(buffer: u32);
    fn audio_playback_stop(playback: u32);
    fn audio_playback_set_volume(playback: u32, volume: f32);
    fn audio_playback_set_pitch(playback: u32, pitch: f32);
}

#[no_mangle]
pub extern "C" fn macroquad_audio_crate_version() -> u32 {
    1
}

pub struct AudioContext;

impl AudioContext {
    pub fn new() -> AudioContext {
        unsafe {
            audio_init();
        }

        AudioContext
    }
}

pub struct Sound(u32);

pub struct Playback(u32);

impl Playback {
    pub fn stop(self, _ctx: &AudioContext) {
        unsafe { audio_playback_stop(self.0) }
    }

    pub fn set_volume(&self, _ctx: &AudioContext, volume: f32) {
        unsafe { audio_playback_set_volume(self.0, volume) }
    }

    pub fn set_pitch(&self, _ctx: &AudioContext, pitch: f32) {
        unsafe { audio_playback_set_pitch(self.0, pitch) }
    }
}

impl Sound {
    pub fn load(_ctx: &AudioContext, data: &[u8]) -> Sound {
        let buffer = unsafe { audio_add_buffer(data.as_ptr(), data.len() as u32) };
        Sound(buffer)
    }

    /// WASM requirement - sound may be used only after it is is_loaded
    /// something like will do:
    ///```skip
    /// let sound = Sound::load(&mut ctx, include_bytes!("test.wav"));
    /// while sound.is_lodead() == false {
    ///     next_frame().await;
    /// }
    /// sound.play(ctx, Default::default());
    ///```
    pub fn is_loaded(&self) -> bool {
        unsafe { audio_source_is_loaded(self.0) }
    }

    pub fn play(&self, _ctx: &AudioContext, params: PlaySoundParams) -> Playback {
        let id = unsafe { audio_play_buffer(self.0, params.volume, params.pitch, params.looped) };

        Playback(id)
    }

    pub fn stop(&self, _ctx: &AudioContext) {
        unsafe { audio_source_stop(self.0) }
    }

    pub fn set_volume(&self, _ctx: &AudioContext, volume: f32) {
        unsafe { audio_source_set_volume(self.0, volume) }
    }

    pub fn delete(&self, _ctx: &AudioContext) {
        unsafe { audio_source_delete(self.0) }
    }
}
