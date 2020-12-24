use macroquad::*;

use macroquad::{
    hash,
    megaui::{widgets, Vector2},
};

struct Oscillator {
    sample_rate: f32,
    freq: f32,
    volume: f32,
    t: f32,
    left: bool,
}

impl quad_snd::SoundGenerator<f32, ()> for Oscillator {
    fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn handle_event(&mut self, evt: f32) {
        self.freq = evt;
        self.volume = 1.0;
    }

    fn next_value(&mut self) -> f32 {
        self.left = !self.left;
        // stereo output. update only every two samples
        if self.left {
            // 440 Hz sin oscillator
            self.t += self.freq / self.sample_rate;
        }
        if self.volume > 0. {
            self.volume -= 0.00005;
        } else {
            self.volume = 0.0;
        }
        (self.t * 3.14159 * 2.0).sin() * self.volume
    }
}

#[macroquad::main("Piano")]
async fn main() {
    let mut snd = quad_snd::SoundDriver::new(Box::new(Oscillator {
        sample_rate: 0.0,
        t: 0.0,
        freq: 440.0,
        volume: 0.0,
        left: false,
    }));

    snd.start();

    loop {
        snd.frame();

        clear_background(WHITE);

        draw_window(
            hash!(),
            Vec2::new(20., 20.),
            Vec2::new(700., 200.),
            None,
            |ui| {
                for i in 0..15 {
                    let octave = 40; // I HAVE NO IDEA

                    if widgets::Button::new(format!("{}", i))
                        .position(Vector2::new(i as f32 * 45. + 10., 40.))
                        .size(Vector2::new(40., 70.))
                        .ui(ui)
                    {
                        let freq = 2f32.powf((i as f32 + octave as f32 - 49.) / 12.) * 440.;
                        snd.send_event(freq);
                    }
                }
            },
        );

        next_frame().await
    }
}
