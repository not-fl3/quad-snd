use crate::PlaySoundParams;

use std::collections::HashMap;
use std::sync::mpsc;

pub enum AudioMessage {
    AddSound(usize, Vec<[f32; 2]>),
    PlaySound(usize, bool, f32),
    SetVolume(usize, f32),
    StopSound(usize),
}

pub enum ControlMessage {
    Pause,
    Resume,
}

pub struct SoundState {
    id: usize,
    sample: usize,
    looped: bool,
    volume: f32,
    dead: bool,
}

pub unsafe fn fill_audio_buffer(buffer: &mut [f32], frames: usize, mixer: &mut Mixer) {
    let num_channels = 2;

    if let Ok(message) = mixer.rx.try_recv() {
        match message {
            AudioMessage::AddSound(id, data) => {
                mixer.sounds.insert(id, data);
            }
            AudioMessage::PlaySound(id, looped, volume) => {
                // this is not really correct, but mirrors how it works on wasm/pc
                if let Some(old) = mixer.mixer_state.iter().position(|s| s.id == id) {
                    mixer.mixer_state.swap_remove(old);
                }
                mixer.mixer_state.push(SoundState {
                    id,
                    sample: 0,
                    looped,
                    volume,
                    dead: false,
                });
            }
            AudioMessage::SetVolume(id, volume) => {
                if let Some(old) = mixer.mixer_state.iter_mut().find(|s| s.id == id) {
                    old.volume = volume;
                }
            }
            AudioMessage::StopSound(id) => {
                if let Some(old) = mixer.mixer_state.iter().position(|s| s.id == id) {
                    mixer.mixer_state.swap_remove(old);
                }
            }
        }
    }

    for dt in 0..frames as usize {
        let mut value = [0.0, 0.0];

        for sound in &mut mixer.mixer_state {
            let sound_data = &mixer.sounds[&sound.id];

            value[0] += sound_data[sound.sample][0] * sound.volume;
            value[1] += sound_data[sound.sample][1] * sound.volume;
            sound.sample = sound.sample + 1;

            if sound.looped {
                sound.sample = sound.sample % sound_data.len();
            } else if sound.sample >= sound_data.len() {
                sound.dead = true;
            }
        }
        mixer.mixer_state.retain(|s| s.dead == false);

        buffer[num_channels * dt as usize] = value[0];
        buffer[num_channels * dt as usize + 1] = value[1];
    }
}

pub struct Mixer {
    pub rx: mpsc::Receiver<AudioMessage>,
    pub rx1: mpsc::Receiver<ControlMessage>,
    sounds: HashMap<usize, Vec<[f32; 2]>>,
    mixer_state: Vec<SoundState>,
}

impl Mixer {
    pub fn new(rx: mpsc::Receiver<AudioMessage>, rx1: mpsc::Receiver<ControlMessage>) -> Mixer {
        Mixer {
            rx,
            rx1,
            sounds: HashMap::new(),
            mixer_state: vec![],
        }
    }
}
