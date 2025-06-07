# orco
[![wakatime](https://wakatime.com/badge/github/InfiniteCoder01/orco.svg)](https://wakatime.com/badge/github/InfiniteCoder01/orco)

orco is a compiler toolchain focusing on performace and extensibility
[Developed on streams](https://www.youtube.com/playlist?list=PLvZASPqsD2VjqJ6968gEhoLlCn0i0rqHH)

## Goals
orco development is currently guided by those goals:
1. LSP features:
   - Syntax highlighting
   - Code completion
   - Hover info
   - Go to [definition/uses/etc.]
   - Inline docs
2. Runtime features such as:
   - Hot code reloading
   - JIT
   - Debugging
   - Interpreting
   - Cross-compilation
3. Easy language interop & generation of C (or C-like) headers, transpilation to C
4. Package/dependency management (to some extent, possibly functional)

## Roadmap for next few streams
You can watch me do this live on [![twitch](https://assets.twitch.tv/assets/favicon-16-52e571ffea063af7a7f4.png) Twitch](https://www.twitch.tv/infinitecoder01) and [![youtube](https://www.youtube.com/favicon.ico) Youtube](https://www.youtube.com/@InfiniteCoder02/)

Roadmap for now (**Lacks behing A LOT of rewrites. This is like a year as outdated at this point**):
- [x] Symbols
- [x] Paths
- [x] Floats
- [x] Frontend-side diagnostics (and diagnostics refactor (and lints))
- [x] Fix cyclic Arc by implementing inner pointers
- [x] Reduce the use of `Spanned<Struct>`
- [x] Make IR first-class (a BIG refactor)
- [x] Remove Ariadne completely (+ lexer abort compilation)
- [x] Move spans to frontend?
- [x] Comptime type hints
- [x] Path as an operator \[cancelled\]
- [x] `orco::Path` borrowing names? \[cancelled\]
- [x] Get metadata traits out of macros
- [x] Reorganize IR Tree to hold references to modules. Maybe local resolve should only be in module?
- [x] Parent modules (`super::`)
- [ ] Fix lazy evaluation:
    - [ ] Extract part of TypeInference struct into something like LocalContext
    - [ ] Rename TypeInference to something like Context and rename all the functions
    - [ ] Remove lifetime from TypeInference/Context struct and make it shareable/cloneable
    - [ ] Isolate LocalContext for all ensure_evaluated
- [ ] Comptimes in blocks
- [ ] Structs
- [ ] Generics
- [ ] Operator Overloading & Traits
- [ ] Finish the interpreter
- [ ] Unwinding?
- [ ] Effect system?!
- [ ] Pointers
- [ ] Typecasts
- [ ] Arrays
- [ ] While loop
- [ ] C Frontend (and a blog post on it hopefully)
- [ ] Post-typechecking frontend-side checks
- [ ] Rust frontend
- [ ] Self-hosting
