# Project Brief: Caption Maker

## Tujuan

Aplikasi desktop Windows-only yang mengubah file audio/video menjadi subtitle `.ass` menggunakan Whisper, sepenuhnya offline. Fokus bahasa: Indonesia. Output mendukung word-level timestamp dan highlight karaoke opsional dengan grouping 1/2/3 kata per baris.

## Keputusan Teknis yang Sudah Terkunci

| Aspek | Keputusan |
|---|---|
| Platform | Windows saja |
| Stack | Tauri + whisper.cpp (via crate `whisper-rs`, dikompilasi statis) |
| Frontend | HTML/JS/TS ringan |
| Input | mp4, mkv, mov, mp3, wav, m4a (via FFmpeg sidecar) |
| Model | base / small / medium (ggml), bisa dipilih di UI, bisa impor manual |
| Bahasa | Indonesia saja (tidak ada terjemahan) |
| Timestamp | Word-level |
| Grouping | 1, 2, atau 3 kata per baris (pilihan pengguna) |
| Highlight | Opsional, pakai tag ASS `\kf` (progressive fill) |
| Output | File `.ass` |
| Mode | 100% offline, tidak ada network call saat runtime |
| Dokumentasi | README lengkap sejak v0.1.0 |

## Arsitektur yang Direncanakan

```text
caption-maker/
├── src-tauri/
│   ├── src/
│   │   ├── main.rs           # entry Tauri
│   │   ├── commands.rs       # command: transcribe, export_ass
│   │   ├── whisper.rs        # wrapper whisper-rs
│   │   └── ass_generator.rs  # word grouping + ASS writer
│   ├── binaries/             # ffmpeg.exe (sidecar)
│   ├── Cargo.toml
│   └── tauri.conf.json       # externalBin: ffmpeg
├── src/                      # frontend
│   ├── index.html
│   └── main.js
├── README.md
└── package.json
```

Catatan teknis penting:

- FFmpeg dibundel sebagai sidecar (`externalBin` di `tauri.conf.json`). Nama file Windows: `ffmpeg-x86_64-pc-windows-msvc.exe`.
- whisper.cpp dipakai via crate `whisper-rs` (kompilasi statis, tanpa sidecar tambahan).
- Token timing dari whisper.cpp di-merge menjadi kata utuh menggunakan pendekatan seperti crate `whisper-captions` (menghindari potongan BPE seperti `" Mul"`, `"do"`, `"on,"`).
- Highlight ASS memakai `\kf` dengan durasi per kata dalam centisecond (`(end_ms - start_ms) / 10`).

## Alur Pipeline Inti

```text
input file → FFmpeg ekstrak audio (16kHz mono WAV)
           → whisper-rs transkripsi (word timestamps ON, lang=id)
           → merge token BPE → daftar kata + timing
           → grouping 1/2/3 kata
           → generate .ass (style default + tag \kf opsional)
           → simpan file
```

## Gaya Subtitle ASS

Satu style default di `[V4+ Styles]`: font, ukuran, warna primer, outline, shadow, alignment (bisa dikustom di UI).

Setiap baris `Dialogue` berisi 1–3 kata dengan tag `\kf<centisecond>` per kata jika highlight diaktifkan.

Contoh 2 kata + highlight: `{\kf50}Halo {\kf70}dunia`

## Fase Kerja

- **Fase 0 — Validasi di Codespaces:** uji whisper-rs + word timestamps pada 1 file audio Indonesia, merge token → group 2 kata → generate `.ass`, buka di VLC/Aegisub.
- **Fase 1 — Core engine (Rust lib/CLI):** fungsi `transcribe()`, `merge_tokens_to_words()`, `group_words()`, `write_ass()`. Unit test untuk generator ASS.
- **Fase 2 — Test + CI:** GitHub Actions (Windows runner) untuk unit & integration test.
- **Fase 3 — UI Tauri:** drag-drop, pilih model, pilih kata per baris (1/2/3), toggle highlight, progress, preview, export.
- **Fase 4 — Packaging Windows:** bundle FFmpeg + model (atau mekanisme impor model), installer NSIS/MSI, uji di mesin bersih, code signing opsional.
- **Fase 5 — Rilis v0.1.0:** README lengkap, changelog, catatan lisensi (Whisper MIT, whisper.cpp MIT, FFmpeg LGPL/GPL).

## Batasan & Asumsi

- Hanya bahasa Indonesia (tidak ada auto-detect multi-bahasa).
- Tidak ada fitur terjemahan.
- Performa tergantung CPU/GPU pengguna; model medium berat di CPU.
- Model tidak di-bundle default di installer (opsi: impor manual) — keputusan final menyusul.
- Distribusi FFmpeg perlu memperhatikan lisensi (LGPL/GPL) — dokumentasikan di README.

## Yang Diminta dari AI

Menyusun spesifikasi teknis detail untuk Fase 0 dan Fase 1 (lihat `SPEC_FASE0_1.md`), mencakup:

1. Struktur crate Rust (lib.rs + binary) untuk core engine yang bisa dites headless.
2. Signature fungsi Rust untuk: `transcribe`, `merge_tokens_to_words`, `group_words`, `write_ass`.
3. Contoh skema data (`Word { text, start_ms, end_ms }`, `Group { words: Vec<Word> }`).
4. Contoh output `.ass` minimal yang valid (header + 3 baris Dialogue dengan `\kf`).
5. Cara mengonfigurasi whisper-rs agar mengeluarkan token-level timing (parameter yang relevan).
6. Rencana unit test untuk `write_ass` (timestamp, escaping, grouping 1/2/3 kata).
