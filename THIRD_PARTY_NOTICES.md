# Third-Party Notices

Proyek ini memakai perangkat lunak pihak ketiga. Ringkasan lisensi dan sumber:

## 1. Rust crates

- **tauri** (MIT / Apache-2.0) - https://github.com/tauri-apps/tauri
- **tauri-plugin-dialog** (MIT / Apache-2.0) - https://github.com/tauri-apps/plugins-workspace
- **tauri-plugin-fs** (MIT / Apache-2.0) - https://github.com/tauri-apps/plugins-workspace
- **whisper-rs** (MIT) - https://github.com/tazz4843/whisper-rs
- **clap** (MIT / Apache-2.0) - https://github.com/clap-rs/clap
- **hound** (Apache-2.0) - https://github.com/ruuda/hound
- **serde**, **serde_json** (MIT / Apache-2.0) - https://github.com/serde-rs/serde
- **thiserror** (MIT / Apache-2.0) - https://github.com/dtolnay/thiserror
- **pretty_assertions** (MIT / Apache-2.0) - https://github.com/rust-pretty-assertions/rust-pretty-assertions

Lihat `Cargo.lock` untuk daftar transparan lengkap.

## 2. whisper.cpp (MIT)

- https://github.com/ggerganov/whisper.cpp

Digunakan secara tidak langsung lewat crate `whisper-rs`.

## 3. FFmpeg (GPL)

- Sumber resmi: https://ffmpeg.org/
- Build yang dipakai: https://github.com/GyanD/codexffmpeg (tag `9.1`, paket `essentials_build`)
- Lisensi: **GPL v3** (karena `--enable-gpl --enable-libx264 --enable-libass`)

**Catatan penting**: FFmpeg yang dibundel bersama aplikasi (sidecar) berlisensi GPL. Redistribusi wajib menyertakan sumber FFmpeg, teks lisensi GPL, dan catatan perubahan (jika ada).

**Pinning & verifikasi**: Versi FFmpeg dipin ke `9.1` dan integritasnya diverifikasi via SHA256 di `.github/workflows/build-windows.yml`. Hash `ffmpeg.exe` dan `ffprobe.exe` yang dipakai saat build dicatat di workflow tersebut.

**Sidecar path**: Aplikasi mencari `ffmpeg.exe` dan `ffprobe.exe` di direktori yang sama dengan executable aplikasi (via `sidecar_path` di `src-tauri/src/commands.rs`), bukan di `PATH` sistem. Ini memastikan aplikasi portable dan konsisten di Windows.

Proyek ini menyediakan FFmpeg sebagai *external binary* di `src-tauri/binaries/` (di-build pada Fase 4). **Binary tidak dilacak Git** (lihat `.gitignore`).

Jika ingin distribusi tanpa kewajiban GPL, gunakan build FFmpeg LGPL tanpa `--enable-gpl` dan tanpa `--enable-libx264`. Namun burn-in akan membutuhkan encoder H.264 alternatif (mis. `libopenh264`, `libx265`, atau `mpeg4`) - belum diimplementasikan di proyek ini.

## 4. Font

| Font | File | Lisensi | Sumber |
|---|---|---|---|
| Poppins Bold | `caption-core/assets/fonts/Poppins-Bold.ttf` | SIL Open Font License 1.1 | https://github.com/google/fonts/tree/main/ofl/poppins |
| Poppins ExtraBold | `caption-core/assets/fonts/Poppins-ExtraBold.ttf` | SIL OFL 1.1 | https://github.com/google/fonts/tree/main/ofl/poppins |
| Anton | `caption-core/assets/fonts/Anton-Regular.ttf` | SIL OFL 1.1 | https://github.com/google/fonts/tree/main/ofl/anton |
| Bangers | `caption-core/assets/fonts/Bangers-Regular.ttf` | SIL OFL 1.1 | https://github.com/google/fonts/tree/main/ofl/bangers |

Tiap font disertai salinan `OFL.txt` di direktori yang sama.

## 5. Model Whisper

Model (mis. `ggml-small.bin`) tidak dilacak Git dan tidak didistribusikan bersama aplikasi. Pengguna mengunduh sendiri dari sumber resmi (Hugging Face `ggerganov/whisper.cpp`). Model Whisper berlisensi MIT (OpenAI), dengan ketentuan tambahan - lihat https://github.com/openai/whisper/blob/main/model-card.md.

## 6. Catatan lisensi proyek

Proyek ini sendiri berlisensi MIT (lihat `LICENSE`). Lisensi MIT tidak mengubah lisensi pihak ketiga di atas. Distribusi binary yang menyertakan FFmpeg GPL harus tetap mematuhi GPL untuk komponen FFmpeg.