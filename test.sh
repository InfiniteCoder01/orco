#!/usr/bin/env bash
cargo build --package orco-rustc && RUSTC_LOG='orco_rustc' rustc samples/simple.rs -Z codegen-backend=./target/debug/liborco_rustc.so
