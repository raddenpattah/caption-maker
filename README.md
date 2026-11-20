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
  --prompt "Nama merek dan istilah khusus" \
  --threads 4 \
  -o out.ass \
  --dump-tokens tokens.json
```

The CLI also accepts `--ffmpeg <path>` to select a non-default FFmpeg
executable, `--prompt <teks>` to provide Whisper's initial prompt, and
`--threads <n>` to set the transcription thread count. By default, the thread
count is the available CPU parallelism capped at 8. The model file is not
tracked by Git.

## Roadmap

Status: Fase 0-1 (core engine) selesai; lihat REPORT.md. Roadmap ini adalah rencana dan dapat berubah.

Tahap 1 - Inti (v0.1.0)
- [x] Core engine: transkripsi Whisper word-level, merge token, grouping 1/2/3 kata, generator .ass dengan highlight \kf, CLI (Fase 0-1)
- [ ] Verifikasi visual .ass di VLC/Aegisub
- [ ] CI GitHub Actions di Windows (Fase 2)
- [ ] UI Tauri: drag-drop, pilih model, 1/2/3 kata per baris, toggle highlight, kustom style, progress, preview, ekspor .ass (Fase 3)
- [ ] Packaging Windows: installer NSIS/MSI, FFmpeg sidecar, impor model manual (Fase 4)
- [ ] Rilis v0.1.0: README lengkap, changelog, catatan lisensi (Fase 5)

Tahap 2 - Kualitas hidup (v0.2.x)
- [ ] Ekspor video dengan caption permanen (burn-in)
- [ ] Editor transkrip sederhana (koreksi ejaan sebelum ekspor)
- [ ] Daftar istilah khusus (initial prompt) di UI
- [ ] Deteksi suara (VAD) untuk mengurangi halusinasi di bagian hening
- [ ] Format ekspor tambahan: SRT, VTT, TXT, JSON
- [ ] Preset gaya, antrian batch, akselerasi GPU

Tahap 3 - Fitur kreator (v0.3+)
- [ ] Template gaya ala TikTok/Reels dengan animasi per kata
- [ ] Pemotongan otomatis bagian hening dan kata isian
- [ ] Pemisahan penutur (speaker diarization)
- [ ] Caption real-time dari mikrofon
- [ ] Ekspor ke format editor video

### Batas yang diketahui

- Whisper bisa salah eja kata serapan dan nama merek; timing token berbasis heuristik, bukan forced alignment.
- Model medium berat di CPU tanpa GPU.
- Distribusi FFmpeg (LGPL/GPL, termasuk x264 untuk burn-in) perlu dokumentasi lisensi.
- Multi-bahasa, terjemahan, dan timeline editor penuh sengaja tidak dijadwalkan; keputusan brief "Indonesia saja" tetap berlaku.