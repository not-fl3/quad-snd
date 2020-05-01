use macroquad::{
    clear_background, draw_window,
    megaui::{widgets, Vector2},
    Vec2, WHITE,
};

use quad_snd::{
    decoder::{read_ogg, read_wav},
    mixer::SoundMixer,
};

const WAV_BYTES: &'static [u8] = include_bytes!("test.wav");
const OGG_BYTES: &'static [u8] = include_bytes!("test.ogg");

#[macroquad::main("Mixer")]
async fn main() {
    let wav_sound = read_wav(WAV_BYTES).unwrap();
    let ogg_sound = read_ogg(OGG_BYTES).unwrap();

    let mut mixer = SoundMixer::new();

    loop {
        clear_background(WHITE);

        draw_window(0, Vec2::new(20., 20.), Vec2::new(110., 125.), None, |ui| {
            if widgets::Button::new("MAGIC 1")
                .position(Vector2::new(5., 20.))
                .size(Vector2::new(100., 17.))
                .ui(ui)
            {
                mixer.play(wav_sound.clone());
            }

            if widgets::Button::new("MAGIC 2")
                .position(Vector2::new(5., 50.))
                .size(Vector2::new(100., 17.))
                .ui(ui)
            {
                mixer.play(ogg_sound.clone());
            }
        });

        mixer.frame();

        macroquad::next_frame().await;
    }
}
