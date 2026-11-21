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

## Preset gaya dan burn-in

Preset gaya caption siap pakai (ala video pendek vertikal) plus ekspor video
dengan caption permanen lewat FFmpeg + libass. Fitur ini belum diverifikasi
secara visual; lihat "Batas yang diketahui".

Preset tersedia (12):

```sh
cargo run --release -- --list-presets
```

Kategori: `kata_per_kata`, `baris_bergaya`, `penekanan_kata`. Contoh:

```sh
# Tulis .ass memakai preset (tanpa transkripsi, dari words-json)
cargo run --release -- \
  --preset kata-tebal-putih \
  --words-json src/mock/words.json \
  --size 1080x1920 \
  -o out/preset.ass

# Burn-in ke MP4 (vertikal; resolusi/fps dipertahankan)
cargo run --release -- \
  --preset tekan-warna-kuning \
  --words-json src/mock/words.json \
  input.mp4 \
  --burn out/burn.mp4 \
  --quality bagus

# Render satu frame PNG (latar abu-abu gelap #202020)
cargo run --release -- \
  --preset baris-neon-mint \
  --words-json src/mock/words.json \
  --size 1080x1920 \
  --render-frame out/frame.png \
  --at-ms 4200
```

Flag baru: `--list-presets`, `--preset <id>`, `--burn <out.mp4>`,
`--quality cepat|bagus` (default `bagus`), `--render-frame <out.png>`
+ `--at-ms <ms>`, `--words-json <path>`, `--size WxH`, `--ffprobe <path>`.
`--preset` mengabaikan `--highlight`/`--words` dengan peringatan.
Tanpa `--preset`, perilaku CLI lama tidak berubah.

### Font

Font berlisensi OFL disertakan di `caption-core/assets/fonts/`:

| Font | File | Lisensi | Sumber |
|---|---|---|---|
| Poppins | `Poppins-Bold.ttf`, `Poppins-ExtraBold.ttf` | OFL | github.com/google/fonts/ofl/poppins |
| Anton | `Anton-Regular.ttf` | OFL | github.com/google/fonts/ofl/anton |
| Bangers | `Bangers-Regular.ttf` | OFL | github.com/google/fonts/ofl/bangers |

Tiap font disertai berkas `OFL.txt` di direktori yang sama.
Fallback bila font tidak ditemukan di sistem: `Arial` (bukan OFL; hanya
fallback runtime, bukan distribusi). Tidak ada panggilan jaringan saat
runtime; font ditanam ke binary lewat `include_bytes!`.

### Peringatan HDR

`probe_video` mendeteksi sumber HDR (`smpte2084` / `arib-std-b67` /
pix_fmt 10-bit) dan mencetak peringatan. **Tone mapping tidak
diimplementasikan**; burn-in pada sumber HDR akan menghasilkan warna
yang belum dikoreksi.

### Lisensi FFmpeg

FFmpeg yang dibundel dengan x264 untuk burn-in berlisensi **GPL**.
Distribusi binary harus mematuhi GPL (sertakan sumber, lisensi, izin).

## Roadmap

Status: Fase 0-1 (core engine) selesai; lihat REPORT.md. Roadmap ini adalah rencana dan dapat berubah.

Tahap 1 - Inti (v0.1.0)
- [x] Core engine: transkripsi Whisper word-level, merge token, grouping 1/2/3 kata, generator .ass dengan highlight \kf, CLI (Fase 0-1)
- [ ] Verifikasi visual .ass di VLC/Aegisub
- [ ] Ekspor video dengan caption permanen (burn-in) - kode selesai, menunggu verifikasi visual
- [ ] Template/preset gaya (12 preset) - kode selesai, menunggu verifikasi visual
- [ ] CI GitHub Actions di Windows (Fase 2)
- [ ] UI Tauri: drag-drop, pilih model, 1/2/3 kata per baris, toggle highlight, kustom style, progress, preview, ekspor .ass (Fase 3)
- [ ] Packaging Windows: installer NSIS/MSI, FFmpeg sidecar, impor model manual (Fase 4)
- [ ] Rilis v0.1.0: README lengkap, changelog, catatan lisensi (Fase 5)

Tahap 2 - Kualitas hidup (v0.2.x)
- [ ] Editor transkrip sederhana (koreksi ejaan sebelum ekspor)
- [ ] Daftar istilah khusus (initial prompt) di UI
- [ ] Deteksi suara (VAD) untuk mengurangi halusinasi di bagian hening
- [ ] Format ekspor tambahan: SRT, VTT, TXT, JSON
- [ ] Antrian batch, akselerasi GPU

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