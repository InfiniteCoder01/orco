# orco
[![wakatime](https://wakatime.com/badge/github/InfiniteCoder01/orco.svg)](https://wakatime.com/badge/github/InfiniteCoder01/orco)

IDK, S2S compiler?
[Developed on streams](https://www.youtube.com/playlist?list=PLvZASPqsD2VjqJ6968gEhoLlCn0i0rqHH)

## Goals
orco development is currently guided by those goals:
1. Effortless language interop.
It should be able to generate very easy bindings for libraries compiled through it.
*Ideally* those bindings should work even with regular compilers when linking the library later.
2. Being able to run the resulting code in any environment. Transpiling to C for compatibility with any
platform, using native libraries, transpiling to JS or LUA.
3. Following on #2, injecting runtime features, such as:
- Hot code reloading
- JIT
- Debugging
- Interpreting

## Roadmap for next few streams
You can watch me do this live on [![twitch](https://assets.twitch.tv/assets/favicon-16-52e571ffea063af7a7f4.png) Twitch](https://www.twitch.tv/infinitecoder01) and [![youtube](https://www.youtube.com/favicon.ico) Youtube](https://www.youtube.com/@InfiniteCoder02/)
Currently working with rust frontend and C backend (reference backend),
going through some of [rust by example](<https://doc.rust-lang.org/stable/rust-by-example/>) and figuring out generics.
