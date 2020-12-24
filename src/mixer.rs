use crate::{SoundDriver, SoundGenerator};

use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum PlaybackStyle {
    Once,
    Looped,
}

#[derive(Copy, Clone)]
struct SampleRateCorrection {
    progress_increment_amount: usize,
    ticks_pre_increment: usize,
}

#[derive(Clone)]
pub struct Sound {
    pub sample_rate: f32,
    pub channels: u16,
    pub samples: Vec<f32>,
    pub playback_style: PlaybackStyle,
}
impl Sound {
    fn get_sample_rate_correction(&self) -> SampleRateCorrection {
        let sample_rate = self.sample_rate as usize;
        let progress_increment_amount = if sample_rate > 44100 {
            sample_rate / 44100
        } else {
            1
        } * self.channels as usize;
        let ticks_pre_increment = if sample_rate >= 44100 {
            1
        } else {
            44100 / sample_rate
        } * 2;
        SampleRateCorrection {
            progress_increment_amount,
            ticks_pre_increment,
        }
    }
}

struct SoundInternal {
    data: Sound,
    progress: usize,
    volume: Volume,
    ear: EarState,
    sample_rate_correction: SampleRateCorrection,
    ticks: usize,
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct SoundId(usize);

#[derive(Clone, Copy, Debug)]
pub struct Volume(pub f32);

enum MixerMessage {
    Play(SoundId, Sound),
    PlayExt(SoundId, Sound, Volume),
    SetVolume(SoundId, Volume),
    SetVolumeSelf(Volume),
    Stop(SoundId),
}
struct MixerInternal {
    sample_rate: f32,
    sounds: HashMap<SoundId, SoundInternal>,
    dead_sounds: Vec<SoundId>,
    volume: Volume,
    ear: EarState,
}
#[derive(PartialEq, Clone, Copy)]
enum EarState {
    Left,
    Right,
}
impl EarState {
    fn switch(&mut self) {
        *self = match self {
            EarState::Left => EarState::Right,
            EarState::Right => EarState::Left,
        }
    }
}

pub struct SoundMixer {
    driver: SoundDriver<MixerMessage, SoundId>,
    uid: usize,
}

impl SoundMixer {
    pub fn new() -> SoundMixer {
        let mut driver = SoundDriver::new(Box::new(MixerInternal {
            sample_rate: 0.,
            sounds: HashMap::new(),
            dead_sounds: Vec::new(),
            volume: Volume(1.0),
            ear: EarState::Left,
        }));
        driver.start();
        SoundMixer { driver, uid: 0 }
    }

    pub fn new_ext(initial_volume: Volume) -> SoundMixer {
        let mut driver = SoundDriver::new(Box::new(MixerInternal {
            sample_rate: 0.,
            sounds: HashMap::new(),
            dead_sounds: Vec::new(),
            volume: initial_volume,
            ear: EarState::Left,
        }));
        driver.start();
        SoundMixer { driver, uid: 0 }
    }

    pub fn play(&mut self, sound: Sound) -> SoundId {
        let sound_id = SoundId(self.uid);
        self.uid += 1;

        self.driver.send_event(MixerMessage::Play(sound_id, sound));

        sound_id
    }

    pub fn play_ext(&mut self, sound: Sound, volume: Volume) -> SoundId {
        let sound_id = SoundId(self.uid);
        self.uid += 1;

        self.driver
            .send_event(MixerMessage::PlayExt(sound_id, sound, volume));

        sound_id
    }

    pub fn get_progress(&self, sound_id: SoundId) -> f32 { self.driver.get_sound_progress(sound_id) }

    pub fn set_volume(&mut self, sound_id: SoundId, volume: Volume) {
        self.driver
            .send_event(MixerMessage::SetVolume(sound_id, volume));
    }

    pub fn set_volume_self(&mut self, volume: Volume) {
        self.driver.send_event(MixerMessage::SetVolumeSelf(volume));
    }

    pub fn stop(&mut self, sound_id: SoundId) {
        self.driver.send_event(MixerMessage::Stop(sound_id));
    }

    pub fn frame(&mut self) {
        self.driver.frame();
    }
}

impl SoundGenerator<MixerMessage, SoundId> for MixerInternal {
    fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn handle_event(&mut self, evt: MixerMessage) {
        match evt {
            MixerMessage::Play(id, sound) => {
                let sample_rate_correction = sound.get_sample_rate_correction();
                self.sounds.insert(
                    id,
                    SoundInternal {
                        data: sound,
                        progress: 0,
                        volume: Volume(1.0),
                        ear: EarState::Left,
                        sample_rate_correction,
                        ticks: sample_rate_correction.ticks_pre_increment,
                    },
                );
            }
            MixerMessage::PlayExt(id, sound, volume) => {
                assert!(volume.0 <= 1.0);
                let sample_rate_correction = sound.get_sample_rate_correction();
                self.sounds.insert(
                    id,
                    SoundInternal {
                        data: sound,
                        progress: 0,
                        volume,
                        ear: EarState::Left,
                        sample_rate_correction,
                        ticks: sample_rate_correction.ticks_pre_increment,
                    },
                );
            }
            MixerMessage::SetVolume(id, volume) => {
                if let Some(sound) = self.sounds.get_mut(&id) {
                    assert!(volume.0 <= 1.0);
                    sound.volume = volume;
                }
            }
            MixerMessage::SetVolumeSelf(volume) => {
                self.volume = volume;
            }
            MixerMessage::Stop(id) => {
                self.sounds.remove(&id);
            }
        }
    }

    fn next_value(&mut self) -> f32 {
        let mut value = 0.;

        for (sound_id, mut sound) in &mut self.sounds {
            if self.ear != sound.ear {
                continue;
            }

            if sound.progress >= sound.data.samples.len() {
                match sound.data.playback_style {
                    PlaybackStyle::Once => {
                        self.dead_sounds.push(*sound_id);
                        continue;
                    }
                    PlaybackStyle::Looped => {
                        sound.progress = 0;
                    }
                }
            }

            let volume = sound.volume.0 * self.volume.0;
            // it's better to remap volume exponentially
            // so user hears difference instantly
            let volume = volume * volume;

            let next_index = match sound.data.channels {
                1 => sound.progress,
                2 => match sound.ear {
                    EarState::Left => sound.progress,
                    EarState::Right => sound.progress + 1,
                },
                _ => unreachable!(),
            };
            sound.ticks -= 1;

            value += sound.data.samples[next_index] * volume;
            if sound.ticks == 0 {
                sound.progress += sound.sample_rate_correction.progress_increment_amount;
                sound.ticks = sound.sample_rate_correction.ticks_pre_increment;
            }
            sound.ear.switch();
        }

        for sound_id in self.dead_sounds.iter() {
            self.sounds.remove(sound_id);
        }
        self.dead_sounds.clear();

        self.ear.switch();

        value
    }

    fn get_sound_progress(&self, sound_id: SoundId) -> f32 {
        if let Some(sound) = self.sounds.get(&sound_id) {
            return sound.progress as f32 / sound.data.samples.len() as f32;
        }

        0.0
    }

    fn has_sound(&self, sound_id: SoundId) -> bool {
        self.sounds.contains_key(&sound_id)
    }
}
