use quad_snd::{AudioContext, Sound};

fn main() {
    let mut ctx = AudioContext::new();
    //let mut sound = Sound::load(&mut ctx, include_bytes!("test.ogg"));
    //let mut sound = Sound::load(&mut ctx, include_bytes!("test_96000.wav"));
    //let mut sound = Sound::load(&mut ctx, include_bytes!("test_13000.wav"));
    let mut sound = Sound::load(&mut ctx, include_bytes!("test.wav"));

    sound.play(&mut ctx, Default::default());

    loop {}
}
