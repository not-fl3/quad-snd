use std;
use std::sync::mpsc::{channel, Sender};
use std::sync::{Arc, Mutex};
use std::thread;

use super::{SoundError, SoundGenerator};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};
use cpal::{SampleFormat, SampleRate};

/// This is the sound API that allows you to send events to your generator.
pub struct SoundDriver<T: Send + 'static, J: Send + 'static> {
    event_loop: Option<cpal::EventLoop>,
    format: Option<cpal::Format>,
    stream_id: Option<cpal::StreamId>,
    tx: Option<Sender<T>>,
    generator: Arc<Mutex<Box<dyn SoundGenerator<T, J>>>>,
    err: SoundError,
}

impl<T: Send + 'static, J: Send + 'static> SoundDriver<T, J> {
    /// After calling [`SoundDriver::new`], you can call this function to see if the audio initialization was a success.
    pub fn get_error(&self) -> SoundError {
        self.err
    }

    pub fn get_sound_progress(&self, id: J) -> f32 {
        let generator = &*(self.generator.lock().unwrap());

        generator.get_sound_progress(id)
    }

    pub fn has_sound(&self, id: J) -> bool {
        let generator = &*(self.generator.lock().unwrap());

        generator.has_sound(id)
    }

    /// Initialize the sound device and provide the generator to the driver.
    pub fn new(generator: Box<dyn SoundGenerator<T, J>>) -> Self {
        // Setup the audio system
        let host = cpal::default_host();
        let event_loop = host.event_loop();

        let device = match host.default_output_device() {
            Some(device) => device,
            None => {
                return Self {
                    event_loop: Some(event_loop),
                    format: None,
                    stream_id: None,
                    tx: None,
                    generator: Arc::new(Mutex::new(generator)),
                    err: SoundError::NoDevice,
                };
            }
        };

        let mut output_format = match device.default_output_format() {
            Ok(default_output_format) => default_output_format,
            Err(_err) => {
                return Self {
                    event_loop: Some(event_loop),
                    format: None,
                    stream_id: None,
                    tx: None,
                    generator: Arc::new(Mutex::new(generator)),
                    err: SoundError::UnknownStreamFormat,
                };
            }
        };

        match device.supported_output_formats() {
            Ok(available_formats) => {
                for available_format in available_formats {
                    if available_format.channels != 2 {
                        continue;
                    }
                    if available_format.data_type != SampleFormat::F32 {
                        continue;
                    }
                    if available_format.min_sample_rate.0 > 44100 {
                        continue;
                    }
                    if available_format.max_sample_rate.0 < 44100 {
                        continue;
                    }
                    output_format.channels = 2;
                    output_format.data_type = SampleFormat::F32;
                    output_format.sample_rate = SampleRate(44100);
                    break;
                }
            }
            Err(_err) => {}
        };

        let stream_id = match event_loop.build_output_stream(&device, &output_format) {
            Ok(output_stream) => output_stream,
            Err(_err) => {
                return Self {
                    event_loop: Some(event_loop),
                    format: Some(output_format),
                    stream_id: None,
                    tx: None,
                    generator: Arc::new(Mutex::new(generator)),
                    err: SoundError::OutputStream,
                };
            }
        };

        Self {
            event_loop: Some(event_loop),
            format: Some(output_format),
            stream_id: Some(stream_id),
            tx: None,
            generator: Arc::new(Mutex::new(generator)),
            err: SoundError::NoError,
        }
    }

    /// Send an event to the generator
    pub fn send_event(&mut self, event: T) {
        if let Some(ref mut tx) = self.tx {
            tx.send(event).unwrap();
        }
    }

    fn get_sample_rate(&self) -> f32 {
        if let Some(ref fmt) = self.format {
            fmt.sample_rate.0 as f32
        } else {
            1.0
        }
    }
    /// This function should be called every frame.
    /// It's only needed on web target to fill the output sound buffer.
    pub fn frame(&mut self) {}
    /// This will call the generator init function.
    /// On native target, it starts the sound thread and the audio loop.
    /// On web target, only the [`SoundDriver::frame`] function produces sound.
    pub fn start(&mut self) {
        let (tx, rx) = channel();
        self.tx = Some(tx);
        let stream_id = self.stream_id.take().unwrap();
        let sample_rate = self.get_sample_rate();

        let generator_clone = self.generator.clone();

        if let Some(evt) = self.event_loop.take() {
            evt.play_stream(stream_id).expect("could not play stream");

            thread::spawn(move || {
                {
                    let generator = &mut *(generator_clone.lock().unwrap());

                    generator.init(sample_rate);
                }

                evt.run(move |_stream_id, stream_result| {
                    let generator = &mut *(generator_clone.lock().unwrap());

                    for event in rx.try_iter() {
                        generator.handle_event(event);
                    }

                    let stream_data = match stream_result {
                        Ok(data) => data,
                        Err(_err) => {
                            return;
                        }
                    };

                    match stream_data {
                        cpal::StreamData::Output {
                            buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer),
                        } => {
                            for elem in buffer.iter_mut() {
                                *elem = ((generator.next_value() * 0.5 + 0.5)
                                    * std::u16::MAX as f32)
                                    as u16;
                            }
                        }
                        cpal::StreamData::Output {
                            buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer),
                        } => {
                            for elem in buffer.iter_mut() {
                                *elem = (generator.next_value() * std::i16::MAX as f32) as i16;
                            }
                        }
                        cpal::StreamData::Output {
                            buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer),
                        } => {
                            for elem in buffer.iter_mut() {
                                *elem = generator.next_value();
                            }
                        }
                        _ => panic!("unsupported stream data"),
                    }
                })
            });
        }
    }
}
