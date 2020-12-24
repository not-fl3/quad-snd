use quad_snd::{SoundDriver, SoundGenerator};

struct Oscillator {
    sample_rate: f32,
    t: f32,
    left: bool,
}

impl SoundGenerator<u8, ()> for Oscillator {
    fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn handle_event(&mut self, _evt: u8) {}

    fn next_value(&mut self) -> f32 {
        self.left = !self.left;
        // stereo output. update only every two samples
        if self.left {
            // 440 Hz sin oscillator
            self.t += 440.0 / self.sample_rate;
        }
        (self.t * std::f32::consts::PI * 2.0).sin()
    }
}

fn main() {
    let mut snd = SoundDriver::new(Box::new(Oscillator {
        sample_rate: 0.0,
        t: 0.0,
        left: false,
    }));
    snd.start();
    loop {
        snd.frame();
    }
}
