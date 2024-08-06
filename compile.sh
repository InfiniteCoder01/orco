RUST_LOG=trace,cranelift=info,regalloc=info cargo run --color=always --package orco-cli -- orco-lang/samples/$1.orco && gcc foo.o && ./a.out
