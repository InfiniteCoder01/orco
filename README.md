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
