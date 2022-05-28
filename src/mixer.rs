use crate::PlaySoundParams;

use std::collections::HashMap;
use std::sync::mpsc;

enum AudioMessage {
    AddSound(usize, Vec<f32>),
    PlaySound(usize, bool, f32),
    SetVolume(usize, f32),
    StopSound(usize),
}

#[derive(Debug)]
pub struct SoundState {
    id: usize,
    sample: usize,
    // Note on safety: this borrows a `Vec` from inside the `HashMap`.
    // Moving the `Vec` inside the `HashMap` doesn't affect pointer
    // safety here at all, but we have to make sure to remove this
    // `SoundState` before the `Vec` is removed in the future.
    data: *const [f32],
    looped: bool,
    volume: f32,
}

unsafe impl Send for SoundState {}

impl SoundState {
    fn get_samples(&mut self, n: usize) -> &[f32] {
        let data = unsafe { &*self.data };
        let data = &data[self.sample..];

        self.sample += n;

        match data.get(..n) {
            Some(data) => data,
            None => data,
        }
    }

    fn rewind(&mut self) {
        self.sample = 0;
    }
}

pub struct Mixer {
    rx: mpsc::Receiver<AudioMessage>,
    sounds: HashMap<usize, Vec<f32>>,
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
                    if let Some(old) = self.mixer_state.iter().position(|s| s.id == id) {
                        self.mixer_state.swap_remove(old);
                    }
                    if let Some(data) = self.sounds.get(&id) {
                        let data = &**data;

                        self.mixer_state.push(SoundState {
                            id,
                            sample: 0,
                            data,
                            looped,
                            volume,
                        });
                    }
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

        // Note: Doing manual iteration so we can remove sounds that finished playing
        let mut i = 0;

        while let Some(sound) = self.mixer_state.get_mut(i) {
            let volume = sound.volume;
            let mut remainder = buffer.len();

            loop {
                let samples = sound.get_samples(remainder);

                for (b, s) in buffer.iter_mut().zip(samples) {
                    *b += s * volume;
                }

                remainder -= samples.len();

                if remainder > 0 && sound.looped {
                    sound.rewind();
                    continue;
                }

                break;
            }

            if remainder > 0 {
                self.mixer_state.swap_remove(i);
            } else {
                i += 1;
            }
        }
    }
}

/// Parse ogg/wav/etc and get  resampled to 44100, 2 channel data
pub fn load_samples_from_file(bytes: &[u8]) -> Result<Vec<f32>, ()> {
    let mut audio_stream = {
        let file = std::io::Cursor::new(bytes);
        audrey::Reader::new(file).unwrap()
    };

    let description = audio_stream.description();
    let channels_count = description.channel_count();
    assert!(channels_count == 1 || channels_count == 2);

    let mut frames: Vec<f32> = Vec::with_capacity(4096);
    let mut samples_iterator = audio_stream
        .samples::<f32>()
        .map(std::result::Result::unwrap);

    // audrey's frame docs: "TODO: Should consider changing this behaviour to check the audio file's actual number of channels and automatically convert to F's number of channels while reading".
    // lets fix this TODO here
    if channels_count == 1 {
        frames.extend(samples_iterator.flat_map(|sample| [sample, sample]));
    } else if channels_count == 2 {
        frames.extend(samples_iterator);
    }

    let sample_rate = description.sample_rate();

    // stupid nearest-neighbor resampler
    if sample_rate != 44100 {
        let mut new_length = ((44100 as f32 / sample_rate as f32) * frames.len() as f32) as usize;

        // `new_length` must be an even number
        new_length -= new_length % 2;

        let mut resampled = vec![0.0; new_length];

        for (n, sample) in resampled.chunks_exact_mut(2).enumerate() {
            let ix = 2 * ((n as f32 / new_length as f32) * frames.len() as f32) as usize;
            sample[0] = frames[ix];
            sample[1] = frames[ix + 1];
        }
        return Ok(resampled);
    }

    Ok(frames)
}
