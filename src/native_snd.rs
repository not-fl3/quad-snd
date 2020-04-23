use std;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use super::{SoundError, SoundGenerator};
use cpal::traits::{DeviceTrait, EventLoopTrait, HostTrait};

/// This is the sound API that allows you to send events to your generator.
pub struct SoundDriver<T: Send + 'static> {
    event_loop: Option<cpal::EventLoop>,
    format: Option<cpal::Format>,
    stream_id: Option<cpal::StreamId>,
    tx: Option<Sender<T>>,
    generator: Option<Box<dyn SoundGenerator<T>>>,
    err: SoundError,
}

impl<T: Send + 'static> SoundDriver<T> {
    /// After calling [`SoundDriver::new`], you can call this function to see if the audio initialization was a success.
    pub fn get_error(&self) -> SoundError {
        self.err
    }

    /// Initialize the sound device and provide the generator to the driver.
    pub fn new(generator: Box<dyn SoundGenerator<T>>) -> Self {
        // Setup the audio system
        let host = cpal::default_host();
        let event_loop = host.event_loop();

        let device = match host.default_output_device() {
            Some(device) => device,
            None => {
                println!("warning : no sound device detected\n");
                return Self {
                    event_loop: Some(event_loop),
                    format: None,
                    stream_id: None,
                    tx: None,
                    generator: Some(generator),
                    err: SoundError::NoDevice,
                };
            }
        };

        let mut output_format = match device.default_output_format() {
            Ok(default_output_format) => default_output_format,
            Err(err) => {
                println!("error : could not get default output format : {:?}\n", err);
                return Self {
                    event_loop: Some(event_loop),
                    format: None,
                    stream_id: None,
                    tx: None,
                    generator: Some(generator),
                    err: SoundError::UnknownStreamFormat,
                };
            }
        };

        // Make it mono
        // TODO support stereo
        output_format.channels = 1;

        let stream_id = match event_loop.build_output_stream(&device, &output_format) {
            Ok(output_stream) => output_stream,
            Err(err) => {
                println!("error : could not build output stream : {}\n", err);
                return Self {
                    event_loop: Some(event_loop),
                    format: Some(output_format),
                    stream_id: None,
                    tx: None,
                    generator: Some(generator),
                    err: SoundError::OutputStream,
                };
            }
        };

        println!(
            "sound device : {} format {:?}\n",
            device.name().unwrap_or("no device name".into()),
            &output_format
        );

        Self {
            event_loop: Some(event_loop),
            format: Some(output_format),
            stream_id: Some(stream_id),
            tx: None,
            generator: Some(generator),
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
        let mut generator = self.generator.take().unwrap();
        if let Some(evt) = self.event_loop.take() {
            evt.play_stream(stream_id).expect("could not play stream");

            thread::spawn(move || {
                println!("starting audio loop");
                generator.init(sample_rate);
                evt.run(move |stream_id, stream_result| {
                    for event in rx.try_iter() {
                        generator.handle_event(event);
                    }

                    let stream_data = match stream_result {
                        Ok(data) => data,
                        Err(err) => {
                            eprintln!("an error occurred on stream {:?}: {}", stream_id, err);
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
