// roughly based on http://equalarea.com/paul/alsa-audio.html

use crate::{error::Error, PlaySoundParams};

use quad_alsa_sys as sys;

use std::sync::mpsc;

mod consts {
    pub const DEVICE: &'static str = "default\0";
    pub const RATE: u32 = 44100;
    pub const CHANNELS: u32 = 2;
    pub const PCM_BUFFER_SIZE: u64 = 4096;
}

unsafe fn setup_pcm_device() -> *mut sys::snd_pcm_t {
    let mut pcm_handle = std::ptr::null_mut();

    // Open the PCM device in playback mode
    if sys::snd_pcm_open(
        &mut pcm_handle,
        consts::DEVICE.as_ptr() as _,
        sys::SND_PCM_STREAM_PLAYBACK,
        0,
    ) < 0
    {
        panic!("Can't open PCM device.");
    }

    let mut hw_params: *mut sys::snd_pcm_hw_params_t = std::ptr::null_mut();
    sys::snd_pcm_hw_params_malloc(&mut hw_params);
    sys::snd_pcm_hw_params_any(pcm_handle, hw_params);

    if sys::snd_pcm_hw_params_set_access(pcm_handle, hw_params, sys::SND_PCM_ACCESS_RW_INTERLEAVED)
        < 0
    {
        panic!("Can't set interleaved mode");
    }

    if sys::snd_pcm_hw_params_set_format(pcm_handle, hw_params, sys::SND_PCM_FORMAT_FLOAT_LE) < 0 {
        panic!("Can't set SND_PCM_FORMAT_FLOAT_LE format");
    }
    if sys::snd_pcm_hw_params_set_buffer_size(pcm_handle, hw_params, consts::PCM_BUFFER_SIZE) < 0 {
        panic!("Cant's set buffer size");
    }
    if sys::snd_pcm_hw_params_set_channels(pcm_handle, hw_params, consts::CHANNELS) < 0 {
        panic!("Can't set channels number.");
    }

    let mut rate = consts::RATE;
    if sys::snd_pcm_hw_params_set_rate_near(pcm_handle, hw_params, &mut rate, std::ptr::null_mut())
        < 0
    {
        panic!("Can't set rate.");
    }

    // Write parameters
    if sys::snd_pcm_hw_params(pcm_handle, hw_params) < 0 {
        panic!("Can't set harware parameters.");
    }
    sys::snd_pcm_hw_params_free(hw_params);

    // tell ALSA to wake us up whenever AudioContext::PCM_BUFFER_SIZE or more frames
    //   of playback data can be delivered. Also, tell
    //   ALSA that we'll start the device ourselves.
    let mut sw_params: *mut sys::snd_pcm_sw_params_t = std::ptr::null_mut();

    if sys::snd_pcm_sw_params_malloc(&mut sw_params) < 0 {
        panic!("cannot allocate software parameters structure");
    }
    if sys::snd_pcm_sw_params_current(pcm_handle, sw_params) < 0 {
        panic!("cannot initialize software parameters structure");
    }

    // if sys::snd_pcm_sw_params_set_avail_min(
    //     pcm_handle,
    //     sw_params,
    //     AudioContext::PCM_BUFFER_SIZE,
    // ) < 0
    // {
    //     panic!("cannot set minimum available count");
    // }
    if sys::snd_pcm_sw_params_set_start_threshold(pcm_handle, sw_params, 0) < 0 {
        panic!("cannot set start mode");
    }
    if sys::snd_pcm_sw_params(pcm_handle, sw_params) < 0 {
        panic!("cannot set software parameters");
    }
    sys::snd_pcm_sw_params_free(sw_params);

    if sys::snd_pcm_prepare(pcm_handle) < 0 {
        panic!("cannot prepare audio interface for use");
    }

    pcm_handle
}

unsafe fn audio_thread(mut mixer: crate::mixer::Mixer) {
    let mut buffer: Vec<f32> = vec![0.0; consts::PCM_BUFFER_SIZE as usize * 2];

    let pcm_handle = setup_pcm_device();

    loop {
        // // find out how much space is available for playback data
        // teoretically it should reduce latency - we will fill a minimum amount of
        // frames just to keep alsa busy and will be able to mix some fresh sounds
        // it does, but also randmly panics sometimes

        // let frames_to_deliver = sys::snd_pcm_avail_update(pcm_handle);
        // println!("{}", frames_to_deliver);
        // let frames_to_deliver = if frames_to_deliver > consts::PCM_BUFFER_SIZE as _ {
        //     consts::PCM_BUFFER_SIZE as i64
        // } else {
        //     frames_to_deliver
        // };

        let frames_to_deliver = consts::PCM_BUFFER_SIZE as i64;

        // ask mixer to fill the buffer
        mixer.fill_audio_buffer(&mut buffer, frames_to_deliver as usize);

        // send filled buffer back to alsa
        let frames_writen = sys::snd_pcm_writei(
            pcm_handle,
            buffer.as_ptr() as *const _,
            frames_to_deliver as _,
        );
        if frames_writen == -libc::EPIPE as i64 {
            println!("Underrun occured: -EPIPE, attempting recover");

            sys::snd_pcm_recover(pcm_handle, frames_writen as _, 0);
        }

        if frames_writen > 0 && frames_writen != frames_to_deliver as _ {
            println!("Underrun occured: frames_writen != frames_to_deliver, attempting recover");

            sys::snd_pcm_recover(pcm_handle, frames_writen as _, 0);
        }
    }
}

pub struct AudioContext {
    mixer_ctrl: crate::mixer::MixerControl,
}

impl AudioContext {
    pub fn new() -> AudioContext {
        use crate::mixer::Mixer;

        let (mixer, mixer_ctrl) = Mixer::new();
        std::thread::spawn(move || unsafe {
            audio_thread(mixer);
        });

        AudioContext { mixer_ctrl }
    }
}

pub struct Sound {
    id: usize,
}

impl Sound {
    pub fn load(ctx: &mut AudioContext, data: &[u8]) -> Sound {
        let id = ctx.mixer_ctrl.load(data);

        Sound { id }
    }

    pub fn play(&mut self, ctx: &mut AudioContext, params: PlaySoundParams) {
        ctx.mixer_ctrl.play(self.id, params);
    }

    pub fn stop(&mut self, ctx: &mut AudioContext) {
        ctx.mixer_ctrl.stop(self.id);
    }

    pub fn set_volume(&mut self, ctx: &mut AudioContext, volume: f32) {
        ctx.mixer_ctrl.set_volume(self.id, volume);
    }
}
