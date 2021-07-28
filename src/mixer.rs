use crate::PlaySoundParams;

use std::collections::HashMap;
use std::sync::mpsc;

pub enum AudioMessage {
    AddSound(usize, Vec<[f32; 2]>),
    PlaySound(usize, bool, f32),
    SetVolume(usize, f32),
    StopSound(usize),
}

#[derive(Debug)]
pub struct SoundState {
    id: usize,
    sample: usize,
    looped: bool,
    volume: f32,
    dead: bool,
}

pub struct Mixer {
    pub rx: mpsc::Receiver<AudioMessage>,
    sounds: HashMap<usize, Vec<[f32; 2]>>,
    mixer_state: Vec<SoundState>,
}

impl Mixer {
    pub fn new(rx: mpsc::Receiver<AudioMessage>) -> Mixer {
        Mixer {
            rx,
            sounds: HashMap::new(),
            mixer_state: vec![],
        }
    }

    pub fn fill_audio_buffer(&mut self, buffer: &mut [f32], frames: usize) {
        let num_channels = 2;

        if let Ok(message) = self.rx.try_recv() {
            match message {
                AudioMessage::AddSound(id, data) => {
                    self.sounds.insert(id, data);
                }
                AudioMessage::PlaySound(id, looped, volume) => {
                    // this is not really correct, but mirrors how it works on wasm/pc
                    if let Some(old) = self.mixer_state.iter().position(|s| s.id == id) {
                        self.mixer_state.swap_remove(old);
                    }
                    self.mixer_state.push(SoundState {
                        id,
                        sample: 0,
                        looped,
                        volume,
                        dead: false,
                    });
                }
                AudioMessage::SetVolume(id, volume) => {
                    if let Some(old) = self.mixer_state.iter_mut().find(|s| s.id == id) {
                        old.volume = volume;
                    }
                }
                AudioMessage::StopSound(id) => {
                    if let Some(old) = self.mixer_state.iter().position(|s| s.id == id) {
                        self.mixer_state.swap_remove(old);
                    }
                }
            }
        }

        for dt in 0..frames as usize {
            let mut value = [0.0, 0.0];

            for sound in &mut self.mixer_state {
                let sound_data = &self.sounds[&sound.id];

                value[0] += sound_data[sound.sample][0] * sound.volume;
                value[1] += sound_data[sound.sample][1] * sound.volume;
                sound.sample = sound.sample + 1;

                if sound.looped {
                    sound.sample = sound.sample % sound_data.len();
                } else if sound.sample >= sound_data.len() {
                    sound.dead = true;
                }
            }
            self.mixer_state.retain(|s| s.dead == false);

            buffer[num_channels * dt as usize] = value[0];
            buffer[num_channels * dt as usize + 1] = value[1];
        }
    }
}
