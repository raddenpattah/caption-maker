# Changelog

Format mengikuti [Keep a Changelog](https://keepachangelog.com/id/1.1.0/),
dan proyek ini mengikuti [Semantic Versioning](https://semver.org/lang/id/).

## [Unreleased]

### Ditambahkan

- Rencana Fase 4: packaging Windows NSIS/MSI, FFmpeg sidecar.
- Uji ekstrem: input HDR, audio non-AAC, video rotasi 90/270.
- Build & uji `cargo tauri dev` / WebView di Windows.

## [0.1.0] - 2026-11-21

Rilis pertama core engine + CLI + preset + burn-in + Tauri backend.

### Ditambahkan — Fase 0-1 (core engine)

- Workspace Rust `caption-core` (lib + binary `caption-cli`).
- Ekstraksi audio via FFmpeg (`extract_audio`) dan loader WAV 16kHz mono.
- Transkripsi Whisper word-level via `whisper-rs` (feature opsional `whisper`).
- Merge token BPE ke kata (menangani bytes UTF-8 lintas token).
- Grouping 1/2/3 kata per baris.
- Generator ASS dengan highlight `\kf` kumulatif.
- CLI: `--model`, `--words`, `--highlight`, `-o`, `--ffmpeg`, `--dump-tokens`, `--prompt`, `--threads`.
- Validator ASS struktural (`validate_ass`).

### Ditambahkan — Fase 2 (perbaikan core)

- `RawToken` mempertahankan bytes; karakter UTF-8 lintas token pulih saat merge.
- CLI mencetak metrik RTF (`audio=... transcribe=... rtf=...`).
- Workflow Windows GitHub Actions (belum diverifikasi hijau).

### Ditambahkan — Fase 3A (UI statis mock)

- Frontend HTML/CSS/JS di `src/` (tanpa framework, tanpa bundler).
- `--dump-words <path>` untuk ekspor kata sebagai JSON.
- API mock di `src/js/api.js`.
- Preview karaoke HTML/CSS (bukan renderer ASS).
- 6 tes JS via `node --test`.

### Ditambahkan — Format ekspor

- SRT (`to_srt`), VTT (`to_vtt`), TXT (`to_txt`), JSON (`to_json`).
- Flag CLI `--format ass|srt|vtt|txt|json`.

### Ditambahkan — Preset gaya

- Skema `StylePreset` (serde JSON) + validasi + 12 preset bawaan.
- Kategori: `kata_per_kata`, `baris_bergaya`, `penekanan_kata`.
- `render_ass_with_preset` dengan 5 mode emphasis: `none`, `fill`, `color_active`, `underline_active`, `scale_active`.
- Scaling otomatis berdasarkan geometri video (acuan 1080 sisi pendek).
- Dukungan box (BorderStyle 3), glow (2 layer), uppercase, letter spacing.

### Ditambahkan — Burn-in FFmpeg

- `probe_video` (ffprobe JSON) — deteksi rotasi, HDR (peringatan saja, tanpa tone mapping).
- `build_burn_args` — x264 (`-crf`, `-preset`), audio copy (AAC) atau re-encode (AAC 192k).
- Parser progress `-progress pipe:1`.
- Path helper anti-escape Windows (relative path di filter).

### Ditambahkan — CLI Fase 3B / burn-in

- `--list-presets`, `--preset <id>`, `--burn <out.mp4>`, `--quality cepat|bagus`,
  `--render-frame <out.png>`, `--at-ms <ms>`, `--words-json <path>`, `--size WxH`,
  `--ffprobe <path>`.

### Ditambahkan — Font

- Poppins Bold, Poppins ExtraBold, Anton, Bangers (SIL OFL 1.1), ditanam via `include_bytes!`.

### Ditambahkan — Tauri backend (Fase 3B, scaffold)

- Crate `caption-tauri` dengan Tauri 2.
- Command: `list_models`, `import_model`, `pick_input_file`, `validate_size_cmd`,
  `list_presets`, `render_ass_from_words`, `transcribe`.
- Integrasi UI: `src/js/tauri-bridge.js` (`invoke`/`listen` via `window.__TAURI__`),
  `api.js` mendeteksi Tauri dan fallback ke mock di browser.
- `withGlobalTauri: true` (tanpa bundler, tanpa npm).

### Ditambahkan — Script uji

- `scripts/render_preset_sheet.sh` — render 24 PNG (12 preset × 2 frame) + video uji sintetis 1080×1920 + burn-in.

### Ditambahkan — Dokumentasi

- README bagian "Preset gaya dan burn-in".
- `REPORT_STYLE.md` (laporan fase style + burn-in).
- `THIRD_PARTY_NOTICES.md` (lisensi pihak ketiga).
- `LICENSE` (MIT).

### Diketahui belum

- **Verifikasi visual NOT RUN** (VLC/Aegisub, penilaian estetika preset).
- Bundle Windows NSIS/MSI belum dibuat (butuh mesin Windows).
- Uji `cargo tauri dev` / WebView belum dilakukan.
- Model Whisper tidak didistribusikan.
- FFmpeg belum dibundel (menunggu Fase 4).
