use std;
use std::sync::mpsc::{channel, Sender};
use std::thread;

use super::{SoundError, SoundGenerator};
use cpal;

/// This is the sound API that allows you to send events to your generator.
pub struct SoundDriver<T: Send + 'static> {
    event_loop: Option<cpal::EventLoop>,
    format: Option<cpal::Format>,
    stream_id: Option<cpal::StreamId>,
    tx: Option<Sender<T>>,
    generator: Option<Box<SoundGenerator<T>>>,
    err: SoundError,
}

impl<T: Send + 'static> SoundDriver<T> {
    /// After calling [`SoundDriver::new`], you can call this function to see if the audio initialization was a success.
    pub fn get_error(&self) -> SoundError {
        self.err
    }

    /// Initialize the sound device and provide the generator to the driver.
    pub fn new(generator: Box<SoundGenerator<T>>) -> Self {
        let device = cpal::default_output_device();
        let mut event_loop = None;
        let mut format = None;
        let mut stream_id = None;
        let mut err = SoundError::NoError;
        if let Some(ref dev) = device {
            match dev.default_output_format() {
                Ok(fmt) => {
                    let evt = cpal::EventLoop::new();
                    match evt.build_output_stream(dev, &fmt) {
                        Ok(str) => stream_id = Some(str),
                        Err(e) => {
                            err = SoundError::OutputStream;
                            println!("error : could not build output stream : {}\n", e)
                        }
                    }
                    println!("sound device : {} format {:?}\n", dev.name(), fmt.clone());
                    format = Some(fmt);
                    event_loop = Some(evt);
                }
                Err(e) => {
                    err = SoundError::UnknownStreamFormat;
                    println!("error : could not get default output format : {:?}\n", e);
                }
            }
        } else {
            err = SoundError::NoDevice;
            println!("warning : no sound device detected\n");
        }
        Self {
            event_loop,
            format,
            stream_id,
            tx: None,
            generator: Some(generator),
            err,
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
            thread::spawn(move || {
                println!("starting audio loop");
                evt.play_stream(stream_id);
                generator.init(sample_rate);
                evt.run(move |_stream_id, stream_data| {
                    for event in rx.try_iter() {
                        generator.handle_event(event);
                    }

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
