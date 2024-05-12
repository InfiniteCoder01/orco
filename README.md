# OrCo
OrCo compiler toolchain
[Developed on streams](https://www.youtube.com/playlist?list=PLvZASPqsD2VjqJ6968gEhoLlCn0i0rqHH)

## Goals
OrCo has the following goals:
- Easy FFI
- Multiple backends

Note for developers:
Intermediate Representation is nesesary, because
we can't just invoke a backend(f.e. cranelift) and
tell it to declare a trait. And we can't just parse
a language into an IR, because of LSP support.

## Roadmap for next few streams
You can watch me do this live on [Twitch](https://www.twitch.tv/infinitecoder01) and [Youtube](https://www.youtube.com/@InfiniteCoder02/)

Roadmap for now:
- [] Symbols
- [] Paths
- [] Frontend-side diagnostics (and diagnostics refactor)
- [] While loop
- [] C Frontend
- [] Operator Overloading & Traits
- [] Rust frontend
- [] Self-hosting
