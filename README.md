# quad-snd

High-level, light-weight, and opinionated audio library. 

- [x] Web: WebAudio  
- [x] Android: OpenSLES  
- [x] Linux: Alsa  
- [x] Mac: CoreAudio  
- [x] Windows: Wasapi   
- [ ] IOS: CoreAudio(?)  

Being high-level enough allows `quad-snd` to use very different approaches to each backend. For example, for WebAudio all the playback and mixing is based on Audio nodes, while in OpenSLES `quad-snd` itself is responsible for mixing.

`quad-snd` lacks lots of features and the best way to use the library - either fork a repo and fine-tune it for your needs or just copy-paste some code from certain audio backends.

Biggest difference from any other sound library in rust:  
`quad-snd` is small. Each backend implementation is ~300LoC code and is self sufficient - you can copy-paste the whole thing and run it, (almost)no common code, dependencies or anything like that would be required.

The only dependency is `audrey`. `audrey` helps backends that do not have file parsing functionality (all the platforms but web) to get bytes out of encoded .wav/.ogg. When web is not required - getting rid of `audrey` and use anything else(or nothing at all) for audio decoding is a super easy fix.

## Attribution

While building `quad-snd` I looked into the implementation of the following libraries:

https://github.com/floooh/sokol/blob/master/sokol_audio.h  
https://github.com/norse-rs/audir  
https://github.com/unrust/uni-snd  
