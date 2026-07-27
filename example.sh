#!/bin/bash

bin="./target/release/rs-fd2locked2mmap2lc4wasm"

ENV_FILENAME=./src/lib.rs \
  ENV_WAT_FILENAME=./example.wat \
  ENV_COUNT_MODE=wasm \
  "${bin}"
