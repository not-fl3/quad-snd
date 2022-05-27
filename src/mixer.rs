use crate::PlaySoundParams;

use std::collections::HashMap;
use std::sync::mpsc;

enum AudioMessage {
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
}

impl SoundState {
    fn next_sample(&mut self, sound_data: &[[f32; 2]]) -> Option<[f32; 2]> {
        let mut sample = match sound_data.get(self.sample) {
            Some(sample) => {
                self.sample += 1;
                *sample
            }
            None if self.looped => {
                self.sample = 1;
                *sound_data.first()?
            }
            None => return None,
        };

        sample[0] *= self.volume;
        sample[1] *= self.volume;

        Some(sample)
    }
}

pub struct Mixer {
    rx: mpsc::Receiver<AudioMessage>,
    sounds: HashMap<usize, Vec<[f32; 2]>>,
    mixer_state: Vec<SoundState>,
}
pub struct MixerControl {
    tx: mpsc::Sender<AudioMessage>,
    id: usize,
}

impl MixerControl {
    pub fn load(&mut self, data: &[u8]) -> usize {
        let id = self.id;

        let samples = load_samples_from_file(data).unwrap();

        self.tx
            .send(crate::mixer::AudioMessage::AddSound(id, samples))
            .unwrap_or_else(|_| println!("Audio thread died"));
        self.id += 1;

        id
    }

    pub fn play(&mut self, id: usize, params: PlaySoundParams) {
        self.tx
            .send(AudioMessage::PlaySound(id, params.looped, params.volume))
            .unwrap_or_else(|_| println!("Audio thread died"));
    }

    pub fn stop(&mut self, id: usize) {
        self.tx
            .send(AudioMessage::StopSound(id))
            .unwrap_or_else(|_| println!("Audio thread died"));
    }

    pub fn set_volume(&mut self, id: usize, volume: f32) {
        self.tx
            .send(AudioMessage::SetVolume(id, volume))
            .unwrap_or_else(|_| println!("Audio thread died"));
    }
}

impl Mixer {
    pub fn new() -> (Mixer, MixerControl) {
        let (tx, rx) = mpsc::channel();

        (
            Mixer {
                rx,
                sounds: HashMap::new(),
                mixer_state: vec![],
            },
            MixerControl { tx, id: 0 },
        )
    }

    pub fn fill_audio_buffer(&mut self, buffer: &mut [f32], frames: usize) {
        while let Ok(message) = self.rx.try_recv() {
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

        // zeroize the buffer
        buffer.fill(0.0);

        let buffer = {
            assert!(buffer.len() >= frames * 2);

            let ptr = buffer.as_mut_ptr() as *mut [f32; 2];

            unsafe { std::slice::from_raw_parts_mut(ptr, frames) }
        };

        // Note: Doing manual iteration to facilitate backtrack
        let mut i = 0;

        while let Some(sound) = self.mixer_state.get_mut(i) {
            let sound_data = &self.sounds[&sound.id][..];

            i += 1;

            for value in buffer.iter_mut() {
                match sound.next_sample(sound_data) {
                    Some(sample) => {
                        value[0] += sample[0];
                        value[1] += sample[1];
                    }
                    None => {
                        // Decrement the count to remove current sound
                        // and continue at sound swapped in
                        i -= 1;

                        self.mixer_state.swap_remove(i);
                        break;
                    }
                }
            }
        }
    }
}

/// Parse ogg/wav/etc and get  resampled to 44100, 2 channel data
pub fn load_samples_from_file(bytes: &[u8]) -> Result<Vec<[f32; 2]>, ()> {
    let mut audio_stream = {
        let file = std::io::Cursor::new(bytes);
        audrey::Reader::new(file).unwrap()
    };

    let description = audio_stream.description();
    let channels_count = description.channel_count();
    assert!(channels_count == 1 || channels_count == 2);

    let mut frames: Vec<[f32; 2]> = vec![];
    let mut samples_iterator = audio_stream
        .samples::<f32>()
        .map(std::result::Result::unwrap);

    // audrey's frame docs: "TODO: Should consider changing this behaviour to check the audio file's actual number of channels and automatically convert to F's number of channels while reading".
    // lets fix this TODO here
    loop {
        if channels_count == 1 {
            if let Some(sample) = samples_iterator.next() {
                frames.push([sample, sample]);
            } else {
                break;
            };
        }

        if channels_count == 2 {
            if let (Some(sample_left), Some(sample_right)) =
                (samples_iterator.next(), samples_iterator.next())
            {
                frames.push([sample_left, sample_right]);
            } else {
                break;
            };
        }
    }

    let sample_rate = description.sample_rate();

    // stupid nearest-neighbor resampler
    if sample_rate != 44100 {
        let new_length = ((44100 as f32 / sample_rate as f32) * frames.len() as f32) as usize;

        let mut resampled = vec![[0.0; 2]; new_length];

        for (n, i) in resampled.iter_mut().enumerate() {
            let ix = ((n as f32 / new_length as f32) * frames.len() as f32) as usize;
            *i = frames[ix];
        }
        return Ok(resampled);
    }

    Ok(frames)
}
