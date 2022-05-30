use crate::PlaySoundParams;

extern "C" {
    fn audio_init();
    fn audio_add_buffer(content: *const u8, content_len: u32) -> u32;
    fn audio_play_buffer(buffer: u32, volume_l: f32, volume_r: f32, speed: f32, repeat: bool);
    fn audio_source_is_loaded(buffer: u32) -> bool;
    fn audio_source_set_volume(buffer: u32, volume_l: f32, volume_r: f32);
    fn audio_source_stop(buffer: u32);
}

#[no_mangle]
pub extern "C" fn macroquad_audio_crate_version() -> u32 {
    let major = 0;
    let minor = 1;
    let patch = 0;

    (major << 24) + (minor << 16) + patch
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
    pub fn stop(self, _ctx: &mut AudioContext) {
        // ctx.mixer_ctrl.send(AudioMessage::Stop(self.play_id));
    }

    pub fn set_volume(&mut self, _ctx: &mut AudioContext, volume: f32) {
        // ctx.mixer_ctrl.send(AudioMessage::SetVolume(self.play_id, volume));
    }
}

impl Sound {
    pub fn load(_ctx: &mut AudioContext, data: &[u8]) -> Sound {
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

    pub fn play(&mut self, _ctx: &mut AudioContext, params: PlaySoundParams) -> Playback {
        unsafe { audio_play_buffer(self.0, params.volume, params.volume, 1.0, params.looped) };

        Playback(0)
    }

    pub fn stop(&mut self, _ctx: &mut AudioContext) {
        unsafe { audio_source_stop(self.0) }
    }

    pub fn set_volume(&mut self, _ctx: &mut AudioContext, volume: f32) {
        unsafe { audio_source_set_volume(self.0, volume, volume) }
    }
}
