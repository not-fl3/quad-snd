use macroquad::{
    clear_background, draw_window,
    megaui::{widgets, Vector2},
    Vec2, WHITE,
};

use quad_snd::mixer::Volume;
use quad_snd::{
    decoder::{read_ogg, read_wav_ext},
    mixer::{PlaybackStyle, SoundMixer},
};

const WAV_BYTES: &'static [u8] = include_bytes!("test.wav");
const OGG_BYTES: &'static [u8] = include_bytes!("test.ogg");

const VOLUMES: &[f32] = &[0.8, 0.3, 0.9, 1.0, 0.1, 0.4, 0.6, 0.5, 0.7];

#[macroquad::main("Mixer")]
async fn main() {
    let wav_sound = read_wav_ext(WAV_BYTES, PlaybackStyle::Looped).unwrap();
    let ogg_sound = read_ogg(OGG_BYTES).unwrap();

    let mut mixer = SoundMixer::new();

    let mut sound_ids = Vec::new();
    let mut next_random_volume_id = 0;

    loop {
        clear_background(WHITE);

        draw_window(0, Vec2::new(20., 20.), Vec2::new(140., 155.), None, |ui| {
            if widgets::Button::new("MAGIC 1")
                .position(Vector2::new(5., 20.))
                .size(Vector2::new(130., 17.))
                .ui(ui)
            {
                sound_ids.push(mixer.play_ext(wav_sound.clone(), Volume(0.8)));
            }

            if widgets::Button::new("MAGIC 2")
                .position(Vector2::new(5., 50.))
                .size(Vector2::new(130., 17.))
                .ui(ui)
            {
                sound_ids.push(mixer.play(ogg_sound.clone()));
            }

            if widgets::Button::new("RANDOMIZE_VOLUMES")
                .position(Vector2::new(5., 80.))
                .size(Vector2::new(130., 17.))
                .ui(ui)
            {
                let new_volume = VOLUMES[next_random_volume_id];
                next_random_volume_id = (next_random_volume_id + 1) % VOLUMES.len();
                mixer.set_volume_self(Volume(new_volume));
            }

            if widgets::Button::new("STOP")
                .position(Vector2::new(5., 110.))
                .size(Vector2::new(130., 17.))
                .ui(ui)
            {
                while let Some(id) = sound_ids.pop() {
                    mixer.stop(id);
                }
            }

            if widgets::Button::new("PROGRESS")
                .position(Vector2::new(5., 140.))
                .size(Vector2::new(130., 17.))
                .ui(ui)
            {
                for id in &sound_ids {
                    println!("Progress for {:?}: {:?}", id, mixer.get_progress(id.clone()));
                }
            }
        });

        mixer.frame();

        macroquad::next_frame().await;
    }
}
