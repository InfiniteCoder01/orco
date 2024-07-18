# OrCo
OrCo compiler toolchain
[Developed on streams](https://www.youtube.com/playlist?list=PLvZASPqsD2VjqJ6968gEhoLlCn0i0rqHH)

## Goals
OrCo has the following goals:
- Bring hot code reloading, intellisence, debugging, interpreters, cross-compilers and similar features to all supported languages
- Easy language interop

Note for developers:
Intermediate Representation is nesesary, because
we can't just invoke a backend(f.e. cranelift) and
tell it to declare a trait. And we can't just parse
a language into an IR, because of LSP support.

## Concerns
Some things might be concerning:
- Span. Spans are probably too heavy
- AST is made of Arc's, instead of centrual storadge and IDs
- Metadata. Proper way would be to have custom AST nodes inherit normal AST nodes, but Rust doesn't have inheritance

## Roadmap for next few streams
You can watch me do this live on [Twitch](https://www.twitch.tv/infinitecoder01) and [Youtube](https://www.youtube.com/@InfiniteCoder02/)

Roadmap for now:
- [x] Symbols
- [x] Paths
- [x] Floats
- [ ] Frontend-side diagnostics (and diagnostics refactor (and lints))
- [ ] Fix cyclic Arc by implementing inner pointers
- [ ] Typecasts
- [ ] While loop
- [ ] Pointers
- [ ] Arrays
- [ ] Structs
- [ ] C Frontend (and a blog post on it hopefully)
- [ ] Post-typechecking frontend-side checks
- [ ] Operator Overloading & Traits
- [ ] Rust frontend
- [ ] Self-hosting
