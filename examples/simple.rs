use quad_snd::{AudioContext, Sound};

fn main() {
    let ctx = AudioContext::new();
    let sound_ogg = Sound::load(&ctx, include_bytes!("test.ogg"));
    let sound_wav = Sound::load(&ctx, include_bytes!("test_13000.wav"));

    sound_wav.play(&ctx, Default::default());
    sound_ogg.play(&ctx, Default::default());

    loop {}
}
