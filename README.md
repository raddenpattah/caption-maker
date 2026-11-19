# Caption Maker

The current implementation is the headless Rust core and CLI (Fases 0–1).
Tauri, UI, and packaging are not included yet.

## Requirements

- Rust stable and Cargo
- CMake and Clang (the default `whisper` feature builds whisper.cpp)
- FFmpeg available on `PATH`
- A whisper.cpp GGML model file

## Test

Run the core tests without compiling whisper.cpp:

```sh
cargo test --no-default-features
```

Run tests with the default Whisper feature:

```sh
cargo test
```

## CLI

```sh
cargo run --release -- \
  samples/test.wav \
  --model models/ggml-small.bin \
  --words 2 \
  --highlight \
  -o out.ass \
  --dump-tokens tokens.json
```

The CLI also accepts `--ffmpeg <path>` to select a non-default FFmpeg
executable. The model file is not tracked by Git.