use macroquad::{
    clear_background, draw_window,
    megaui::{widgets, Vector2},
    Vec2, WHITE,
};

use quad_snd::{
    decoder::{read_ogg, read_wav},
    mixer::SoundMixer,
};
use quad_snd::decoder::read_wav_ext;
use quad_snd::mixer::PlaybackStyle;

const WAV_BYTES: &'static [u8] = include_bytes!("test.wav");
const OGG_BYTES: &'static [u8] = include_bytes!("test.ogg");

#[macroquad::main("Mixer")]
async fn main() {
    let wav_sound = read_wav_ext(WAV_BYTES, PlaybackStyle::Looped).unwrap();
    let ogg_sound = read_ogg(OGG_BYTES).unwrap();

    let mut mixer = SoundMixer::new();

    let mut sound_ids = Vec::new();

    loop {
        clear_background(WHITE);

        draw_window(0, Vec2::new(20., 20.), Vec2::new(110., 125.), None, |ui| {
            if widgets::Button::new("MAGIC 1")
                .position(Vector2::new(5., 20.))
                .size(Vector2::new(100., 17.))
                .ui(ui)
            {
                sound_ids.push(mixer.play(wav_sound.clone()));
            }

            if widgets::Button::new("MAGIC 2")
                .position(Vector2::new(5., 50.))
                .size(Vector2::new(100., 17.))
                .ui(ui)
            {
                sound_ids.push(mixer.play(ogg_sound.clone()));
            }

            if widgets::Button::new("STOP")
                .position(Vector2::new(5., 80.))
                .size(Vector2::new(100., 17.))
                .ui(ui)
            {
                while let Some(id) = sound_ids.pop() {
                    mixer.stop(id);
                }
            }
        });

        mixer.frame();

        macroquad::next_frame().await;
    }
}
