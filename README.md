# quad-snd

> Note: This is a fork of quad-snd. It removes audrey.


High-level, light-weight, and opinionated audio library.

- [x] Web: WebAudio  
- [x] Android: OpenSLES  
- [x] Linux: Alsa  
- [x] macOS: CoreAudio  
- [x] Windows: Wasapi
- [X] iOS: CoreAudio

Being high-level enough allows `quad-snd` to use very different approaches to each backend. For example, for WebAudio all the playback and mixing is based on Audio nodes, while in OpenSLES `quad-snd` itself is responsible for mixing.

`quad-snd` lacks lots of features and the best way to use the library - either fork a repo and fine-tune it for your needs or just copy-paste some code from certain audio backends.

Biggest difference from any other sound library in rust:  
`quad-snd` is small. Each backend implementation is ~300LoC code and is self sufficient - you can copy-paste the whole thing and run it, (almost)no common code, dependencies or anything like that would be required.

## Attribution

While building `quad-snd` I looked into the implementation of the following libraries:

https://github.com/floooh/sokol/blob/master/sokol_audio.h  
https://github.com/norse-rs/audir  
https://github.com/unrust/uni-snd  
