#!/usr/bin/env bash
cargo build --package orco-rustc && RUSTC_LOG='orco_rustc' rustc "frontends/orco-rustc/samples/$1.rs" -Z codegen-backend=./target/debug/liborco_rustc.so
