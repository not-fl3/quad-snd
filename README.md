# quad-snd

Sound library for `miniquad`. Originally a fork of [uni-snd](https://github.com/unrust/uni-snd).

## Features

* wav/ogg decoding
* sound mixer

## Supported platforms

* Linux/Mac/Windows (with cpal)
* Wasm (with WebAudio)
* ~~Android~~ WIP

## Usage example

```rust
// quad-snd can load wav and ogg files
let wav_sound = decoder::read_wav(WAV_BYTES).unwrap();
let ogg_sound = decoder::read_ogg(OGG_BYTES).unwrap();

// and play them simulteniously
mixer.play(wav_sound);
mixer.play(ogg_sound);

let mut mixer = SoundMixer::new();

// `mixer.frame()` is wasm-friendly and can be invoked on requestAnimationFrame()
loop {
    mixer.frame();
}

```


