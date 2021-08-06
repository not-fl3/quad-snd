// https://github.com/floooh/sokol/blob/master/sokol_audio.h
// https://github.com/norse-rs/audir/blob/master/audir/src/wasapi/mod.rs

use crate::PlaySoundParams;

use winapi::shared::guiddef::{CLSID, IID};
use winapi::shared::ksmedia;
use winapi::shared::minwindef::*;
use winapi::shared::mmreg::*;
use winapi::um::audioclient::*;
use winapi::um::audiosessiontypes::*;
use winapi::um::combaseapi::*;
use winapi::um::mmdeviceapi::*;
use winapi::um::objbase::*;
use winapi::um::synchapi::*;
use winapi::um::winbase::*;

use std::sync::mpsc;

// thanks sokol_audio!
// https://github.com/floooh/sokol/blob/master/sokol_audio.h#L559
static IID_IAudioClient: IID = IID {
    Data1: 0x1cb9ad4c,
    Data2: 0xdbfa,
    Data3: 0x4c32,
    Data4: [0xb1, 0x78, 0xc2, 0xf5, 0x68, 0xa7, 0x03, 0xb2],
};
static IID_IMMDeviceEnumerator: IID = IID {
    Data1: 0xa95664d2,
    Data2: 0x9614,
    Data3: 0x4f35,
    Data4: [0xa7, 0x46, 0xde, 0x8d, 0xb6, 0x36, 0x17, 0xe6],
};
static CLSID_IMMDeviceEnumerator: CLSID = CLSID {
    Data1: 0xbcde0395,
    Data2: 0xe52f,
    Data3: 0x467c,
    Data4: [0x8e, 0x3d, 0xc4, 0x57, 0x92, 0x91, 0x69, 0x2e],
};
static IID_IAudioRenderClient: IID = IID {
    Data1: 0xf294acfc,
    Data2: 0x3146,
    Data3: 0x4483,
    Data4: [0xa7, 0xbf, 0xad, 0xdc, 0xa7, 0xc2, 0x60, 0xe2],
};
static IID_Devinterface_Audio_Render: IID = IID {
    Data1: 0xe6327cad,
    Data2: 0xdcec,
    Data3: 0x4949,
    Data4: [0xae, 0x8a, 0x99, 0x1e, 0x97, 0x6a, 0x79, 0xd2],
};
static IID_IActivateAudioInterface_Completion_Handler: IID = IID {
    Data1: 0x94ea2b94,
    Data2: 0xe9cc,
    Data3: 0x49e0,
    Data4: [0xc0, 0xff, 0xee, 0x64, 0xca, 0x8f, 0x5b, 0x90],
};

mod consts {
    pub const CHANNELS: u32 = 2;
    pub const SAMPLE_RATE: u32 = 44100;
    pub const BUFFER_FRAMES: u32 = 4096;
}

unsafe fn audio_thread(mut mixer: crate::mixer::Mixer) {
    CoInitializeEx(std::ptr::null_mut(), COINIT_MULTITHREADED);

    let buffer_end_event = CreateEventA(std::ptr::null_mut(), FALSE, FALSE, std::ptr::null());
    assert!(buffer_end_event.is_null() == false);

    let mut device_enumerator: *mut IMMDeviceEnumerator = std::ptr::null_mut();
    let hr = CoCreateInstance(
        &CLSID_IMMDeviceEnumerator,
        std::ptr::null_mut(),
        CLSCTX_ALL,
        &IID_IMMDeviceEnumerator,
        &mut device_enumerator as *mut _ as _,
    );
    assert!(hr >= 0, "CoCreatInstance failed");

    let mut device: *mut IMMDevice = std::ptr::null_mut();
    let hr = (*device_enumerator).GetDefaultAudioEndpoint(eRender, eConsole, &mut device);
    assert!(hr >= 0, "GetDefaultAudioEndPoint failed");

    let mut audio_client: *mut IAudioClient = std::ptr::null_mut();
    let hr = (*device).Activate(
        &IID_IAudioClient,
        CLSCTX_ALL,
        std::ptr::null_mut(),
        &mut audio_client as *mut _ as _,
    );
    assert!(hr >= 0, "Device Activate failed");

    let mut state = 0;
    (*device).GetState(&mut state);
    assert!(
        state & DEVICE_STATE_ACTIVE != 0,
        "Default device not active"
    );

    let format = WAVEFORMATEX {
        nChannels: consts::CHANNELS as _,
        nSamplesPerSec: consts::SAMPLE_RATE as _,
        wFormatTag: WAVE_FORMAT_EXTENSIBLE,
        wBitsPerSample: 32,
        nBlockAlign: consts::CHANNELS as u16 * 4,
        nAvgBytesPerSec: consts::CHANNELS as u32 * consts::SAMPLE_RATE as u32 * 4,
        cbSize: (std::mem::size_of::<WAVEFORMATEXTENSIBLE>() - std::mem::size_of::<WAVEFORMATEX>())
            as _,
    };

    const FRONT_LEFT: u32 = 0b0001;
    const FRONT_RIGHT: u32 = 0b0010;

    let format_extensible = WAVEFORMATEXTENSIBLE {
        Format: format,
        Samples: 4 * 8,
        dwChannelMask: FRONT_LEFT | FRONT_RIGHT,
        SubFormat: ksmedia::KSDATAFORMAT_SUBTYPE_IEEE_FLOAT,
    };

    // https://docs.microsoft.com/en-us/windows/win32/coreaudio/audclnt-streamflags-xxx-constants
    const AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM: u32 = 0x80000000;
    const AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY: u32 = 0x08000000;

    let dur = consts::BUFFER_FRAMES as f64 / (consts::SAMPLE_RATE as f64 * 1.0 / 10000000.0);
    let hr = (*audio_client).Initialize(
        AUDCLNT_SHAREMODE_SHARED,
        AUDCLNT_STREAMFLAGS_EVENTCALLBACK
            | AUDCLNT_STREAMFLAGS_AUTOCONVERTPCM
            | AUDCLNT_STREAMFLAGS_SRC_DEFAULT_QUALITY,
        dur as _,
        0,
        &format_extensible as *const _ as _,
        std::ptr::null(),
    );
    if hr < 0 {
        println!("Error code: 0x{:x}", hr as u32);
    }
    assert!(hr >= 0, "audio_client.Initialize failed");

    let mut dst_buffer_frames = 0;
    let hr = (*audio_client).GetBufferSize(&mut dst_buffer_frames);
    assert!(hr >= 0, "GetBufferSize failed");

    let mut render_client: *mut IAudioRenderClient = std::ptr::null_mut();
    let hr = (*audio_client).GetService(&IID_IAudioRenderClient, &mut render_client as *mut _ as _);
    assert!(
        hr >= 0,
        "sokol_audio wasapi: audio client GetService failed"
    );

    let hr = (*audio_client).SetEventHandle(buffer_end_event);
    assert!(hr >= 0, "SetEventHandle failed");

    (*audio_client).Start();
    loop {
        WaitForSingleObject(buffer_end_event, INFINITE);

        let mut padding = 0;
        if (*audio_client).GetCurrentPadding(&mut padding) < 0 {
            continue;
        }
        let num_frames = dst_buffer_frames - padding;

        let mut wasapi_buffer: *mut u8 = std::ptr::null_mut();
        if (*render_client).GetBuffer(num_frames, &mut wasapi_buffer) < 0 {
            continue;
        }
        assert!(wasapi_buffer.is_null() == false);

        let buffer = std::slice::from_raw_parts_mut(
            wasapi_buffer as *mut f32,
            num_frames as usize * consts::CHANNELS as usize,
        );

        mixer.fill_audio_buffer(buffer, num_frames as _);

        (*render_client).ReleaseBuffer(num_frames, 0);
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
