use crate::PlaySoundParams;

use std::sync::mpsc;

#[path = "coreaudio/coreaudio.rs"]
mod coreaudio;

// bindgen lost some defines from coreaudio.rs
const _saudio_kAudioFormatLinearPCM: u32 = 1819304813; //'lpcm';
const _saudio_kLinearPCMFormatFlagIsFloat: u32 = 1 << 0;
const _saudio_kAudioFormatFlagIsPacked: u32 = 1 << 3;

use coreaudio::*;

mod consts {
    pub const DEVICE: &'static str = "default\0";
    pub const RATE: u32 = 44100;
    pub const CHANNELS: u32 = 2;
    pub const BUFFER_FRAMES: u32 = 4096;
}

pub struct AudioContext {
    tx: mpsc::Sender<crate::mixer::AudioMessage>,
    id: usize,
}

unsafe extern "C" fn saudio_coreaudio_callback(
    user_data: *mut ::std::os::raw::c_void,
    queue: _saudio_AudioQueueRef,
    buffer: _saudio_AudioQueueBufferRef,
) {
    let mut mixer = &mut *(user_data as *mut crate::mixer::Mixer);

    let num_frames = (*buffer).mAudioDataByteSize / (2 * 4);
    let buf =
        std::slice::from_raw_parts_mut((*buffer).mAudioData as *mut f32, num_frames as usize * 2);

    mixer.fill_audio_buffer(buf, num_frames as _);

    AudioQueueEnqueueBuffer(queue, buffer, 0, std::ptr::null_mut());
}

impl AudioContext {
    pub fn new() -> AudioContext {
        use crate::mixer::{self, Mixer};

        let (tx, rx) = mpsc::channel();

        let mixer = Box::new(Mixer::new(rx));

        unsafe {
            let fmt = _saudio_AudioStreamBasicDescription {
                mSampleRate: consts::RATE as f64,
                mFormatID: _saudio_kAudioFormatLinearPCM,
                mFormatFlags: _saudio_kLinearPCMFormatFlagIsFloat
                    | _saudio_kAudioFormatFlagIsPacked,
                mFramesPerPacket: 1,
                mChannelsPerFrame: consts::CHANNELS,
                mBytesPerFrame: 4 * consts::CHANNELS,
                mBytesPerPacket: 4 * consts::CHANNELS,
                mBitsPerChannel: 32,
                mReserved: 0,
            };
            let mut ca_audio_queue: _saudio_AudioQueueRef = std::mem::zeroed();
            let res = AudioQueueNewOutput(
                &fmt,
                Some(saudio_coreaudio_callback),
                Box::into_raw(mixer) as *mut _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut ca_audio_queue,
            );
            assert!(res == 0);
            assert!(ca_audio_queue.is_null() == false);

            // create 2 audio buffers
            for _ in 0..2 {
                let mut buf: _saudio_AudioQueueBufferRef = std::ptr::null_mut();
                let buf_byte_size = consts::BUFFER_FRAMES * fmt.mBytesPerFrame;
                let res = AudioQueueAllocateBuffer(ca_audio_queue, buf_byte_size, &mut buf);
                assert!(res == 0);
                assert!(buf.is_null() == false);
                (*buf).mAudioDataByteSize = buf_byte_size;
                std::ptr::write_bytes(
                    (*buf).mAudioData as *mut u8,
                    0,
                    (*buf).mAudioDataByteSize as usize,
                );
                AudioQueueEnqueueBuffer(ca_audio_queue, buf, 0, std::ptr::null_mut());
            }

            let res = AudioQueueStart(ca_audio_queue, std::ptr::null_mut());
            assert!(res == 0);
        }

        AudioContext { id: 0, tx }
    }
}

pub struct Sound {
    id: usize,
}

impl Sound {
    pub fn load(ctx: &mut AudioContext, data: &[u8]) -> Sound {
        let id = ctx.id;

        let mut audio_stream = {
            let file = std::io::Cursor::new(data);
            audrey::Reader::new(file).unwrap()
        };
        let samples = audio_stream
            .frames::<[f32; 2]>()
            .map(std::result::Result::unwrap)
            .collect::<Vec<_>>();

        ctx.tx
            .send(crate::mixer::AudioMessage::AddSound(id, samples))
            .unwrap();
        ctx.id += 1;

        Sound { id }
    }

    pub fn play(&mut self, ctx: &mut AudioContext, params: PlaySoundParams) {
        ctx.tx
            .send(crate::mixer::AudioMessage::PlaySound(
                self.id,
                params.looped,
                params.volume,
            ))
            .unwrap();
    }

    pub fn stop(&mut self, ctx: &mut AudioContext) {
        ctx.tx
            .send(crate::mixer::AudioMessage::StopSound(self.id))
            .unwrap();
    }

    pub fn set_volume(&mut self, ctx: &mut AudioContext, volume: f32) {
        ctx.tx
            .send(crate::mixer::AudioMessage::SetVolume(self.id, volume))
            .unwrap();
    }
}
