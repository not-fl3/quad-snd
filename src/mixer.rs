use crate::{SoundDriver, SoundGenerator};

use std::collections::HashMap;

#[derive(Clone, Copy, PartialEq)]
pub enum PlaybackStyle {
    Once,
    Looped
}

#[derive(Clone)]
pub struct Sound {
    pub sample_rate: f32,
    pub channels: u16,
    pub samples: Vec<f32>,
    pub playback_style: PlaybackStyle
}

struct SoundInternal {
    data: Sound,
    progress: usize,
    volume: f32
}

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct SoundId(usize);

enum MixerMessage {
    Play(SoundId, Sound),
    PlayExt(SoundId, Sound, f32),
    SetVolume(SoundId, f32),
    Stop(SoundId),
}
struct MixerInternal {
    sample_rate: f32,
    sounds: HashMap<SoundId, SoundInternal>,
}

pub struct SoundMixer {
    driver: SoundDriver<MixerMessage>,
    uid: usize,
}

impl SoundMixer {
    pub fn new() -> SoundMixer {
        let mut driver = SoundDriver::new(Box::new(MixerInternal {
            sample_rate: 0.,
            sounds: HashMap::new(),
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

    pub fn play_ext(&mut self, sound: Sound, volume: f32) -> SoundId {
        let sound_id = SoundId(self.uid);
        self.uid += 1;

        self.driver.send_event(MixerMessage::PlayExt(sound_id, sound, volume));

        sound_id
    }

    pub fn set_volume(&mut self, sound_id: SoundId, volume: f32) {
        self.driver.send_event(MixerMessage::SetVolume(sound_id, volume))
    }

    pub fn stop(&mut self, sound_id: SoundId) {
        self.driver.send_event(MixerMessage::Stop(sound_id))
    }

    pub fn frame(&mut self) {
        self.driver.frame();
    }
}

impl SoundGenerator<MixerMessage> for MixerInternal {
    fn init(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate;
    }

    fn handle_event(&mut self, evt: MixerMessage) {
        match evt {
            MixerMessage::Play(id, sound) => {
                self.sounds.insert(
                    id,
                    SoundInternal {
                        data: sound,
                        progress: 0,
                        volume: 1.0
                    },
                );
            },
            MixerMessage::PlayExt(id, sound, volume) => {
                assert!(volume <= 1.0);
                self.sounds.insert(
                    id,
                    SoundInternal {
                        data: sound,
                        progress: 0,
                        volume: volume * volume // it's better to remap volume exponentially so user hears difference more
                    },
                );
            },
            MixerMessage::SetVolume(id, volume) => {
                if let Some(sound) = self.sounds.get_mut(&id) {
                    assert!(volume <= 1.0);
                    sound.volume = volume * volume;
                }
            },
            MixerMessage::Stop(id) => {
                self.sounds.remove(&id);
            },
            _ => unreachable!() //it's nice to fail when we added some new event and didn't handle it properly here
        }
    }

    fn next_value(&mut self) -> f32 {
        let mut value = 0.;

        for (_, mut sound) in &mut self.sounds {
            if sound.progress >= sound.data.samples.len() {
                match sound.data.playback_style {
                    PlaybackStyle::Once => {
                        continue;
                    }
                    PlaybackStyle::Looped => {
                        sound.progress = 0;
                    }
                }
            }

            let divisor = match sound.data.channels {
                1 => 2,
                2 => 1,
                _ => panic!("unsupported format"),
            };

            value += sound.data.samples[sound.progress / divisor] * sound.volume;
            sound.progress += 1;
        }
        value
    }
}
