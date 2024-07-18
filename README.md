# OrCo
OrCo compiler toolchain
[Developed on streams](https://www.youtube.com/playlist?list=PLvZASPqsD2VjqJ6968gEhoLlCn0i0rqHH)

## Goals
OrCo has the following goals:
- Bring hot code reloading, intellisence, debugging, interpreters, cross-compilers and similar features to all supported languages
- Easy language interop

Some note I worte a long time ago. Doesn't really make sence to me, but should make sence for new developers:
> Note for developers:
> > Intermediate Representation is nesesary, because
> > we can't just invoke a backend(f.e. cranelift) and
> > tell it to declare a trait. And we can't just parse
> > a language into an IR, because of LSP support.

## Concerns
Some things might be concerning:
- Span. Spans are probably too heavy
- AST is made of Arc's, instead of centrual storadge and IDs
- Metadata. Proper way would be to have custom AST nodes inherit normal AST nodes, but Rust doesn't have inheritance

## Some guidelines which I'll probably forget soon
- Add `span: Span` filed into structs instead of using `Spanned<Struct>`. This will make code simpler. `Spanned` was added mainly for enums

## Roadmap for next few streams
You can watch me do this live on [Twitch](https://www.twitch.tv/infinitecoder01) and [Youtube](https://www.youtube.com/@InfiniteCoder02/)

Roadmap for now:
- [x] Symbols
- [x] Paths
- [x] Floats
- [x] Frontend-side diagnostics (and diagnostics refactor (and lints))
- [x] Fix cyclic Arc by implementing inner pointers
- [x] Reduce the use of `Spanned<Struct>`
- [ ] Typecasts
- [ ] Effect system?!
- [ ] ?Unset literal
- [ ] While loop
- [ ] Pointers
- [ ] Arrays
- [ ] Structs
- [ ] C Frontend (and a blog post on it hopefully)
- [ ] Post-typechecking frontend-side checks
- [ ] Operator Overloading & Traits
- [ ] Rust frontend
- [ ] Self-hosting
