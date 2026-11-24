# Laporan Fase 0–1

## 1. Ringkasan

- Selesai: workspace Rust, `caption-core`, audio/Whisper pipeline, merge/group, ASS renderer, CLI, dan tes.
- Berhasil menjalankan Whisper small pada `samples/test.wav`: 92 token mentah, 48 kata, 24 grup.
- Tersimpan fixture token mentah dan golden ASS dari run aktual.
- Tidak dikerjakan: Tauri/UI/packaging dan verifikasi visual di VLC/Aegisub.

## 2. Environment

- `rustc --version`: `rustc 1.100.0 (0ef1bc386 2026-11-09)`
- `cargo --version`: `cargo 1.100.0 (3acc79c0c 2026-11-05)`
- `ffmpeg -version | head -1`: `ffmpeg version 6.1.1-3ubuntu5 Copyright (c) 2000-2023 the FFmpeg developers`
- whisper-rs dipin: `=0.14.2` (API diperiksa dari source crate lokal); whisper-rs-sys yang terpilih: `0.12.1`.
- Model: `models/ggml-small.bin`, tipe small, diunduh dari URL Hugging Face pada spec; model tidak dilacak Git.
- Audio uji: `samples/test.wav`, durasi `23.185125` detik, PCM 16-bit stereo 44.1 kHz.
- Transkripsi berjalan menggunakan CPU.

## 3. File tree (`tree -I target -L 4`)

```text
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── REPORT.md
├── caption-core
│   ├── Cargo.toml
│   ├── src
│   │   ├── ass.rs
│   │   ├── audio.rs
│   │   ├── bin
│   │   │   └── caption-cli.rs
│   │   ├── error.rs
│   │   ├── group.rs
│   │   ├── lib.rs
│   │   ├── merge.rs
│   │   ├── model.rs
│   │   └── whisper.rs
│   └── tests
│       ├── ass_tests.rs
│       ├── audio_tests.rs
│       ├── fixtures
│       │   ├── expected.ass
│       │   └── tokens_id_sample.json
│       ├── group_tests.rs
│       └── merge_tests.rs
├── docs
│   ├── PROJECT_BRIEF.md
│   └── SPEC_FASE0_1.md
├── models
│   └── ggml-small.bin
├── out.ass
├── samples
│   └── test.wav
└── tokens.json

9 directories, 26 files
```

## 4. Checklist kriteria selesai dari spec

- **PASS** — Cargo workspace dan `caption-core` lib + binary `caption-cli`: manifest dan `cargo run --bin caption-cli -- --help`.
- **PASS** — Feature `whisper` opsional dan bisa dimatikan: `cargo test --no-default-features` lulus tanpa membangun whisper.cpp.
- **PASS** — `extract_audio` FFmpeg dan `load_wav_16k_mono`: dijalankan end-to-end oleh CLI; unit test konversi i16→f32 dan validasi WAV lulus.
- **PASS** — `transcribe`: run aktual dengan bahasa `id`, `translate=false`, token timestamps aktif, dan konfigurasi token waktu 10 ms.
- **PASS** — `merge_tokens_to_words`: unit test BPE, tanda baca, token khusus, waktu invalid/overlap, serta fixture aktual lulus.
- **PASS** — `group_words`: grouping 1/2/3 kata, sisa, input kosong, dan urutan diuji.
- **PASS** — `write_ass`/`render_ass`: timestamp pembulatan, warna BGR, highlight kumulatif, jeda, escaping, koma, dan golden test lulus.
- **PASS** — CLI mendukung `<input> --model --words 1|2|3 [--highlight] [-o] [--ffmpeg] [--dump-tokens]`; run menghasilkan `.ass` dan JSON.
- **PASS** — Raw token fixture dan golden ASS dari run nyata tersimpan di `caption-core/tests/fixtures/`.
- **PASS** — Model biner diabaikan oleh `.gitignore` dan tidak masuk commit.
- **NOT RUN** — Membuka dan menilai hasil visual/audio di VLC atau Aegisub; aplikasi pemutar tidak dijalankan.
- **PASS** — `cargo fmt --check`, Clippy default dan no-default dengan `-D warnings`, serta test default dan no-default lulus.
- **NOT RUN** — Fase 2+ (CI, Tauri/UI, packaging, rilis), di luar ruang lingkup tugas.

## 5. Output mentah

### a. `cargo test --no-default-features`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running unittests src/lib.rs (target/debug/build/caption-core/f9705d6660ebd458/out/caption_core-f9705d6660ebd458)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/ass_tests.rs (target/debug/build/caption-core/fa537377552b483a/out/ass_tests-fa537377552b483a)

running 14 tests
test comma_in_text_is_preserved ... ok
test braces_and_newlines_are_escaped ... ok
test empty_groups_only_emit_header ... ok
test dialogue_uses_first_start_and_last_end ... ok
test groups_from_grouping_api_render_in_word_order ... ok
test header_contains_required_sections_and_event_columns ... ok
test highlight_inserts_gap_tag ... ok
test highlight_rounding_does_not_accumulate_drift ... ok
test highlight_uses_exact_cumulative_word_durations ... ok
test plain_mode_has_no_karaoke_tags ... ok
test style_line_maps_colors_to_ass_bgr_order ... ok
test real_sample_fixture_matches_golden_ass ... ok
test timestamp_format_basic ... ok
test timestamp_rounds_not_truncates ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/audio_tests.rs (target/debug/build/caption-core/2f4e718c3473aa1d/out/audio_tests-2f4e718c3473aa1d)

running 2 tests
test loads_16khz_mono_i16_as_normalized_f32 ... ok
test rejects_wav_with_wrong_channel_count_or_sample_rate ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/group_tests.rs (target/debug/build/caption-core/fa0f30a99e19dbcb/out/group_tests-fa0f30a99e19dbcb)

running 3 tests
test fewer_words_than_group_size_stays_in_one_group ... ok
test seven_words_are_grouped_with_remainders_for_each_size ... ok
test empty_input_returns_no_groups ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/merge_tests.rs (target/debug/build/caption-core/126953c38ffc9bdc/out/merge_tests-126953c38ffc9bdc)

running 7 tests
test empty_input_returns_no_words ... ok
test bpe_fragments_joined ... ok
test invalid_timestamps_are_repaired ... ok
test overlapping_words_are_shifted_forward ... ok
test punctuation_attached_and_spacing_starts_words ... ok
test special_tokens_dropped ... ok
test real_sample_fixture_merges_into_expected_word_count ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests caption_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### b. `cargo test` dengan feature whisper

```text
   Compiling caption-core v0.1.0 (/workspaces/caption-maker/caption-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 1.17s
     Running unittests src/lib.rs (target/debug/build/caption-core/8394430a313c9b62/out/caption_core-8394430a313c9b62)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/caption-cli.rs (target/debug/build/caption-core/af980ef2fa6aebbe/out/caption_cli-af980ef2fa6aebbe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/ass_tests.rs (target/debug/build/caption-core/95bfad3f47e57438/out/ass_tests-95bfad3f47e57438)

running 14 tests
test braces_and_newlines_are_escaped ... ok
test dialogue_uses_first_start_and_last_end ... ok
test empty_groups_only_emit_header ... ok
test comma_in_text_is_preserved ... ok
test groups_from_grouping_api_render_in_word_order ... ok
test header_contains_required_sections_and_event_columns ... ok
test highlight_inserts_gap_tag ... ok
test highlight_rounding_does_not_accumulate_drift ... ok
test highlight_uses_exact_cumulative_word_durations ... ok
test plain_mode_has_no_karaoke_tags ... ok
test style_line_maps_colors_to_ass_bgr_order ... ok
test timestamp_format_basic ... ok
test timestamp_rounds_not_truncates ... ok
test real_sample_fixture_matches_golden_ass ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/audio_tests.rs (target/debug/build/caption-core/7b153f573995266a/out/audio_tests-7b153f573995266a)

running 2 tests
test rejects_wav_with_wrong_channel_count_or_sample_rate ... ok
test loads_16khz_mono_i16_as_normalized_f32 ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/group_tests.rs (target/debug/build/caption-core/ba78d125270cd397/out/group_tests-ba78d125270cd397)

running 3 tests
test empty_input_returns_no_groups ... ok
test fewer_words_than_group_size_stays_in_one_group ... ok
test seven_words_are_grouped_with_remainders_for_each_size ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/merge_tests.rs (target/debug/build/caption-core/1d6b83f26dd7c646/out/merge_tests-1d6b83f26dd7c646)

running 7 tests
test bpe_fragments_joined ... ok
test empty_input_returns_no_words ... ok
test invalid_timestamps_are_repaired ... ok
test overlapping_words_are_shifted_forward ... ok
test punctuation_attached_and_spacing_starts_words ... ok
test real_sample_fixture_merges_into_expected_word_count ... ok
test special_tokens_dropped ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests caption_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### c. Clippy

`cargo clippy --all-targets -- -D warnings`

```text
    Checking caption-core v0.1.0 (/workspaces/caption-maker/caption-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.35s
```

`cargo clippy --all-targets --no-default-features -- -D warnings`

```text
    Checking serde v1.0.229
    Checking thiserror v1.0.69
    Checking caption-core v0.1.0 (/workspaces/caption-maker/caption-core)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.88s
```

### d. Perintah CLI dan stdout/stderr

Perintah yang dijalankan:

```sh
cargo run --release --bin caption-cli -- samples/test.wav --model models/ggml-small.bin --words 2 --highlight -o out.ass --dump-tokens tokens.json
```

Output mentah:

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.27
   Compiling glob v0.3.4
   Compiling clang-sys v1.9.1
   Compiling prettyplease v0.2.37
   Compiling syn v2.0.119
   Compiling libc v0.2.191
   Compiling memchr v2.8.3
   Compiling regex-syntax v0.8.12
   Compiling minimal-lexical v0.2.1
   Compiling cfg-if v1.0.5
   Compiling libloading v0.8.9
   Compiling nom v7.1.3
   Compiling regex-automata v0.4.19
   Compiling shlex v2.0.1
   Compiling either v1.19.0
   Compiling find-msvc-tools v0.1.15
   Compiling bindgen v0.71.1
   Compiling cc v1.11.2
   Compiling itertools v0.13.0
   Compiling regex v1.13.2
   Compiling cexpr v0.6.0
   Compiling log v0.4.35
   Compiling shlex v1.3.0
   Compiling bitflags v2.13.2
   Compiling rustc-hash v2.1.3
   Compiling cmake v0.1.58
   Compiling fs_extra v1.3.0
   Compiling syn v3.0.10
   Compiling whisper-rs-sys v0.12.1
   Compiling utf8parse v0.2.2
   Compiling serde_core v1.0.229
   Compiling anstyle-parse v1.0.0
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-query v1.1.5
   Compiling anstyle v1.0.14
   Compiling zmij v1.0.23
   Compiling colorchoice v1.0.5
   Compiling anstream v1.0.0
   Compiling whisper-rs v0.14.2
   Compiling thiserror v1.0.69
   Compiling clap_lex v1.1.2
   Compiling heck v0.5.0
   Compiling serde v1.0.229
   Compiling serde_json v1.0.154
   Compiling strsim v0.11.1
   Compiling clap_builder v4.6.7
   Compiling clap_derive v4.6.8
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v1.0.69
   Compiling itoa v1.0.19
   Compiling clap v4.6.8
   Compiling hound v3.5.1
   Compiling caption-core v0.1.0 (/workspaces/caption-maker/caption-core)
    Finished `release` profile [optimized] target(s) in 1m 24s
     Running `target/release/caption-cli samples/test.wav --model models/ggml-small.bin --words 2 --highlight -o out.ass --dump-tokens tokens.json`
whisper_init_from_file_with_params_no_state: loading model from 'models/ggml-small.bin'
whisper_init_with_params_no_state: use gpu    = 0
whisper_init_with_params_no_state: flash attn = 0
whisper_init_with_params_no_state: gpu_device = 0
whisper_init_with_params_no_state: dtw        = 0
whisper_init_with_params_no_state: devices    = 1
whisper_init_with_params_no_state: backends   = 1
whisper_model_load: loading model
whisper_model_load: n_vocab       = 51865
whisper_model_load: n_audio_ctx   = 1500
whisper_model_load: n_audio_state = 768
whisper_model_load: n_audio_head  = 12
whisper_model_load: n_audio_layer = 12
whisper_model_load: n_text_ctx    = 448
whisper_model_load: n_text_state  = 768
whisper_model_load: n_text_head   = 12
whisper_model_load: n_text_layer  = 12
whisper_model_load: n_mels        = 80
whisper_model_load: ftype         = 1
whisper_model_load: qntvr         = 0
whisper_model_load: type          = 3 (small)
whisper_model_load: adding 1608 extra tokens
whisper_model_load: n_langs       = 99
whisper_model_load:      CPU total size =   487.01 MB
whisper_model_load: model size    =  487.01 MB
whisper_init_state: kv self size  =   18.87 MB
whisper_init_state: kv cross size =   56.62 MB
whisper_init_state: kv pad  size  =    4.72 MB
whisper_init_state: compute buffer (conv)   =   22.41 MB
whisper_init_state: compute buffer (encode) =  128.01 MB
whisper_init_state: compute buffer (cross)  =    6.18 MB
whisper_init_state: compute buffer (decode) =   97.27 MB
tokens=92 words=48 groups=24 output=out.ass
```

## 6. Hasil nyata run Fase 0

- Token mentah: **92**.
- Kata setelah merge: **48**.
- Grup (`--words 2`): **24**.
- Audio: `samples/test.wav`, 23.185125 detik.

20 baris pertama `out.ass`:

```text
[Script Info]
; Script generated by Caption Maker
ScriptType: v4.00+
PlayResX: 1920
PlayResY: 1080
WrapStyle: 0
ScaledBorderAndShadow: yes

[V4+ Styles]
Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding
Style: Default,Arial,64,&H0000FFFF,&H00FFFFFF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3,1,2,40,40,60,1

[Events]
Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text
Dialogue: 0,0:00:00.12,0:00:00.74,Default,,0,0,0,,{\kf24}lagi {\k3}{\kf35}cari
Dialogue: 0,0:00:00.77,0:00:01.94,Default,,0,0,0,,{\kf63}kaution {\kf54}ringan
Dialogue: 0,0:00:02.06,0:00:02.55,Default,,0,0,0,,{\kf21}tapi {\k14}{\kf14}bikin
Dialogue: 0,0:00:02.68,0:00:03.52,Default,,0,0,0,,{\kf36}kulit {\k2}{\kf46}flawless
Dialogue: 0,0:00:03.77,0:00:04.64,Default,,0,0,0,,{\kf19}ini {\kf68}Loistice
Dialogue: 0,0:00:04.79,0:00:05.83,Default,,0,0,0,,{\kf66}Ultimate {\k9}{\kf29}DNA
```

10 token mentah pertama dari `tokens.json`:

```json
{"text": "[_BEG_]", "t0_ms": 0, "t1_ms": 0, "id": 50364, "p": 0.9863177, "is_special": true}
{"text": " lagi", "t0_ms": 120, "t1_ms": 360, "id": 17742, "p": 0.6236981, "is_special": false}
{"text": " car", "t0_ms": 390, "t1_ms": 660, "id": 1032, "p": 0.90209544, "is_special": false}
{"text": "i", "t0_ms": 660, "t1_ms": 740, "id": 72, "p": 0.998302, "is_special": false}
{"text": " k", "t0_ms": 770, "t1_ms": 840, "id": 350, "p": 0.355681, "is_special": false}
{"text": "aut", "t0_ms": 840, "t1_ms": 1080, "id": 1375, "p": 0.26125896, "is_special": false}
{"text": "ion", "t0_ms": 1160, "t1_ms": 1400, "id": 313, "p": 0.27088684, "is_special": false}
{"text": " ring", "t0_ms": 1400, "t1_ms": 1780, "id": 4875, "p": 0.97360873, "is_special": false}
{"text": "an", "t0_ms": 1780, "t1_ms": 1940, "id": 282, "p": 0.9990658, "is_special": false}
{"text": " tapi", "t0_ms": 2060, "t1_ms": 2270, "id": 23901, "p": 0.8324469, "is_special": false}
```

## 7. Deviasi dari spec

1. `TranscribeOptions.on_progress` menggunakan `Option<Arc<dyn Fn(i32) + Send + Sync>>`, bukan `Option<Box<dyn Fn(i32) + Send>>`. `whisper-rs 0.14.2` menyediakan callback aman dengan closure `'static`; `Arc` memungkinkan callback dimiliki closure tersebut tanpa referensi borrowed atau unsafe.
2. Token bytes dikonversi per token memakai `String::from_utf8_lossy`. Skema `RawToken` yang ditentukan menyimpan `String`, bukan byte mentah; bytes UTF-8 yang terbelah antar-token karenanya tidak bisa direpresentasikan persis tanpa mengubah skema. Run ini menghasilkan 0 karakter replacement, tetapi kasus multibyte terbelah tetap perlu ditangani/diuji secara khusus.

## 8. Masalah yang ditemukan / hal yang meragukan

- Run menghasilkan 3 token non-special dengan `t0_ms == t1_ms`: `"it"` pada 3040 ms, `" y"` pada 19670 ms, dan `" r"` pada 20190 ms. Merge memperbaiki waktu tidak valid/berdurasi nol; alignment aslinya tetap meragukan.
- Tidak ada timestamp negatif dan tidak ada karakter U+FFFD pada token mentah run ini.
- Teks hasil Whisper belum diperiksa dengan mendengarkan seluruh audio; ejaan nama/istilah seperti `kaution` dan `Loistice` belum diverifikasi.
- `.ass` belum dibuka di VLC/Aegisub, sehingga rendering visual dan sinkronisasi saat playback belum dinilai.

## 9. Asumsi yang dibuat tanpa konfirmasi

- `samples/test.wav` yang sudah tersedia di workspace adalah audio uji Indonesia yang dimaksud untuk Fase 0.
- Model `ggml-small.bin` digunakan sesuai perintah Tahap D; transkripsi dilakukan offline setelah pengunduhan.
- Output default style ASS (Arial 64, warna kuning/putih, posisi bawah tengah) cukup untuk validasi engine awal.

## 10. Hal yang belum dikerjakan dan saran langkah berikutnya

- Validasi hasil `.ass` dengan playback di VLC/Aegisub dan koreksi kualitas transkripsi/timing berdasarkan audio.
- Tambahkan penanganan byte UTF-8 lintas token BPE (atau bentuk representasi token bytes yang dapat dipulihkan) dan fixture khusus untuk karakter multibyte.
- Pertimbangkan tes integrasi otomatis FFmpeg yang terpisah dari tes transkripsi/model.
- Lanjutkan Fase 2 (CI), lalu Fase 3+ (Tauri/UI/packaging) sesuai rencana proyek; tidak ada pekerjaan Fase 2+ yang dilakukan di sini.

## Tahap 1.1 - Kontrol kata per baris

### Ringkasan

- Kontrol “Kata per baris” dipindahkan keluar dari accordion “Kustomisasi Lanjutan” dan ditempatkan tepat di bawah `#preset-gallery` pada panel kanan.
- Kontrol tetap berupa segmented control dan sekarang menampilkan empat pilihan: 1 Kata, 2 Kata, 3 Kata, dan Kalimat.
- Nilai radio 1, 2, dan 3 dipertahankan beserta nama elemen dan alur JS/API yang sudah ada; opsi Kalimat ditambahkan sebagai nilai UI `4` sementara backend belum mendukungnya.
- Tidak ada perubahan pada `caption-core` atau `src-tauri`; pilihan Kalimat hanya ditampilkan di HTML.

### File yang berubah

- `src/index.html`: 9 baris ditambah dan 8 baris dihapus untuk memindahkan dan menambah opsi pada kontrol.
- `src/css/app.css`: 1 baris diubah (kolom segmented control dari 3 menjadi 4).
- `REPORT.md`: laporan tahap ini ditambahkan di bagian paling bawah.

### ID elemen dan fungsi JS

- ID elemen yang dipakai ulang: `#preset-gallery`, `#highlight-toggle`, `#font-select`, `#font-size`, `#text-color`, `#highlight-color`, `#position-select`, `#outline-size`, dan `#shadow-size`.
- Tidak ada ID elemen baru yang diperlukan untuk kontrol ini.
- Nama radio yang dipertahankan: `words-per-line`.
- Fungsi JS yang dipakai ulang: `updateSettingsFromControls()` di `src/js/main.js` membaca radio terpilih dan membentuk `settings.wordsPerLine`; fungsi tersebut diteruskan oleh `api.js` sebagai `words_per_line` pada `transcribe()` dan `render_ass_from_words()`.

### Status opsi “Kalimat”

- Belum didukung backend. Enum Rust `WordsPerLine` di `caption-core/src/model.rs` hanya memiliki `One`, `Two`, dan `Three`.
- Fungsi `group_words()` di `caption-core/src/group.rs` membagi kata berdasarkan `per_line.get()` dan belum memiliki mode per kalimat.
- `render_ass_impl()` di `src-tauri/src/commands.rs` menerima hanya `Some(1)`, `Some(2)`, `Some(3)`, atau mengembalikan `words_per_line tidak valid` untuk nilai lain.
- Untuk backend, perlu ditambah mode Kalimat pada enum/representasi nilai, penentuan batas kalimat dari teks atau timing, serta penanganan pada `render_ass_impl()` dan jalur transkripsi. API/UI juga perlu memutuskan nilai wire yang baru, misalnya mode enum/string, alih-alih nilai `4`.

### Hal yang tidak yakin atau belum bisa diuji

- Belum menjalankan aplikasi Tauri atau tes visual, sehingga posisi visual empat segmen pada panel kanan belum diverifikasi.
- Belum dijalankan tes atau build penuh; belum dipastikan apakah ada kebutuhan CSS responsif untuk panel sempit.
- Nilai wire `4` baru bersifat sementara hanya untuk mencocokkan bentuk HTML; backend saat ini akan menolaknya dan belum ada implementasi pembatasan kalimat.
- Saya tidak memverifikasi perilaku aplikasi saat opsi Kalimat dipilih di browser/Tauri.

### Hal yang sengaja tidak diubah

- Tidak mengubah `caption-core`, `src-tauri`, `src/js/main.js`, `src/js/api.js`, atau `src/js/preview.js`.
- Tidak mengubah nilai radio yang sudah ada (`1`, `2`, `3`) dan tidak mengubah nama `words-per-line`.
- Tidak mengubah preset, pengaturan lanjutan lain, logika ekspor, atau format payload backend.
- Tidak membuat commit atau perubahan Git.

### Langkah uji manual

1. Jalankan aplikasi Tauri.
2. Pastikan kontrol “Kata per baris” tampil tepat di bawah galeri preset `#preset-gallery`, di luar accordion.
3. Pastikan segmented control menampilkan 1 Kata, 2 Kata, 3 Kata, dan Kalimat dalam satu baris pada lebar panel normal.
4. Pilih 1 Kata, 2 Kata, dan 3 Kata; pastikan pilihan aktif terlihat dan pratinjau/ekspor tetap berjalan seperti sebelumnya.
5. Pilih Kalimat; periksa bahwa opsi dapat dipilih di UI, lalu catat bahwa backend saat ini belum mendukungnya dan dapat mengembalikan error untuk nilai `4`.

## Tahap 1.2 - Mode Kalimat

### Ringkasan

- Mode `Sentence` ditambahkan pada enum Rust `WordsPerLine` dan digunakan oleh `group_words()` untuk membentuk grup berdasarkan tanda baca akhir, jeda waktu, dan batas maksimal delapan kata.
- Grup Kalimat berakhir setelah kata yang berakhiran `.`, `?`, atau `!`, setelah jeda ke kata berikutnya mencapai atau melewati 700 ms, atau saat grup mencapai delapan kata; sisa kata di akhir tetap dibuat satu grup.
- Konstanta `SENTENCE_MAX_WORDS = 8` dan `SENTENCE_MIN_GAP_MS = 700` ditambahkan di atas `group_words()` pada `caption-core/src/group.rs`.
- Nilai radio Kalimat di UI diteruskan sebagai string wire `"sentence"`; backend Tauri membaca string tersebut serta angka `1`, `2`, dan `3` yang tetap dipertahankan.

### File yang berubah

- `caption-core/src/model.rs`: menambah varian `Sentence`.
- `caption-core/src/group.rs`: implementasi pemisahan kalimat dan konstanta baru.
- `caption-core/src/preset_render.rs`: mendukung override `WordsPerLine::Sentence` tanpa mengelompokkan ulang grup kalimat.
- `caption-core/tests/group_tests.rs`: menambah lima skenario mode Kalimat.
- `caption-core/tests/ass_preset_tests.rs`: menyesuaikan tes override ke enum Rust.
- `src-tauri/src/commands.rs`: menambah tipe wire dan menerima mode Kalimat pada transkripsi serta render ASS.
- `src-tauri/src/commands_tests.rs`: menambah tes penerimaan nilai wire `"sentence"`.
- `src/js/main.js`: mengubah nilai radio UI `4` menjadi string `"sentence"`.
- `src/js/preview.js`: menambahkan grouping Kalimat untuk pratinjau browser agar konsisten dengan aturan backend.
- `REPORT.md`: menambahkan laporan tahap ini.

### Nilai wire dan alasannya

- Pilihan: `"sentence"` untuk Kalimat, sedangkan angka `1`, `2`, dan `3` tetap dipakai untuk opsi lama.
- Alasan: bentuk ini tidak mengandalkan nilai numerik `4`, memperjelas semantik mode, dan memerlukan perubahan kode paling kecil pada payload JS yang sudah memakai field `words_per_line`.
- Implementasi wire memakai enum `WordsPerLineWire` yang menerima angka atau string `"sentence"` pada `src-tauri/src/commands.rs`.

### Hasil perintah test dan clippy

- `cargo fmt --check`: lulus setelah format terakhir.
- `cargo test --no-default-features`: lulus. Seluruh suite no-feature lulus, termasuk 8 tes `group_tests.rs` dan 11 tes unit Tauri.
- `cargo clippy --all-targets --no-default-features -- -D warnings`: lulus.
- `cargo test`: belum berhasil dijalankan sepenuhnya karena build `whisper-rs-sys 0.12.1` gagal menemukan `clang.dll`/`libclang.dll`; masalah ini berasal dari lingkungan Windows saat build feature Whisper.
- `cargo clippy --all-targets -- -D warnings`: belum berhasil dijalankan sepenuhnya karena build feature Whisper mengalami masalah `libclang` yang sama.
- Tes awal no-default sebelumnya sempat gagal satu ekspektasi unit yang salah pada jumlah kata; ekspektasi diperbaiki dan jalannya berikutnya lulus.

### Hal yang tidak yakin atau tidak diuji

- Perilaku mode Kalimat tidak dijalankan langsung melalui aplikasi Tauri, tetapi jalur render telah diuji oleh unit test Tauri.
- Jalur transkripsi dengan feature Whisper tidak dijalankan karena lingkungan tidak menyediakan libclang untuk `whisper-rs-sys`.
- Definisi kalimat mengandalkan tanda baca yang sudah menempel pada teks kata hasil `merge_tokens_to_words`; kasus tanda baca terpisah tidak ditambahkan.
- Saya tidak melakukan verifikasi visual output ASS atau playback.

### Hal yang sengaja tidak diubah

- Tidak mengubah validasi preset atau nilai `words_per_line` yang disimpan dalam file preset; preset tetap dibatasi 1 sampai 3.
- Tidak mengubah kontrol atau penataan visual lain di UI, selain pemetaan nilai radio dan pratinjau JS.
- Tidak mengubah komit atau riwayat Git.

### Langkah uji manual

1. Jalankan aplikasi Tauri dengan feature Whisper yang tersedia.
2. Pilih 1 Kata, 2 Kata, dan 3 Kata untuk memastikan payload tetap berupa angka dan hasilnya tidak berubah.
3. Pilih Kalimat, lalu transkripsikan atau ekspor ASS.
4. Pastikan baris baru berakhir setelah `.`, `?`, `!`, jeda sekitar 700 ms atau lebih, atau delapan kata.
5. Jika backend mengirim error atau tidak melakukan pemisahan Kalimat, periksa bahwa payload berisi `"sentence"` pada field `words_per_line`.

## Tahap 1.2b - Perbaikan ekspor

### Hasil diagnosis

- Tombol `#export-ass` di `src/index.html` memanggil `exportSubtitles()` melalui listener di `src/js/main.js`.
- Sebelumnya, `exportSubtitles()` mengambil `result.ass`, membuat `Blob`, membuat URL object, dan mengklik elemen `<a download="caption_simulasi.ass">`. Itu jalur unduhan browser, bukan pemanggilan dialog simpan native; lalu UI menampilkan `"File .ass berhasil diunduh (simulasi)."` walaupun hasil penyimpanan tidak dikonfirmasi.
- `exportAss()` di `src/js/api.js` pada Tauri hanya memanggil command `render_ass_from_words`; command tersebut dirender oleh `render_ass_impl()` di `src-tauri/src/commands.rs` dan mengembalikan string ASS, bukan menulis file.
- `tauri_plugin_dialog` dan `tauri_plugin_fs` sudah terdaftar di `src-tauri/src/lib.rs`. Capability sebelumnya memakai `dialog:default`, yang pada manifest plugin dialog mencakup `allow-open` dan `allow-save`; FS plugin tidak dipakai untuk menulis ekspor.
- Sebelumnya tidak ditemukan command Rust yang menulis file ekspor ASS/SRT/VTT/TXT/JSON. Operasi `std::fs` yang ada di `commands.rs` hanya menangani file WAV sementara dan tes sidecar.
- Penyebab pratinjau “Iniadalahcontoh”: `renderPreview()` di `src/js/preview.js` menyisipkan text node spasi antar-span, tetapi parent `.subtitle-layer` di `src/css/app.css` menggunakan `display: flex`. Text node whitespace-only di dalam flex container tidak dirender, sehingga spasi antar kata hilang. Ini hanya dilaporkan; kode pratinjau tidak diubah.

### File yang berubah

- `src/js/main.js`: mengganti Blob/download browser dengan pemanggilan ekspor, default nama dari file media, dan status sukses/batal/gagal.
- `src/js/api.js`: setelah ASS dirender lewat command Tauri, membuka dialog native dan meneruskan path serta isi ASS ke command tulis Rust.
- `src-tauri/src/commands.rs`: menambahkan `write_export_file` dan implementasi tulis file dengan error eksplisit serta mengembalikan path yang ditulis.
- `src-tauri/src/lib.rs`: mendaftarkan command tulis ekspor.
- `src-tauri/src/commands_tests.rs`: menambah tes penulisan isi file dan path hasil.
- `src-tauri/capabilities/default.json`: mengganti `dialog:default` dengan izin lebih spesifik `dialog:allow-open` dan `dialog:allow-save`, mempertahankan dialog pilih media/model serta mengaktifkan dialog simpan. Tidak menambah izin `fs` karena penulisan dilakukan lewat command Rust, bukan API plugin FS.
- `REPORT.md`: menambahkan laporan tahap ini.

### Izin capability

- Izin aktif: `dialog:allow-open` dan `dialog:allow-save`.
- `dialog:default` sebelumnya telah memberikan keduanya; capability kini menyatakan izin yang dibutuhkan secara eksplisit dan tidak meminta API dialog `message` yang tidak digunakan.
- Tidak ada permission tulis FS plugin yang ditambahkan: command Rust menerima path hasil dialog dan menulis dengan `std::fs::write`.

### Hal yang tidak yakin atau tidak diuji

- `cargo fmt --check`, `cargo test --no-default-features` (111 tes lulus), dan `cargo clippy --all-targets --no-default-features -- -D warnings` berhasil.
- Perintah `node --test src/tests/preview.test.mjs` tidak dapat dijalankan karena executable `node` tidak tersedia di environment ini.
- Belum menjalankan aplikasi Tauri interaktif untuk melihat dialog native pada Windows atau menguji lokasi/path aktual yang dikembalikan plugin dialog.
- Tes command memverifikasi file ditulis ke temp path dan isi/path hasil benar, tetapi tidak menguji izin filesystem OS, overwrite prompt native, atau perilaku cancel dialog.
- Tidak menjalankan transkripsi nyata; konten kata tetap berasal dari jalur simulasi yang sudah ada.
- Diagnosis pratinjau didasarkan pada kombinasi kode JS dan CSS; tampilan langsung tidak diverifikasi.

### Hal yang sengaja tidak diubah

- Tidak mengubah transkripsi mock atau data kata tiruan.
- Tidak memperbaiki pratinjau tanpa spasi, sesuai instruksi hanya melaporkan.
- Tidak menambah plugin/library dan tidak menggunakan Blob atau unduhan browser untuk ekspor.
- Tidak mengubah format ASS, preset, maupun grouping kata yang sudah ada.
- Tidak membuat commit.

### Langkah uji manual

1. Jalankan aplikasi Tauri dan pilih file media bernama misalnya `video-demo.mp4`.
2. Buat subtitle seperti biasa, lalu klik “Ekspor .ass”.
3. Pastikan dialog native “Simpan sebagai” muncul dengan nama awal `video-demo.ass`.
4. Pilih lokasi yang dapat ditulis dan simpan; pastikan progress menampilkan pesan sukses beserta path lengkap.
5. Buka file `.ass` pada path tersebut dan pastikan isinya memuat `[Script Info]` dan event subtitle.
6. Ulangi lalu batalkan dialog; pastikan UI menampilkan ekspor dibatalkan dan tidak menyatakan sukses.
7. Uji simpan ke lokasi yang tidak dapat ditulis atau buat kegagalan filesystem; pastikan pesan gagal ditampilkan, bukan sukses.

## Tahap 1.2d - Kata per baris vs preset

### Diagnosis

- Tipe `StylePreset` di `caption-core/src/preset.rs` menyimpan `words_per_line` sebagai `u8` wajib; validasi hanya menerima 1–3. `PresetSummary` awalnya tidak menyertakan properti tersebut ke UI.
- Semua file preset bawaan menetapkan `words_per_line`, jadi tidak ada preset saat ini yang menyerahkan pilihan ini ke kontrol manual.

| Preset | `words_per_line` | Perilaku |
|---|---:|---|

|---:|---|
| Kata Tebal Putih (`kata-tebal-putih`) | 1 | Dipaksa preset |
| Kata Kotak Merah (`kata-kotak-merah`) | 1 | Dipaksa preset |
| Kata Anton Kuning (`kata-anton-kuning`) | 1 | Dipaksa preset |
| Kata Bangers (`kata-bangers`) | 1 | Dipaksa preset |
| Baris Kotak Hitam (`baris-kotak-hitam`) | 3 | Dipaksa preset |
| Baris Kotak Kuning (`baris-kotak-kuning`) | 3 | Dipaksa preset |
| Baris Neon Mint (`baris-neon-mint`) | 3 | Dipaksa preset |
| Baris Isi Kuning (`baris-isi-kuning`) | 3 | Dipaksa preset |
| Tekan Warna Kuning (`tekan-warna-kuning`) | 2 | Dipaksa preset |
| Tekan Garis Bawah (`tekan-garis-bawah`) | 2 | Dipaksa preset |
| Tekan Besar (`tekan-besar`) | 2 | Dipaksa preset |
| Tekan Ungu Kotak Gelap (`tekan-ungu-kotak-gelap`) | 3 | Dipaksa preset |

- Di pratinjau, `src/js/preview.js` sebelumnya mengelompokkan dengan `settings.wordsPerLine` dari radio tanpa melihat preset.
- Saat ekspor, `exportSubtitles()` sebelumnya tidak mengirim `wordsPerLine`; `render_ass_impl()` di `src-tauri/src/commands.rs` lalu memakai nilai preset. Jika nilai UI dikirim, implementasi Rust saat ini mengutamakan request tersebut sebelum nilai preset. `render_ass_with_preset()` di `caption-core/src/preset_render.rs` juga memakai `PresetOptions.words_per_line` terlebih dahulu, baru fallback ke preset. Jadi akar perbedaan saat ini: pratinjau mengikuti UI, sedangkan ekspor mengikuti preset.
- Jalur transkripsi saat ini tetap mock di `generateSubtitles()` pada `src/js/main.js` dan tidak memanggil `transcribe()`. Walaupun `api.js` mengirim `words_per_line` bila `transcribe()` dipanggil, command Rust mentranskripsi dan mengembalikan daftar kata; grouping untuk tampilan/ASS terjadi kemudian, bukan saat transkripsi.

### Perubahan

- `src-tauri/src/commands.rs`: `PresetSummary` kini menyertakan nilai preset `words_per_line` untuk dikirim ke UI.
- `src/js/api.js`: daftar preset mock browser dilengkapi nilai `words_per_line` yang sama dengan preset bawaan.
- `src/js/main.js`: menambah `effectiveWordsPerLine()` sebagai satu resolver yang dipakai oleh pratinjau dan ekspor; pergantian preset langsung menyinkronkan pilihan radio, status disabled, dan pratinjau.
- `src/index.html`: menambah ID untuk fieldset dan keterangan “Diatur oleh preset ini.”; teks bantuan galeri diperjelas agar menyebut hanya pengaturan yang ditentukan preset.
- `src/css/app.css`: kontrol preset-disabled diredupkan.
- `src-tauri/src/commands_tests.rs`: menegaskan 12 nilai yang dikirim dari daftar preset sesuai data preset.
- `REPORT.md`: menambahkan laporan tahap ini.

### Nilai efektif

- Jika `words_per_line` preset tersedia, nilai itu menjadi nilai efektif: kontrol menampilkan segmen yang cocok, dinonaktifkan, dan menampilkan keterangan.
- Jika properti tidak tersedia, kontrol aktif dan nilai manual yang tersimpan di pengaturan digunakan.
- `effectiveWordsPerLine()` di `src/js/main.js` adalah sumber nilai tunggal untuk pratinjau dan ekspor; ekspor mengirim nilai yang sama ke backend.
- Saat ini semua 12 preset menetapkan angka, sehingga pada galeri saat ini kontrol akan nonaktif untuk setiap preset. Jalur kontrol manual tetap disediakan untuk preset yang kelak tidak memberi nilai.

### Hal yang tidak yakin atau belum diuji

- `cargo fmt --check`, `cargo test --no-default-features` (111 tes lulus), dan `cargo clippy --all-targets --no-default-features -- -D warnings` berhasil; pemeriksaan Problems tidak menemukan error pada file yang diubah.
- Tes JavaScript dan pemeriksaan visual browser/Tauri tidak dijalankan; Node tidak tersedia di environment ini.
- Belum menjalankan aplikasi Tauri/browser secara visual untuk memeriksa keadaan redup, perubahan radio langsung, dan keterangan pada seluruh pilihan preset.
- Tidak ada preset tanpa `words_per_line` saat ini, sehingga jalur kontrol aktif untuk preset yang tidak menentukan nilai belum bisa diverifikasi terhadap data nyata.
- Pratinjau dan ekspor tidak diuji end-to-end dengan satu file media; transkripsi tetap berupa simulasi sebagaimana sebelumnya.

### Hal yang sengaja tidak diubah

- Tidak mengubah isi atau nilai `words_per_line` dari file preset.
- Tidak mengubah backend/jalur transkripsi mock.
- Tidak mengubah aturan grouping kalimat atau format ASS.
- Tidak membuat commit.

### Langkah uji manual

1. Jalankan aplikasi dan pilih masing-masing preset; pastikan fieldset “Kata per baris” nonaktif dan nilai yang tersorot cocok dengan preset.
2. Pastikan keterangan “Diatur oleh preset ini.” tampil saat preset aktif dan penjelasan galeri tidak lagi mengklaim semua pengaturan manual diabaikan.
3. Buat subtitle, geser waktu pratinjau, lalu ekspor `.ass`; periksa jumlah kata per grup di pratinjau dan ASS cocok dengan nilai preset.
4. Jika nanti ada preset tanpa `words_per_line`, pastikan fieldset aktif, pilihan manual dapat diubah, dan pratinjau/ekspor mengikuti nilai itu.

## Tahap 1.2c - Pratinjau: spasi dan gaya preset

### Hasil diagnosis

- Sebelum perubahan, `renderPreview()` di `src/js/preview.js` mengambil font, ukuran, warna isi, ukuran garis tepi, warna highlight, dan bayangan dari `settings` (nilai kontrol/default `state.js`), bukan dari preset. Warna garis tepi bahkan ditetapkan tetap `#000`.
- CSS `.subtitle-layer` memberi fallback system font, teks putih, ukuran 20px, dan bobot 700; style inline dari JS kemudian mengganti sebagian nilai tersebut dengan settings manual.
- Di ekspor ASS, `caption-core/src/preset_render.rs` memakai `preset.colors.text`, `preset.colors.outline`, `preset.outline`, `preset.box_`, serta `preset.font.uppercase`. Keberadaan `box_` dipetakan menjadi ASS `BorderStyle=3`; teks uppercase diterapkan saat renderer menyusun setiap kata.
- `PresetSummary` sebelumnya mengirim `id`, `name`, `category`, dan `words_per_line` saja. Mock di `src/js/api.js` juga hanya berisi metadata tersebut. Data `font`, `colors`, `outline`, `shadow`, dan `box` belum sampai ke UI.
- Spasi sebelumnya ditambahkan sebagai text node whitespace di antara kata, tetapi `.subtitle-layer` adalah flex container. Flex mengabaikan flex item whitespace-only. Kini spasi visual dibuat dengan CSS `gap` berbasis `em`.

### File yang berubah

- `src/js/preview.js`: menerapkan metadata preset untuk font, ukuran, warna teks/highlight/outline, bayangan, kotak latar, dan uppercase.
- `src/css/app.css`: membuat layer pratinjau block dan menambah wrapper kata flex yang membungkus dengan jarak berbasis `em`.
- `src-tauri/src/commands.rs`: memperluas `PresetSummary` agar menyertakan `font`, `colors`, `outline`, `shadow`, dan `box`.
- `src/js/api.js`: melengkapi data mock browser dengan metadata gaya yang sama untuk 12 preset.
- `src-tauri/src/commands_tests.rs`: menambahkan assertion bahwa metadata Anton Kuning dan preset berkotak sampai pada summary.
- `REPORT.md`: menambahkan laporan tahap ini.

### Data preset yang sekarang dikirim ke UI

- Field baru: `font` (termasuk `family`, `size`, `bold`, `italic`, `uppercase`, dan `letter_spacing`), `colors` (termasuk text/highlight/outline/shadow), `outline`, `shadow`, dan `box` (warna serta padding atau `null`).
- `font.file` ikut tersedia dari tipe Rust, tetapi pratinjau tidak memuat file font tersebut; browser memakai family preset bila tersedia, lalu fallback ke system font.

### Gaya yang sengaja tidak ditiru penuh

- Tidak memuat font file, jadi tampilan family dapat berbeda bila font preset tidak terpasang.
- `box` ASS ditampilkan sebagai CSS background dengan padding; ukuran/layout tidak dijamin identik dengan renderer ASS atau skala video.
- Efek preset lanjutan seperti glow dan variasi emphasis ASS (misalnya underline/scale khusus) tidak direplikasi. Pratinjau tetap memakai progress highlight karaoke yang sudah ada, kini memakai warna dari preset.

### Hal yang tidak yakin atau tidak diuji

- `cargo fmt --check`, `cargo test --no-default-features` (111 tes lulus), dan `cargo clippy --all-targets --no-default-features -- -D warnings` berhasil; Problems tidak menemukan error pada file yang diubah.
- Tidak menjalankan Node atau tes `src/tests/preview.test.mjs` sesuai instruksi; tes yang ada hanya menguji grouping kata, dan grouping tidak diubah.
- Tidak menjalankan aplikasi Tauri/browser secara visual, sehingga wrapping, keterbacaan outline, ukuran font preset, dan kotak pada berbagai resolusi belum diverifikasi.
- Pemeriksaan Rust compile/test mencakup serialisasi/list preset, tetapi tidak membuktikan hasil pixel pratinjau.

### Hal yang sengaja tidak diubah

- Tidak mengubah file/nilai preset, format ASS, renderer ASS, backend transkripsi mock, atau aturan grouping.
- Tidak menambah dependency/library.
- Tidak membuat commit.

### Langkah uji manual

1. Jalankan aplikasi dan tampilkan kata multi-kata; pastikan ada jarak antar kata dan baris tetap membungkus rata tengah.
2. Pilih “Kata Anton Kuning”; pastikan isi kuning, outline gelap, family/ukuran preset semampunya, dan teks kapital.
3. Pilih “Baris Kotak Kuning” atau “Kata Kotak Merah”; pastikan background kotak dan padding mengikuti preset.
4. Pilih preset dengan highlight (misalnya “Tekan Warna Kuning”); putar/geser waktu dan pastikan warna progres highlight mengikuti palet preset.
5. Bandingkan dengan ASS ekspor; perbedaan font file, glow, dan efek emphasis lanjutan memang tidak dijanjikan identik.

## Tahap 2.0 - Diagnosis jalur video

### 1. FFmpeg di aplikasi desktop

- **Fakta - konfigurasi Tauri:** `src-tauri/tauri.conf.json` mengatur `bundle.externalBin` ke `binaries/ffmpeg` dan `binaries/ffprobe`. `src-tauri/binaries/README.md` menjelaskan nama target Windows x64 dan bahwa `.exe` tidak dilacak Git.
- **Fakta - dev lokal:** pada workspace saat diperiksa, `src-tauri/binaries/ffmpeg-x86_64-pc-windows-msvc.exe` dan pasangan `ffprobe` tersedia. Keduanya diabaikan Git. Salinannya juga ada di `target/debug/ffmpeg.exe` dan `target/debug/ffprobe.exe`; SHA-256 masing-masing cocok dengan nilai yang dipatok di workflow Windows. `sidecar_path()` di `src-tauri/src/commands.rs` mencari `ffmpeg.exe`/`ffprobe.exe` di sebelah executable aplikasi, lalu fallback ke nama polos yang dicari lewat `PATH`. Tidak ada `ffmpeg` atau `ffprobe` di `PATH` pada sesi pemeriksaan ini (`where.exe` tidak menemukannya).
- **Fakta - installer CI:** `.github/workflows/build-windows.yml` mengunduh FFmpeg tag `9.1` dari `GyanD/codexffmpeg` (`*essentials_build.zip`), menyalin executable ke nama target Tauri, memeriksa SHA-256, lalu menjalankan `cargo tauri build`. Jadi CI tidak mengompilasi FFmpeg sendiri. Workflow menyatakan paket Gyan GPL menyertakan x264 dan libass.
- **Fakta - validasi binary lokal:** binary sidecar yang diperiksa melaporkan FFmpeg `9.1-essentials_build-www.gyan.dev`; `-buildconf` menampilkan `--enable-libass` dan `--enable-libx264`, sedangkan `-filters` menampilkan filter `ass` dan `subtitles`. Hash FFmpeg cocok dengan `ED8F66CB877087A96F5BBB40A2944F0E764F2AD8870D5CB647360D912E62BC07` yang dipatok CI. Ini membuktikan binary lokal yang cocok hash mendukung filter tersebut; belum membuktikan installer CI yang sudah terpasang dijalankan end-to-end.
- **Izin shell:** tidak ada `tauri-plugin-shell` di `src-tauri/Cargo.toml`/`src-tauri/src/lib.rs` dan tidak ada izin shell di `src-tauri/capabilities/default.json`. FFmpeg dipanggil dari Rust, bukan melalui plugin shell Tauri.
- **Cara verifikasi di Windows:** jalankan `.\src-tauri\binaries\ffmpeg-x86_64-pc-windows-msvc.exe -hide_banner -buildconf` dan cari `--enable-libass`, lalu `.\src-tauri\binaries\ffmpeg-x86_64-pc-windows-msvc.exe -hide_banner -filters` dan pastikan `ass` serta `subtitles` terdaftar. Untuk binary hasil instalasi, jalankan perintah yang sama pada `ffmpeg.exe` di folder aplikasi; bandingkan SHA-256 dengan hash di `.github/workflows/build-windows.yml`.

### 2. Burn-in di Rust

- **Fakta - argumen:** `build_burn_args()` di `caption-core/src/video.rs` membuat argumen FFmpeg dengan `-vf ass=sub.ass:fontsdir=fonts`, `-c:v libx264`, `-pix_fmt yuv420p`, `-movflags +faststart`, dan `-progress pipe:1 -nostats`. Kualitas `Cepat` memakai CRF 23/`veryfast`, sedangkan `Bagus` memakai CRF 18/`slow`. Ekstensi path output menentukan muxer; fungsi tidak menetapkan `-f mp4`, sehingga caller perlu memberi path `.mp4` untuk keluaran MP4.
- **Fakta - ukuran, frame rate, dan audio:** tidak ada argumen skala/resolusi atau frame-rate (`-s`/`-r`); video diproses pada ukuran dan frame rate sumber, tanpa pemaksaan nilai baru. Audio AAC disalin, audio dengan codec lain dienkode AAC 192 kb/s, dan bila `ffprobe` tidak menemukan audio, argumen audio tidak ditambahkan.
- **Fakta - path Windows:** filter tidak memasukkan path `.ass` absolut. `setup_work_dir()` menulis `sub.ass` dan direktori `fonts/`, kemudian `run_burn()` di `caption-core/src/bin/caption-cli.rs` mengatur `current_dir` ke work directory. Path input/output menjadi argumen terpisah pada `Command`, bukan string shell. Dengan begitu path seperti `G:\Folder dengan spasi\video.mp4` tidak perlu di-escape sebagai bagian filter; path relatif filter menghindari masalah titik dua/backslash pada drive Windows.
- **Fakta - pemanggil dan progress:** burn-in hanya dipanggil oleh `run_burn()` di CLI `caption-core/src/bin/caption-cli.rs`. Fungsi itu membaca `-progress pipe:1` dan mencetak persentase ke stderr; ia memeriksa status proses dan mengembalikan error jika spawn/wait gagal atau FFmpeg keluar dengan status gagal. Belum ada command Tauri burn-in di `src-tauri/src/lib.rs`/`src-tauri/src/commands.rs`, jadi progress belum dikirim ke UI dan belum ada pembatalan burn-in dari aplikasi.
- **Fakta - tes:** `caption-core/tests/video_tests.rs` menguji argumen kualitas, penanganan audio, filter ASS relatif, tidak adanya flag ukuran/FPS, serta parsing progress. Tes ini tidak menjalankan proses FFmpeg penuh. Pada kegagalan FFmpeg, direktori kerja sementara tidak dibersihkan oleh jalur sukses yang ada, sehingga berkas sementara dapat tertinggal.

### 3. Pratinjau video

- **Fakta:** `src/index.html` memiliki `<video id="video-player">` dan `<audio id="audio-player">`; panel juga memiliki elemen subtitle terpisah `#preview-subtitle` dan label `SIMULASI`. `loadLocalFile()` di `src/js/main.js` memakai `convertFileSrc(path)` untuk path dari dialog Tauri dan Blob URL untuk objek `File` di browser. Ekstensi `mp4`, `mkv`, dan `mov` dianggap video; file audio ditangani oleh elemen audio.
- **Fakta:** `src-tauri/tauri.conf.json` mengaktifkan asset protocol dengan scope `["**"]` dan menetapkan `csp` ke `null`. Konfigurasi ini mendukung pemuatan path lokal melalui `convertFileSrc`; tidak tampak pembatasan CSP khusus yang menghalangi media pada konfigurasi saat ini.
- **Belum dipastikan:** aplikasi tidak memeriksa codec video/audio sebelum memuat media. Pemutaran bergantung pada codec yang tersedia di WebView2/Windows. Tidak ada dasar di kode untuk menyatakan hanya H.264/AAC yang bisa diputar; H.264/AAC umum digunakan, tetapi dukungan aktual perlu diuji pada Windows/WebView2 target. MKV/MOV adalah container dan dapat berisi codec yang tidak didukung.

### 4. Font

- **Fakta:** keluarga font yang dipakai preset di `caption-core/presets/*.json` adalah Poppins, Poppins ExtraBold, Anton, dan Bangers. File `Poppins-Bold.ttf`, `Poppins-ExtraBold.ttf`, `Anton-Regular.ttf`, dan `Bangers-Regular.ttf` serta lisensi OFL-nya ada di `caption-core/assets/fonts/` dan dilacak Git.
- **Fakta:** `BUNDLED_FONTS` di `caption-core/src/video.rs` menyertakan TTF dengan `include_bytes!`; `setup_work_dir()` menulisnya ke subfolder `fonts/`. Filter ASS menggunakan `fontsdir=fonts` relatif ke work directory. Jadi CLI menyediakan font bagi libass tanpa mengandalkan font tersebut terpasang di sistem. Jalur ini belum dipanggil dari Tauri karena command burn-in belum ada.

### 5. Rencana usulan (belum diimplementasikan)

1. **Pemutar video:** pertahankan elemen `<video>` dan alur `convertFileSrc()` yang sudah ada, lalu uji pemutaran, seek, error, serta format/codec pada WebView2 Windows target; ubah overlay CSS menjadi mode yang jelas saat video hasil burn-in diputar. **Risiko:** dukungan codec bervariasi dan pemutaran file lokal besar bisa menambah penggunaan memori atau gagal karena codec/container.
2. **Command burn-in `.mp4`:** tambahkan command Tauri yang menggunakan `build_burn_args()` dan `setup_work_dir()` yang ada; simpan output melalui dialog, jalankan FFmpeg sebagai child process, kirim progress terukur ke UI, dan rancang pembatalan eksplisit beserta pembersihan file sementara pada sukses/gagal/batal. **Risiko:** proses panjang, penanganan lifecycle child process, disk penuh, izin/path Windows, error codec, serta keluaran parsial saat pembatalan. Perlu memastikan implementasi desktop memakai binary sidecar yang sama dan melaporkan error nyata.
3. **Klip pratinjau 5-10 detik:** setelah burn-in dasar bekerja, gunakan pipeline dan preset ASS yang sama untuk membuat klip temporer beresolusi rendah dari rentang waktu yang dipilih, lalu tampilkan melalui pemutar lokal yang sama; bersihkan klip lama dan cegah hasil preset sebelumnya tampil setelah preset berubah. **Risiko:** klip tetap perlu waktu encode, scaling dapat mengubah ukuran teks relatif, hasil dapat berbeda dari ekspor penuh, dan pekerjaan lama dapat selesai setelah pilihan preset berubah.

### Batas pemeriksaan

- Pemeriksaan binary dilakukan langsung terhadap sidecar lokal dan pencocokan hash; saya tidak membuat atau memasang installer pada sesi ini, dan tidak menguji burn-in end-to-end dengan video nyata.
- Aplikasi Tauri/WebView2 tidak dijalankan secara interaktif. Karena itu dukungan codec aktual, playback path dari instalasi MSI/NSIS, dan perilaku progress/pembatalan UI belum diuji.
- Tidak ada source code aplikasi yang diubah untuk tahap diagnosis ini; bagian ini hanya ditambahkan ke `REPORT.md`. Tidak ada commit.

## Tahap 2.1 - Command burn-in

### Ringkasan

- **Fakta:** ditambahkan command Tauri `burn_video` yang menerima input media, daftar kata, preset ID, `words_per_line` efektif (angka 1/2/3 atau `"sentence"`), path output, dan kualitas `"cepat"`/`"bagus"`. Command memakai `render_ass_impl()`, `setup_work_dir()`, dan `build_burn_args()` yang sudah ada, serta mengukur progress FFmpeg dari `-progress pipe:1`.
- **Fakta:** command `cancel_burn` menghentikan satu pekerjaan burn yang aktif dan menunggu cleanup; pekerjaan burn kedua ditolak selama pekerjaan pertama belum selesai. Progress dikirim melalui event `burn-progress` dengan `progress` 0–100 atau `null`, flag `indeterminate`, dan `status`.
- **Fakta:** output diwajibkan berekstensi `.mp4` (case-insensitive). FFmpeg sudah memiliki `-y` di `build_burn_args()`, jadi tidak ada perubahan pada argumen overwrite; proses berjalan dengan stdin tertutup dan tanpa jendela konsol Windows. FFmpeg dan ffprobe diperoleh dari sidecar, bukan dicari lewat PATH.
- **Fakta:** tidak ada dependency atau plugin baru. UI yang disentuh hanya wrapper `burnVideo()`/`cancelBurn()` dan subscription event di `src/js/api.js`; HTML, CSS, dan `main.js` tidak diubah pada tahap ini.

### Command, event, dan file yang berubah

- Command Tauri: `burn_video` dan `cancel_burn`, didaftarkan di `src-tauri/src/lib.rs`.
- Event progress: `burn-progress`; payload memuat `progress`, `indeterminate`, dan `status` (`running`, `completed`, `failed`, atau `cancelled`).
- Wrapper JS: `burnVideo(options, onProgress)` dan `cancelBurn()` di `src/js/api.js`.
- File implementasi tahap ini:
  - `src-tauri/src/commands.rs`: request burn, validasi ekstensi, penggunaan sidecar/ffprobe, progress, status pekerjaan tunggal, pembatalan, pengumpulan tail stderr, dan cleanup output parsial.
  - `src-tauri/src/lib.rs`: state `BurnManager` dan registrasi command.
  - `src/js/api.js`: pembungkus invoke dan listener event.
  - `src-tauri/src/commands_tests.rs`: tes unit dan tes integrasi `#[ignore]`.
  - `caption-core/src/video.rs`: `WorkDirGuard`; durasi yang tidak tersedia menjadi `0.0` agar progress dapat tak-tentu dan `percent()` aman untuk nilai non-finite.
  - `caption-core/src/lib.rs`: ekspor `WorkDirGuard`.
  - `caption-core/src/bin/caption-cli.rs`: burn dan render-frame memakai guard untuk menghapus workdir pada sukses/gagal.
  - `caption-core/tests/video_tests.rs`: tes `-y`, durasi tidak diketahui, progress non-finite, dan pembersihan direktori.
  - `REPORT.md`: laporan tahap ini.

### Pembatalan, progress, dan pembersihan

- `burn_video` mengeksekusi pekerjaan blocking di `tauri::async_runtime::spawn_blocking`, sehingga proses FFmpeg tidak berjalan di thread async UI. `BurnManager` menjaga maksimal satu `BurnJob` aktif.
- `cancel_burn` mengirim kill ke child FFmpeg jika proses sudah berjalan, lalu menunggu pekerjaan burn menyelesaikan pembersihan. Jika dibatalkan sebelum child dibuat, flag pembatalan diperiksa sebelum proses dijalankan.
- Reader stdout mem-parsing `out_time_us`/`out_time_ms` dengan `parse_progress_line()` dan menghitung persentase terhadap durasi dari ffprobe. Jika durasi hilang/tidak valid, payload berjalan adalah `progress: null` dan `indeterminate: true`, bukan pembagian dengan nol.
- FFmpeg berjalan dengan `stdin(Stdio::null())`; pada Windows command diberi `CREATE_NO_WINDOW`. Jika FFmpeg gagal, error menyertakan hingga 12 baris terakhir stderr.
- `WorkDirGuard` menghapus folder `sub.ass` dan font pada jalur sukses maupun error/drop; cleanup eksplisit melaporkan kegagalan hapus. Jika burn gagal atau dibatalkan setelah child dimulai, file output parsial juga dihapus. Jalur CLI burn dan render-frame kini memakai guard yang sama.
- `-y` memang sudah ada sebelum tahap ini. Karena itu FFmpeg tidak meminta konfirmasi overwrite dari stdin. Ini mengasumsikan UI/dialog simpan sudah mendapat persetujuan overwrite sebelum memanggil command.

### Hasil tes

- **Lulus:** `cargo fmt --all -- --check`.
- **Lulus:** `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`.
- **Lulus:** `cargo test --workspace --no-default-features` — 117 tes lulus, 1 tes integrasi di-ignore pada run normal.
- **Lulus saat dijalankan eksplisit:** `cargo test -p caption-tauri --lib --no-default-features burn_video_sidecar_integration -- --ignored --nocapture` — tes burn sidecar nyata lulus. Tes membuat video `testsrc` 1080x1920 selama 2 detik, merender preset ASS, menghasilkan `.mp4`, lalu memastikan output terbaca oleh sidecar ffprobe dengan dimensi dan durasi yang benar.
- **Tidak lulus karena environment:** `cargo test --workspace` dengan fitur default berhenti saat build `whisper-rs-sys`; build script tidak menemukan executable CMake (`is cmake not installed?`). CMake tidak tersedia di PATH atau lokasi Visual Studio yang diperiksa, jadi suite fitur default belum terverifikasi.
- Node tidak tersedia di environment; wrapper `src/js/api.js` tidak diuji dengan test runner JavaScript.

### Pemeriksaan tambahan rasio video (laporan saja)

ASS yang sama dibuat dengan `PlayResX: 1080` dan `PlayResY: 1920`, preset `Kata Anton Kuning`, lalu dipakai pada dua video `testsrc` 2 detik:

| Input | Ukuran output video | Ukuran file MP4 | Ukuran PNG frame | Hasil pengamatan |
|---|---:|---:|---:|---|
| Vertikal 1080x1920 | 1080x1920 | 49,997 byte | 1080x1920, 105,254 byte | Teks kuning ber-outline hitam tampak jelas, terpusat horizontal dan berada di area bawah sesuai alignment preset. |
| Horizontal 1920x1080 | 1920x1080 | 57,879 byte | 1920x1080, 109,959 byte | Teks tetap tampak di area bawah dan terpusat horizontal, tetapi jauh lebih kecil relatif terhadap frame; skala/layout tidak konsisten dengan output vertikal karena `PlayRes` ASS portrait dipakai untuk video landscape. |

- PNG frame pada detik sekitar 1:
  - `C:\Users\Alto\.copilot\session-state\b6a299df-e35c-4cd5-acbb-f8fb66397d3c\files\ratio-check\vertical-frame.png`
  - `C:\Users\Alto\.copilot\session-state\b6a299df-e35c-4cd5-acbb-f8fb66397d3c\files\ratio-check\horizontal-frame.png`
- Kedua PNG berhasil dibuka dan diperiksa secara visual. Ini uji diagnostik tambahan dengan ASS portrait yang sengaja sama, bukan perubahan kode atau uji UI Tauri. Command burn aplikasi memakai dimensi hasil ffprobe untuk membuat ASS, jadi kasus aplikasi normal akan menghasilkan PlayRes sesuai input.

### Hal yang tidak yakin atau belum diuji

- **Fakta:** tes integrasi menjalankan helper pipeline langsung dengan sidecar Windows dan memverifikasi output lewat ffprobe. **Belum diuji:** invoke command melalui runtime Tauri aktif, pengiriman event ke WebView, atau perilaku UI.
- Tes unit memverifikasi penolakan burn kedua saat satu pekerjaan aktif, tetapi mekanisme kill/cancel dan penghapusan output parsial belum diuji secara otomatis. Belum ada tes integrasi yang membatalkan proses FFmpeg aktif; dialog/pemanggil UI juga belum tersedia pada tahap ini.
- Jalur kompilasi default belum diuji karena CMake tidak tersedia untuk `whisper-rs-sys`. Tes tanpa default features dan Clippy no-default berhasil.
- Perintah pengujian rasio memakai file sintetis, bukan footage nyata. Pengamatan posisi/ukuran hanya berlaku pada ASS portrait yang sama untuk kedua rasio.
- Jika OS menolak penghapusan file/direktori (misalnya file sedang dikunci atau izin tidak cukup), cleanup mengembalikan error dan guard mencoba menghapus ulang saat drop; penghapusan tetap bergantung pada izin filesystem.

### Hal yang sengaja tidak diubah

- Tidak mengubah isi preset, format ASS, grouping/transkripsi mock, maupun `build_burn_args()` untuk opsi selain menggunakan `-y` yang sudah tersedia.
- Tidak mengubah `src/index.html`, `src/css/app.css`, atau `src/js/main.js`; integrasi tombol/dialog dan UI progress/cancel belum termasuk tahap backend ini.
- Tidak menambah dependency/plugin dan tidak membuat commit.

### Langkah uji manual

1. Jalankan aplikasi Tauri desktop dengan sidecar FFmpeg/ffprobe berada di samping executable aplikasi.
2. Dari console/invoke atau pemanggil tahap UI selanjutnya, panggil `burnVideo()` dengan path media yang valid, daftar kata, preset ID, nilai `wordsPerLine` angka atau `"sentence"`, path berakhiran `.mp4`, dan kualitas `"cepat"` atau `"bagus"`.
3. Pastikan listener menerima `burn-progress` yang bergerak, lalu menerima status `completed`; buka hasil `.mp4` dan verifikasi dimensinya serta caption.
4. Saat burn berlangsung, panggil `cancelBurn()`; pastikan command selesai setelah proses berhenti, status menjadi `cancelled`, output parsial hilang, dan tidak ada folder `caption-burn-*` yang tertinggal.
5. Coba output `.ass` atau tanpa ekstensi; pastikan command menolak dengan pesan ekstensi yang jelas.
6. Mulai burn kedua ketika burn pertama berjalan; pastikan ditolak dengan pesan bahwa burn lain sedang berjalan.
7. Pilih output yang sudah ada melalui dialog dengan persetujuan overwrite, lalu pastikan proses tidak menunggu input terminal karena FFmpeg memakai `-y` dan stdin tertutup.

## Tahap 2.2 - UI Buat Video

### Ringkasan

- **Fakta:** footer kini memiliki tombol utama `Buat Video`, sedangkan `Ekspor .ass` menjadi tombol sekunder. Tombol `Batalkan` menggunakan `cancelBurn()` saat render video aktif dan tetap membatalkan simulasi pembuatan subtitle saat controller simulasi aktif.
- **Fakta:** jika subtitle belum ada, alur video memanggil fungsi `generateSubtitles()` yang sama dengan tombol `Buat Subtitle`; daftar kata/mock tidak diduplikasi atau diubah. Setelah itu UI membuka dialog simpan `.mp4` dengan nama default dari nama media.
- **Fakta:** UI memanggil wrapper `burnVideo()` dengan media, kata, preset aktif, `effectiveWordsPerLine()`, dan kualitas `"cepat"`. Event `burn-progress` memperbarui persentase footer; payload indeterminate menampilkan teks `Merender video: Memproses...`.
- **Fakta:** setelah sukses, path hasil dikonversi memakai `convertFileSrc()` lalu dimuat ke `<video id="video-player">`. Overlay `#preview-subtitle` dan label `SIMULASI` disembunyikan saat hasil diputar. Tombol `Lihat video asli`, pemilihan media baru, atau pergantian preset mengembalikan preview sumber.

### File yang berubah

- `src/index.html`: tombol `Buat Video`, tombol `Ekspor .ass` sekunder, dan tombol `Lihat video asli`.
- `src/css/app.css`: ukuran tombol utama/preview serta aturan agar overlay benar-benar tersembunyi ketika memakai atribut `hidden`.
- `src/js/main.js`: alur pembuatan video, progress, status sibuk, pembatalan, pemutaran hasil, dan pemulihan media sumber.
- `REPORT.md`: laporan tahap ini.

### Status tombol selama proses

- Saat persiapan, dialog simpan, dan burn, `Buat Video`, `Ekspor .ass`, `Buat Subtitle`, `Pilih File`, hapus media, serta tombol-tombol preset dinonaktifkan. Ini juga mencegah klik ganda dan perubahan media/preset di tengah proses.
- Setelah backend mengirim progress dengan status `running`, `Batalkan` aktif sampai ada permintaan cancel atau status terminal. Saat simulasi pembuatan subtitle berjalan sebelum burn, `Batalkan` membatalkan controller simulasi yang sama.
- Setelah proses berakhir (sukses, error, atau batal), `finally` menghapus status sibuk dan memanggil `updateAvailability()` untuk mengembalikan keadaan kontrol sesuai media/subtitle yang ada.
- Jika dialog simpan ditutup tanpa memilih path, tidak ada command burn yang dipanggil dan footer menampilkan `Pembuatan video dibatalkan.` Jika tidak ada media, formatnya audio, atau file tidak memiliki path lokal Tauri, UI menampilkan pesan yang sesuai.

### Pemeriksaan PlayRes (laporan saja)

- **Fakta dari kode:** `burn_video_impl()` di `src-tauri/src/commands.rs` memanggil `probe_video(&input, &ffmpeg)` untuk memperoleh `info`. Fungsi itu menjalankan ffprobe dan mem-parsing dimensi video.
- **Fakta dari kode:** `RenderRequest` yang kemudian diberikan ke `render_ass_impl()` mengisi `width: info.width` dan `height: info.height`; renderer ASS membentuk `PlayResX`/`PlayResY` dari geometri tersebut. Nilainya bukan 1080x1920 tetap. Jadi burn video horizontal memperoleh PlayRes horizontal dari dimensi yang diprobe.
- Backend tidak diubah pada Tahap 2.2. Pemeriksaan ini berdasarkan pembacaan kode, bukan pengukuran ASS hasil invoke UI.

### Hal yang tidak yakin atau belum diuji

- Node tidak dijalankan sesuai instruksi, sehingga tidak ada pemeriksaan sintaks atau tes JavaScript otomatis. Tes `src/tests/preview.test.mjs` tidak diubah karena logika preview/grouping tidak diubah.
- Aplikasi Tauri tidak dijalankan secara interaktif. Dialog simpan, event progress di WebView, disable/enable kontrol, pemutaran video hasil, serta pemulihan video sumber/audio belum diverifikasi langsung melalui UI.
- Burn backend dan tes sidecar telah diverifikasi pada Tahap 2.1, tetapi alur lengkap dari klik UI hingga hasil diputar belum diuji end-to-end.
- Jika file berhasil ditulis tetapi WebView tidak dapat mengonversi atau memutar path, UI tetap menyatakan path tersimpan dan memberi keterangan bahwa preview tidak dapat dimuat/tidak dapat diputar.
- Uji manual perlu memastikan pengguna menyetujui overwrite di dialog simpan untuk path yang sudah ada; command backend memakai `-y` sesuai keputusan sebelumnya.

### Hal yang sengaja tidak diubah

- Tidak mengubah command/backend `burn_video` atau `cancel_burn`, format ASS, preset, dan data transkripsi mock.
- Tidak menambah dependency/library. Tidak mengubah pengaturan kualitas menjadi kontrol UI; kualitas burn ditetapkan `"cepat"` sesuai permintaan.
- Tidak mengubah alur ekspor `.ass` selain menjadikannya tombol sekunder.
- Tidak membuat commit.

### Langkah uji manual

1. Jalankan aplikasi Tauri di Windows, pilih video melalui `Pilih File`, lalu klik `Buat Video` sebelum membuat subtitle. Pastikan simulasi subtitle berjalan melalui jalur yang sama, kemudian dialog simpan muncul dengan nama `<nama-media>.mp4`.
2. Batalkan dialog simpan; pastikan footer menyatakan dibatalkan dan tidak ada output baru.
3. Ulangi, pilih lokasi output, dan amati footer. Pastikan persentase berubah mengikuti progress nyata; bila backend mengirim progress indeterminate, teks menunjukkan `Merender video: Memproses...`.
4. Selama burn, pastikan tombol buat video/ekspor/subtitle/pilih media dan preset tidak bisa digunakan, sementara `Batalkan` bisa digunakan. Batalkan dan pastikan tidak ada pesan sukses atau file parsial yang tertinggal.
5. Setelah sukses, pastikan footer menampilkan `Video disimpan: <path lengkap>`, hasil otomatis tampil di player, overlay subtitle dan label `SIMULASI` hilang, dan video dapat diputar.
6. Klik `Lihat video asli`; pastikan media sumber dan overlay simulasi kembali. Ulangi dengan preset berbeda dan pastikan pergantian preset kembali ke sumber; pilih media baru dan pastikan mode hasil burn sebelumnya tidak tertinggal.
7. Coba tanpa media, dengan media audio, dan dengan file yang di-drop tanpa path lokal; pastikan masing-masing mendapat pesan jelas, bukan error runtime.

## Tahap 3.0 - Diagnosis Whisper

### 1. Command transcribe dan wrapper API

- **Fakta — input/output:** command `transcribe` di `src-tauri/src/commands.rs` menerima `TranscribeRequest` berisi `input`, `model`, `language`, `threads`, `prompt`, dan `words_per_line` (semuanya `Option`). Implementasi ber-feature Whisper mewajibkan `input` dan `model`; default bahasa dari `TranscribeOptions` adalah `"id"`, default thread dihitung dari CPU, dan prompt opsional diteruskan ke Whisper. Hasilnya `Vec<WordDto>` dengan `text`, `start_ms`, `end_ms`, bukan `RawToken` mentah. `words_per_line` saat ini belum dipakai command transcribe.
- **Fakta — feature build:** `src-tauri/Cargo.toml` menonaktifkan default feature dependency (`caption-core` memakai `default-features = false`) dan `caption-tauri` memiliki `default = []`; feature `whisper` baru mengaktifkan `caption-core/whisper`. Jadi build Tauri biasa tanpa `--features caption-tauri/whisper` memakai command stub yang selalu mengembalikan error “fitur whisper tidak aktif”. Workflow installer Windows memang eksplisit membangun dengan `cargo tauri build --features caption-tauri/whisper` (`.github/workflows/build-windows.yml`). `caption-core` sendiri memiliki Whisper sebagai default, tetapi itu tidak mengaktifkannya untuk dependency Tauri yang default-features-nya dimatikan.
- **Fakta — ekstraksi audio:** `transcribe` memakai `sidecar_path("ffmpeg")` dan `caption_core::extract_audio()` (`caption-core/src/audio.rs`). Argumen FFmpeg menghilangkan video dan menghasilkan WAV PCM signed 16-bit, mono, 16 kHz (`-vn -ac 1 -ar 16000 -c:a pcm_s16le`). WAV sementara dibuat di direktori temp OS dengan nama unik, lalu dibaca seluruhnya sebagai `Vec<f32>` oleh `load_wav_16k_mono()`.
- **Fakta — progress:** command mengirim event `transcribe_progress` pada awal/akhir ekstraksi dan awal/akhir transkripsi; belum ada progress persen nyata dari Whisper. `caption-core/src/whisper.rs` menyediakan `set_progress_callback_safe`, tetapi `transcribe` Tauri tidak mengisi `TranscribeOptions.on_progress`, sehingga callback itu tidak tersambung.
- **Fakta — pembatalan dan cleanup:** belum ada command cancel transkripsi. `transcribe()` di `src/js/api.js` memasang listener dan melepasnya di `finally`, tetapi `AbortSignal` hanya digunakan jalur mock browser, bukan untuk menghentikan `invoke("transcribe")`. WAV dihapus dengan `remove_file` best-effort hanya setelah berhasil dibaca; bila ekstraksi atau pembacaan WAV gagal, cleanup tidak terjamin. Sebaliknya, CLI memakai `TemporaryWav` dengan `Drop` di `caption-core/src/bin/caption-cli.rs`.
- **Fakta — pemanggilan:** command terdaftar di `src-tauri/src/lib.rs`. Wrapper JS mengirim input/model/bahasa/thread/prompt/words-per-line dan menerima array `WordDto`; jika bukan Tauri, wrapper mensimulasikan progress lalu mengambil `src/mock/words.json`.

### 2. Model dan UI Impor

- **Fakta:** `list_models_impl()` di `src-tauri/src/commands.rs` mengembalikan daftar statis Base (`installed: false`), Small (`true`), dan Medium (`false`); nilai Small tidak mengecek apakah berkas model benar-benar ada. `loadModels()` di `src/js/main.js` memakai daftar itu untuk opsi UI dan mengaktifkan transkripsi jika suatu model bertanda terpasang. Pesan UI menyebut Small tersedia “dalam simulasi”.
- **Fakta:** tombol Impor memanggil dialog file `.bin` di `src/js/api.js`, lalu `import_model` di `commands.rs` hanya memeriksa `path.ends_with(".bin")` dan mengembalikan path yang sama. Tidak ada pemeriksaan file ada/terbaca/formatnya benar, penyalinan, pembaruan daftar model, atau penyimpanan model. UI desktop menerima string path dari command tetapi `importSelectedModel()` membaca `file.name`; untuk hasil string itu nilai nama berkas tidak tersedia, sehingga status impor dapat menampilkan `undefined`.
- **Fakta:** tidak ditemukan penggunaan `app_data_dir`, folder model aplikasi, atau resolver model di kode saat ini. Model yang dipilih UI bernilai ID seperti `"small"`, sedangkan command transcribe menafsirkan `request.model` langsung sebagai `PathBuf`; model tidak ditemukan berdasarkan ID. Dokumentasi lama menyebut `models/ggml-small.bin` relatif ke workspace/CLI dan menegaskan model diabaikan Git.
- **Usulan, belum diimplementasikan:** untuk aplikasi desktop simpan model di subfolder khusus dalam Tauri app data directory (mis. `app_data_dir()/models`), daftarkan model berdasarkan berkas nyata di folder itu, dan ubah impor agar menyalin secara aman sebelum menandainya tersedia. Validasi minimal: ekstensi tanpa case sensitivity, berkas ada dan bisa dibaca, ukuran masuk akal, lalu validasi parser Whisper saat membuka model; gunakan penulisan sementara + rename agar impor terputus tidak meninggalkan model parsial. Hindari menganggap ekstensi `.bin` sebagai validasi format.

### 3. Mock UI dan aliran kata

- **Fakta:** `generateSubtitles()` di `src/js/main.js` tidak memanggil `transcribe()` dari `api.js`. Ia menunggu timer satu detik, membuat enam `dummyWords` tetap, menyimpan kata-kata itu di `state.words`, lalu menandai `generated: true`. `AbortController` yang ada hanya membatalkan timer mock. Jadi ada mock di dua tempat: fungsi UI memakai enam kata contoh, sedangkan wrapper API browser memakai fixture `src/mock/words.json`.
- **Fakta:** jalur backend nyata menghasilkan Whisper `RawToken` berisi teks, bytes, timestamp, token ID, probabilitas, dan penanda special (`caption-core/src/whisper.rs`). `merge_tokens_to_words()` (`caption-core/src/merge.rs`) menggabungkannya menjadi `Word`; command mengubahnya menjadi DTO `{text, start_ms, end_ms}`. Format tiga field yang sama dipakai kata di UI/fixture.
- **Fakta:** `renderPreviewText()` meneruskan `state.words` ke preview; `exportSubtitles()` meneruskan kata yang sama ke `exportAss()`; `createVideo()` jika belum ada hasil memanggil `generateSubtitles()` lalu meneruskan `state.words` ke `burnVideo()`. Ekspor ASS dan burn mengirim DTO kata itu ke command Rust. Dengan demikian titik penggantian utama adalah isi `generateSubtitles()` agar memanggil wrapper `transcribe()` dan menyimpan DTO hasilnya; preview, ASS, dan burn tidak memerlukan perubahan format kata.

### 4. Performa pada mesin ini

- **Fakta — percobaan saat ini:** `samples/test.wav` ada dan durasinya 23,185 detik berdasarkan ukuran data WAV/header (sesuai durasi 23,185125 detik yang tercatat di laporan lama). Folder `models/` dan `models/ggml-small.bin` tidak ada pada workspace ini; pencarian `ggml-*.bin` tidak menemukan file. Saya mencoba menjalankan `target/debug/caption-cli.exe samples/test.wav --model models/ggml-small.bin`; CLI berhenti dengan `failed to open 'models\\ggml-small.bin'` / `Failed to create a new whisper context`. Karena model hilang, waktu transkripsi aktual dan thread yang benar-benar dipakai pada percobaan ini tidak dapat diukur.
- **Fakta — thread yang dikonfigurasi:** `TranscribeOptions::new()` memilih `min(available_parallelism(), 8)`; `whisper.rs` mengirim nilainya ke `set_n_threads()`. PowerShell saat pemeriksaan melaporkan 4 logical processor, tetapi nilai Rust `available_parallelism()` tidak tercatat karena model gagal dibuka. Dua pengukuran lama di workspace mencatat 2 thread; itu bukan pengukuran ulang pada percobaan sekarang.
- **Fakta historis, bukan hasil ukur baru:** `REPORT_2.md` mencatat `audio=23.185s transcribe=17.974s rtf=0.775 threads=2`, sedangkan `REPORT_3A.md` mencatat `audio=23.185s transcribe=60.883s rtf=2.626 threads=2`. Angka ini saling berbeda besar dan berasal dari run terdahulu; saat ini tidak dapat dipastikan perbedaan kondisi yang menyebabkannya.
- **Perkiraan kasar:** ekstrapolasi linear ke 60 detik audio memberi sekitar 46,5 detik berdasarkan RTF 0,775 atau 157,6 detik berdasarkan RTF 2,626. Ini hanya perkiraan waktu tahap transkripsi dari dua catatan lama; bukan benchmark mesin saat ini dan tidak termasuk ekstraksi audio FFmpeg atau biaya lain. Variasi hasil yang besar membuat satu angka prediksi tidak dapat diandalkan.

### 5. Risiko UX yang terlihat

- **Model belum ada:** daftar statis tetap menandai Small terpasang walau file tidak ada, sehingga tombol dapat tampak aktif tetapi pembukaan model gagal. Impor saat ini tidak memasang atau mendaftarkan file.
- **Video tanpa audio:** ekstraksi FFmpeg meminta stream audio; jika video tidak punya audio, FFmpeg akan gagal dan command mengembalikan error ekstraksi. Belum ada pemeriksaan awal atau jalur transkripsi video-only.
- **Video panjang:** seluruh audio diubah menjadi array float 16 kHz di memori dan tidak ada batas durasi. Whisper bekerja di CPU; model small pada catatan terdahulu memerlukan sekitar 18–61 detik untuk audio 23,185 detik, bergantung kondisi. Command Tauri diberi `async`, tetapi operasi beratnya sinkron dan belum dipindah ke blocking worker, sehingga dapat mengganggu respons runtime/UI.
- **Bahasa selain Indonesia:** default model/UI adalah `"id"` dan belum ada kontrol bahasa di UI. Backend menerima kode bahasa, tetapi pilihan bahasa yang salah berisiko menurunkan akurasi.
- **Pembatalan:** tombol batal sekarang hanya mengaborsi timer mock. Saat Tauri transcribe dipakai, wrapper tidak dapat membatalkan `invoke`; ekstraksi menggunakan proses FFmpeg tanpa handle proses untuk cancel dan transkripsi belum memiliki jalur abort yang terhubung. Pengguna dapat menunggu lama tanpa cara menghentikan proses dengan andal.

### 6. Rencana usulan (tanpa kode)

1. **Manajemen model:** tetapkan app data `models/` sebagai lokasi desktop; ubah daftar model agar dibangun dari file yang benar-benar ada; impor ke file sementara, validasi keberadaan/akses dan buka model melalui Whisper untuk validasi format sebelum rename/registrasi. Mulai dari impor lokal; unduh dapat menjadi langkah terpisah dengan konfirmasi ukuran, sumber, lisensi, dan kegagalan jaringan. **Risiko:** model besar menghabiskan ruang dan penyalinan/validasi dapat lama; validasi dengan membuka model membutuhkan memori tambahan.
2. **Command transcribe:** aktifkan feature Whisper pada konfigurasi development dan build yang dituju; selesaikan model ID menjadi path app-data; jalankan ekstraksi dan inferensi pada blocking worker; gunakan callback progress Whisper serta progress ekstraksi; tambahkan state satu tugas dan command cancel yang menghentikan FFmpeg serta menghentikan inferensi bila API Whisper mendukung abort, dengan cleanup WAV memakai guard. **Risiko:** pembatalan saat transisi FFmpeg/Whisper perlu sinkronisasi; progress FFmpeg dan Whisper memiliki skala/tahap berbeda; build Whisper menambah waktu build dan penggunaan CPU/memori.
3. **Hubungkan UI:** ganti timer dan `dummyWords` di `generateSubtitles()` dengan `transcribe()` memakai path media dan model terpasang; sambungkan progress/cancel UI, tampilkan error eksplisit, dan simpan hasil `WordDto` ke `state.words`. Pertahankan format kata yang sudah dipakai. **Risiko:** model ID/path harus konsisten; input tanpa audio, import yang gagal, serta durasi/bahasa harus ditangani sebelum menandai hasil generated. Setelah bentuk DTO cocok, preview, render ASS, burn video, dan ekspor tidak perlu diubah.

### Hal yang tidak yakin atau belum diuji

- **Fakta:** benchmark baru gagal sebelum transkripsi karena model tidak tersedia; tidak ada base/tiny/medium model di folder proyek untuk diukur. Riwayat mencatat hanya small, dengan dua hasil waktu yang berbeda.
- **Belum diuji:** membuka model pada UI, impor model nyata, bahasa selain Indonesia, video tanpa audio, video panjang, progress Whisper yang nyata, pembatalan proses, dan transkripsi dari awal hingga ekspor/burn melalui UI.
- Tidak ada kode aplikasi yang diubah pada tahap diagnosis ini; hanya bagian laporan ini ditambahkan. Tidak ada model yang diunduh dan tidak ada commit.

## Tahap 3.1 - Manajemen model

### Ringkasan

- **Fakta:** daftar model desktop sekarang memeriksa file yang benar-benar ada, menampilkan katalog Tiny/Base/Small/Medium serta model `ggml-*.bin` lain. Impor memvalidasi nama, ekstensi, ukuran, dan magic header sebelum menyalin model melalui file sementara `.part` dan menggantinya dengan rename.
- **Fakta:** model ditempatkan di subfolder `models` dalam Tauri `app_data_dir()`. UI menampilkan path folder, memuat ulang daftar setelah impor berhasil, dan memilih model yang baru diimpor.
- **Fakta:** command resolver `resolve_model_path()` dapat menerjemahkan ID ke file terpasang dan memberi pesan bila model belum ada. Command transcribe dan alur `generateSubtitles()` tetap belum disambungkan ke Whisper; itu ditunda ke tahap berikut.

### Commands dan lokasi model

- `list_models`: kini mengembalikan id, nama tampilan, nama file, ukuran file aktual (`size_bytes`, null jika belum terpasang), perkiraan ukuran katalog (`estimated_size_bytes`), dan status `installed`.
- `models_directory`: command baru untuk memberikan path direktori model kepada UI.
- `import_model`: kini menerima path sumber dan mengembalikan `ModelInfo` hasil impor; penyalinan/validasi file dijalankan dengan `tauri::async_runtime::spawn_blocking`.
- **Fakta — Windows:** folder dibentuk dari `app.path().app_data_dir().join("models")`; dengan identifier `com.captionmaker.desktop` di `src-tauri/tauri.conf.json`, lokasi Windows umumnya `%APPDATA%\\com.captionmaker.desktop\\models`. Path aktual ditampilkan UI di bawah dropdown dan path itulah yang menjadi acuan bila tata letak platform berbeda.
- Katalog menunjukkan estimasi kasar Tiny 75 MiB, Base 142 MiB, Small 466 MiB, dan Medium 1530 MiB; jika file terpasang, UI menampilkan ukuran nyata dari metadata. Untuk model lain yang ditemukan, ditampilkan ukuran nyata dan ID/nama diambil dari nama file.

### Validasi format Whisper

- **Fakta — sumber:** dependency terkunci adalah `whisper-rs-sys 0.12.1`. Pada sumber lokal `C:\\Users\\Alto\\.cargo\\registry\\src\\index.crates.io-1949cf8c6b5b557f\\whisper-rs-sys-0.12.1\\whisper.cpp\\ggml\\include\\ggml.h`, `GGML_FILE_MAGIC` didefinisikan sebagai `0x67676d6c` (`"ggml"`). Di `whisper.cpp/src/whisper.cpp`, fungsi `whisper_model_load()` membaca `uint32_t magic` lalu menolak jika nilainya tidak sama dengan `GGML_FILE_MAGIC`.
- **Fakta:** validasi membaca empat byte awal `lmgg`, yakni representasi little-endian dari nilai magic di atas pada target Windows x86/x64, tanpa menebak nilai dari nama ekstensi. Pesan error menyebut nilai magic dan byte yang ditemukan.
- Impor juga mensyaratkan file bisa dibuka/dibaca, ekstensi `.bin` tidak peka huruf besar-kecil, pola nama `ggml-<nama>.bin`, serta ukuran lebih dari 20 MiB. ID dibatasi karakter aman untuk mencegah path traversal; nama tujuan disimpan dalam bentuk `ggml-<id-lowercase>.bin`.
- Salinan ditulis di folder tujuan ke nama unik berakhiran `.part`, di-flush dengan `sync_all()`, lalu dipasang memakai `std::fs::rename()`. Guard `Drop` mencoba menghapus `.part` pada jalur sukses maupun error; rename juga menggantikan model bernama sama.

### File yang berubah

- `src-tauri/src/commands.rs`: katalog dan scan filesystem, lokasi app-data, validasi/impor model, resolver, dan command `models_directory`.
- `src-tauri/src/lib.rs`: mendaftarkan command `models_directory`.
- `src-tauri/src/commands_tests.rs`: tes katalog, impor, penggantian file, cleanup `.part`, dan resolver.
- `src/js/api.js`: metadata katalog mock browser dan wrapper `modelsDirectory()`.
- `src/js/main.js`: tampilan dropdown/status/path serta reload dan pemilihan otomatis setelah impor.
- `src/index.html`: teks awal status model dan baris lokasi model.
- `REPORT.md`: laporan tahap ini.
- **Fakta:** tidak ada dependency/library baru dan tidak ada model besar yang diunduh.

### Hasil verifikasi

- **PASS:** `cargo fmt --all -- --check`.
- **PASS:** `cargo test -p caption-tauri --lib` — 23 lulus, 0 gagal, 1 tes integrasi burn yang sudah ada di-ignore.
- **PASS:** `cargo clippy -p caption-tauri --lib -- -D warnings`.
- **PASS:** `git diff --check` pada file tahap ini.
- Tes validasi memakai file kecil/sparse sintetis; tidak membutuhkan model Whisper berukuran ratusan MB. Tes penggantian nama file sama lulus pada Windows di mesin ini.

### Hal yang tidak yakin atau belum diuji

- **Belum diuji:** dialog pemilih file dan tampilan UI di aplikasi Tauri interaktif; Node tidak dijalankan sesuai instruksi.
- Tes menerima header magic yang benar dan ukuran yang cukup, tetapi tidak membuka seluruh struktur tensor/model Whisper. Model yang rusak setelah header tetap dapat lolos impor dan baru gagal ketika dibuka oleh Whisper pada tahap transkripsi.
- Kegagalan I/O di tengah salin (mis. disk penuh), beberapa impor bersamaan dengan ID sama, dan file model sedang digunakan ketika diganti belum diuji secara langsung.
- Verifikasi tes penggantian berdasarkan `std::fs::rename()` pada Windows di mesin ini; perilaku ketika file tujuan dikunci oleh proses lain belum dipastikan.

### Hal yang sengaja tidak diubah

- Command `transcribe`, `generateSubtitles()`, format kata, preview, preset, burn video, dan ekspor tidak diubah fungsinya. Tombol Buat Subtitle/Buat Video masih menggunakan mock saat ini.
- Tidak ada pengunduhan model, progress impor, penghapusan model, atau tes yang memuat model Whisper nyata.
- Tidak ada dependency baru dan tidak ada git commit.

### Langkah uji manual

1. Jalankan aplikasi Tauri dan lihat daftar model. Tanpa file model, Tiny/Base/Small/Medium harus berstatus belum terpasang, tombol transkripsi tetap mengikuti status terpasang, dan lokasi model tampil di bawah status.
2. Impor file yang bukan `.bin`, nama bukan `ggml-<nama>.bin`, ukuran 20 MiB atau kurang, atau header bukan `lmgg`; pastikan masing-masing menampilkan error yang sesuai dan tidak meninggalkan `.part`.
3. Impor model Whisper valid yang lebih dari 20 MiB. Pastikan model tersalin ke folder yang ditampilkan UI, ukuran dan status terpasang diperbarui, lalu model tersebut otomatis terpilih.
4. Impor ulang model dengan ID/nama file sama dan pastikan file lama diganti, tidak ada `.part` tersisa, serta hanya ada satu entri model.
5. Tutup dan buka kembali aplikasi; pastikan daftar tetap berdasarkan file yang tersimpan di folder models.

## Tahap 3.2 - Command transkripsi

### Ringkasan

- **Fakta:** command Tauri `transcribe` sekarang menerima ID model, menyelesaikannya melalui `resolve_model_path()`, memeriksa stream audio dengan ffprobe, mengekstrak WAV 16 kHz mono, lalu menjalankan Whisper dan mengubah RawToken menjadi format Word yang sudah dipakai UI/backend.
- **Fakta:** ekstraksi dan inferensi berjalan di `spawn_blocking`; `TranscribeManager` menolak pekerjaan kedua. Command `cancel_transcribe` menghentikan FFmpeg ketika ekstraksi berlangsung dan mengirim abort flag yang dibaca callback Whisper saat inferensi.
- **Fakta:** event `transcribe_progress` membawa `stage`, `progress`, `status`; event sukses juga berisi `elapsed_ms`, `audio_duration_ms`, dan jumlah `threads`. Wrapper API memasang listener dan selalu melepasnya di `finally`.
- **Fakta:** `generateSubtitles()` di `main.js` sengaja tidak diubah; tombol UI masih menghasilkan kata mock sampai Tahap 3.3.

### Feature Whisper

- **Pilihan:** `src-tauri/Cargo.toml` kini menetapkan `default = ["whisper"]`, yang mengaktifkan feature `caption-tauri/whisper` pada `cargo tauri dev` dan `cargo tauri build` tanpa flag manual. CI tetap menjalankan `cargo tauri build --features caption-tauri/whisper`; flag itu sekarang redundan tetapi tetap valid.
- `cargo test -p caption-tauri --lib --no-default-features` dan Clippy no-default tetap dapat dijalankan untuk memastikan command stub/build tanpa Whisper tetap berfungsi.

### Progress, pembatalan, dan cleanup

- Event bernama `transcribe_progress` dipertahankan. Tahap ekstraksi memetakan progress FFmpeg dari 0 sampai 10 berdasarkan durasi ffprobe; model dimuat pada status 10; callback progress Whisper dipetakan dari 10 sampai 100. Terminal status memakai `completed`, `failed`, atau `cancelled`.
- **Fakta dari source lokal:** whisper-rs 0.14.2 memiliki `set_abort_callback_safe`. Namun, percobaan benchmark awal yang memakai wrapper callback safe berakhir dengan access violation setelah model dimuat; pemeriksaan implementasinya menunjukkan pengelolaan pointer callback wrapper tersebut tidak aman untuk dipakai di sini. Perubahan akhirnya memakai callback ABI `set_abort_callback` yang membaca `AtomicBool` melalui pointer stabil; Arc pemilik flag tetap hidup sampai inferensi kembali. Callback progress juga memakai callback ABI dengan data `Mutex`/`Arc` yang tetap hidup sepanjang inferensi.
- **Batas pembatalan:** ekstraksi FFmpeg dapat dihentikan dengan `Child::kill()`. Inferensi Whisper dapat meminta abort pada titik pemeriksaan internal callback; operasi tidak dihentikan secara paksa dari thread lain. Pembatalan saat pemuatan model sebelum callback inferensi pertama mungkin menunggu sampai Whisper mencapai titik callback. Uji benchmark sukses, tetapi pembatalan live belum dijalankan manual.
- `TemporaryWav` memakai `Drop` sebagai fallback cleanup dan menghapus file segera setelah WAV dimuat. Jika penghapusan eksplisit gagal, command mengembalikan error; jika cleanup Drop gagal pada jalur error lain, kegagalan dicatat ke stderr.
- Video tanpa stream audio ditolak sebelum ekstraksi dengan pesan `Video tidak memiliki audio.`. Error proses FFmpeg menyertakan tail stderr; error Whisper menyarankan impor ulang model dan menyertakan detail.

### Benchmark nyata

- **Fakta — perintah:** `CAPTION_TEST_MODEL=C:\Users\Alto\Downloads\ggml-small.bin cargo test -p caption-tauri --lib transcribe_whisper_integration -- --ignored --nocapture`.
- **Fakta — hasil run final:** `elapsed_ms=24528`, `audio_duration_ms=23185` (23,185 detik), `rtf=1.058`, `threads=4`, `words=48`; tes integrasi lulus dan waktu mencakup ffprobe, ekstraksi FFmpeg, pemuatan model, inferensi, serta merge kata, bukan kompilasi.
- Thread berasal dari `TranscribeOptions::new()` yang memakai `min(available_parallelism, 8)`; pada run ini hasil konfigurasi aktualnya 4.
- **Fakta:** ada beberapa run sukses pada mesin yang sama dengan elapsed 23,913 ms, 26,455 ms, dan 24,528 ms (run final); variasi ini menunjukkan waktu dipengaruhi kondisi mesin.
- Percobaan awal integrasi sempat gagal karena path ffprobe turunan untuk nama sidecar berarsitektur tidak cocok, lalu sempat access violation saat memakai callback safe. Tes diperbaiki agar memberikan kedua sidecar secara eksplisit, callback ABI diganti, dan benchmark final berhasil. Kegagalan awal tidak mengubah hasil run final yang dilaporkan.

### File yang berubah

- `src-tauri/Cargo.toml`: Whisper menjadi feature default.
- `src-tauri/src/commands.rs`: job manager, command `transcribe`/`cancel_transcribe`, ffprobe audio, ekstraksi yang bisa dihentikan, progress, timing, dan guard WAV.
- `src-tauri/src/lib.rs`: mendaftarkan manager dan `cancel_transcribe`.
- `caption-core/src/model.rs`: opsi abort atomic untuk transkripsi.
- `caption-core/src/whisper.rs`: callback progress dan abort Whisper dengan data callback berumur terjamin.
- `src-tauri/src/commands_tests.rs`: tes progress, tanpa audio, manager, cleanup, dan integrasi Whisper ter-ignore.
- `src/js/api.js`: signature `transcribe(input, modelId, language, onProgress)` dan wrapper `cancelTranscribe()`; mock browser tetap membaca fixture dan dapat menerima `options.signal` opsional.
- `REPORT.md`: laporan tahap ini.
- Tidak ada dependency baru.

### Hasil tes dan hal yang belum diuji

- **PASS:** `cargo test -p caption-tauri --lib` — 27 lulus, 0 gagal, 2 ignored (burn sidecar integration yang sudah ada dan tes transkripsi yang diminta untuk dijalankan eksplisit).
- **PASS:** `cargo test -p caption-tauri --lib --no-default-features` — 26 lulus, 0 gagal, 1 ignored.
- **PASS:** `cargo clippy -p caption-tauri --all-targets -- -D warnings`.
- **PASS:** `cargo clippy -p caption-tauri --all-targets --no-default-features -- -D warnings`.
- **PASS:** tes `transcribe_whisper_integration` dijalankan eksplisit dengan model pada path yang diminta; hasil benchmark tercatat di atas.
- **Belum diuji:** panggilan Tauri dari WebView, event progress yang tampil di UI, pembatalan FFmpeg/Whisper secara live, berkas video tanpa audio sintetis lewat ffprobe end-to-end, dan impor model melalui UI.
- Node tidak dijalankan sesuai instruksi. Waktu benchmark dapat berubah akibat beban CPU dan filesystem; hasil final adalah satu run.

### Hal yang sengaja tidak diubah

- `generateSubtitles()` dan UI `main.js` tidak diubah; belum ada integrasi kata nyata ke tombol Buat Subtitle/Buat Video.
- Tidak mengubah burn video, preset, ASS, preview, format Word/WordDto, dan ekspor.
- Workflow CI tetap memakai flag feature Whisper eksplisit karena sudah benar; tidak ada perubahan workflow atau dependency.
- Tidak ada git commit.

### Langkah uji manual

1. Jalankan `cargo tauri dev` tanpa flag dan pastikan build menyertakan Whisper.
2. Pastikan model ada di folder `models` app-data melalui impor Tahap 3.1, pilih media ber-audio, lalu panggil command `transcribe` dengan `model` berisi ID (mis. `"small"`). Periksa event `transcribe_progress` dari ekstraksi hingga status `completed`, termasuk durasi, elapsed, dan thread.
3. Coba ID model yang belum ada; pastikan error meminta impor model. Coba video tanpa audio; pastikan pesan `Video tidak memiliki audio.` dan WAV sementara tidak tertinggal.
4. Mulai transkripsi kedua saat yang pertama berjalan; pastikan command kedua ditolak dengan pesan transkripsi lain sedang berjalan.
5. Batalkan saat FFmpeg mengekstrak dan saat Whisper inferensi berjalan; pastikan status akhir `cancelled`, command mengembalikan error pembatalan, serta WAV sementara terhapus. Perhatikan bahwa pembatalan saat model baru dimuat dapat menunggu callback Whisper pertama.
6. Jalankan `cargo tauri build` tanpa flag manual; workflow CI tetap dapat memakai flag eksplisit yang sekarang redundan.

## Tahap 3.3 - Transkripsi nyata di UI

### Ringkasan

- **Fakta:** tombol Buat Subtitle kini memakai `transcribe(media.path, modelId, "id", onProgress, { signal })`. Di desktop, validasi media video, path lokal, dan model terpasang dilakukan sebelum command dipanggil; jalur browser tetap memakai mock dari `api.js`.
- **Fakta:** hasil baru disimpan ke `state.words` hanya setelah transkripsi sukses dan berisi kata. Hasil kosong tidak menandai subtitle sebagai generated. Pembatalan tidak mengganti kata yang sudah ada dan tidak lagi membuat kata dummy.
- **Fakta:** Buat Video memakai `generateSubtitles()` yang sama bila belum ada kata, dan alurnya berhenti jika transkripsi gagal, kosong, atau dibatalkan.
- **Fakta:** progress aktual mengikuti event `transcribe_progress`, termasuk tahap dan persen. Wrapper menampilkan status indeterminate singkat sebelum command Tauri dikirim agar tombol Batalkan desktop aktif setelah command siap; listener event tetap dilepas di `finally`.
- **Fakta:** saat transkripsi, file/media, preset, aksi subtitle/video/ekspor, impor model, dan dropdown model dikunci. Batalkan tetap tersedia setelah pekerjaan backend berjalan; browser mock tetap dapat dibatalkan dengan `AbortController`.
- **Fakta:** mengganti media atau model menghapus kata lama. UI desktop menampilkan `PRATINJAU` dan keterangan overlay CSS, sedangkan mode browser/mock tetap menampilkan `SIMULASI`.

### Tinjauan keamanan umur callback Whisper

- **Fakta dari source lokal:** di `caption-core/src/whisper.rs`, `transcribe()` membuat `progress_callback` sebagai `Arc<Mutex<ProgressCallback>>`, memberikan pointer non-owning dari `Arc::as_ptr()` ke `set_progress_callback_user_data()`, lalu memanggil `state.full(params, samples)`. Variabel `progress_callback` tetap berada dalam scope sampai pemanggilan `full` kembali, termasuk ketika `full` mengembalikan error.
- **Fakta dari source lokal:** pointer abort berasal dari `Arc::as_ptr(on_abort)`. Arc itu dimiliki `options.on_abort`, dan referensi `options` tetap hidup melewati `state.full()`. Di `src-tauri/src/commands.rs`, `run_transcription()` membuat options lokal dengan clone `job.cancel_requested`; job dan lease juga tetap hidup selama pekerjaan `spawn_blocking` berlangsung.
- **Penilaian:** pada pemanggilan Whisper sinkron yang terlihat di source whisper-rs 0.14.2 (`WhisperState::full()` memanggil `whisper_full_with_state()` dan baru kembali setelah hasil native tersedia), pointer progress dan abort tidak dibebaskan sebelum inferensi kembali, baik pada sukses maupun error. Implementasi memakai `Arc::as_ptr()` tanpa mengubah hitungan referensi; tidak ada `Arc::from_raw()` yang dapat membebaskan owner lebih awal.
- **Risiko tersisa:** jaminan ini bergantung pada callback native hanya dipanggil selama `whisper_full_with_state()` berjalan sinkron dan tidak disimpan/dipanggil setelah fungsi kembali. Source wrapper menunjukkan pemanggilan sinkron, tetapi tidak ada sanitizer atau uji stress khusus yang membuktikan perilaku setiap jalur native. Callback memakai ABI `extern "C"` tanpa `catch_unwind`; panic dari callback tidak aman untuk melintasi batas C. Callback progress juga melewatkan laporan jika mutex poison.

### File yang berubah

- `src/js/main.js`: transkripsi nyata, validasi awal, progress/pesan hasil, pembatalan, penguncian kontrol, dan invalidasi kata saat media/model berubah.
- `src/js/api.js`: status persiapan indeterminate sebelum invoke desktop; listener progress dan pembungkus pembatalan yang sudah ada tetap digunakan, mock browser tidak diubah.
- `src/index.html`: teks awal desktop untuk model/lencana pratinjau dan id disclaimer untuk memilih keterangan desktop atau browser.
- `REPORT.md`: laporan tahap ini.
- Tidak ada perubahan backend, dependency, preview renderer, burn, preset, ASS, atau format kata.

### Hal yang tidak yakin atau belum diuji

- **Belum diuji:** menjalankan aplikasi Tauri/WebView, transkripsi video nyata dari tombol UI, update persen visual, pembatalan live saat FFmpeg/Whisper, hasil 0 kata, serta alur Buat Video yang melanjutkan setelah transkripsi UI.
- Node tidak dijalankan sesuai instruksi; tes JavaScript tidak dijalankan. Pemeriksaan memori callback di atas adalah tinjauan source, bukan hasil sanitizer atau uji konkurensi.
- **Fakta:** API browser tetap memakai fixture mock. Waktu pada mode browser tidak memiliki metadata elapsed backend, sehingga UI menghitung durasi dari waktu lokal; desktop memakai `elapsed_ms` dari event completed.

### Hal yang sengaja tidak diubah

- Tidak mengubah command `transcribe`, `cancel_transcribe`, Whisper inference, atau format `WordDto`.
- Tidak mengubah `preview.js`, burn-in, ASS, preset, atau CSS overlay; overlay kata nyata tetap perkiraan CSS.
- Tidak menghapus mock browser dan tidak mengubah file fixture mock.
- Tidak menjalankan git commit.

### Langkah uji manual

1. Jalankan aplikasi desktop, pilih video melalui tombol Pilih File, pilih model berstatus terpasang, lalu tekan Buat Subtitle. Pastikan progress menunjukkan ekstraksi audio, pemuatan model, dan transkripsi dengan persen, lalu menampilkan jumlah kata serta lama proses.
2. Selama transkripsi, pastikan Pilih File, preset, Buat Subtitle, Buat Video, Ekspor .ass, Impor model, dan dropdown model nonaktif; Batalkan aktif setelah command mulai berjalan.
3. Batalkan saat ekstraksi dan saat inferensi. Pastikan pesan `Dibatalkan`, UI kembali aktif, serta tidak ada kata dummy atau hasil parsial baru.
4. Uji tanpa media, dengan file audio, dengan media tanpa path lokal (mis. drag-drop), dan tanpa model terpasang. Pastikan pesan jelas muncul tanpa command transkripsi dipanggil; pesan model harus `Impor model Whisper dulu (tombol Impor)`.
5. Uji video tanpa audio dan model rusak. Pastikan error backend tampil dan UI kembali normal.
6. Ganti media setelah subtitle dibuat, lalu ganti model setelah subtitle dibuat. Pastikan kata lama dibuang dan Buat Video tidak memakai kata dari media/model sebelumnya.
7. Uji video yang tidak menghasilkan kata. Pastikan pesan `Tidak ada ucapan terdeteksi di video ini`, ekspor/Buat Video tidak menganggap subtitle generated, dan tidak ada kata lama yang tertinggal.
8. Tekan Buat Video sebelum Buat Subtitle. Pastikan alur memanggil transkripsi yang sama dan hanya membuka dialog simpan jika transkripsi berhasil menghasilkan kata.
9. Buka aplikasi di browser untuk memastikan fixture mock dan label `SIMULASI` tetap berfungsi.

## Tahap 3.4 - Editor teks

### Ringkasan

- **Fakta:** tombol `Edit Teks` aktif jika ada kata dan aplikasi tidak sedang transkripsi atau menyiapkan/render video. Tombol membuka elemen native `<dialog>` dengan daftar kata yang dapat di-scroll, waktu `mm:ss.d`, input teks, gabung, hapus, reset, dan tutup.
- **Fakta:** Enter atau blur menyimpan teks yang sudah di-trim; Enter bergerak ke baris berikutnya. Teks kosong menghapus kata, dan spasi memecah input menjadi kata-kata terpisah. Tombol waktu mengatur posisi video/audio ke `start_ms`; gabung menggabungkan teks tanpa spasi dan memakai awal kata pertama serta akhir kata berikutnya.
- **Fakta:** semua edit memperbarui `state.words` dan memanggil render pratinjau yang sama. Ekspor dan burn yang sudah memakai `state.words` otomatis menerima hasil edit.
- **Fakta:** snapshot kata asli disalin saat transkripsi berhasil. `Kembalikan ke hasil asli` meminta konfirmasi; snapshot dibuang ketika media/model berubah atau transkripsi dijalankan ulang. Menjalankan Buat Subtitle lagi saat `edited` meminta konfirmasi sebelum memulai.
- **Fakta:** operasi kata murni berada di `src/js/wordEdit.js`; tesnya ditulis di `src/tests/wordEdit.test.mjs`.

### Aturan pembagian waktu saat split

- Rentang kata asal tetap dari `start_ms` sampai `end_ms`. Bobot tiap bagian adalah jumlah huruf Unicode; bagian yang tidak memiliki huruf memakai jumlah code point sebagai bobot fallback.
- Batas kumulatif dihitung dengan pembulatan ke bawah, kemudian dipakai bersama sebagai akhir bagian sebelumnya dan awal bagian berikutnya. Dengan demikian bagian tidak bertumpang-tindih atau terbalik, dan batas awal/akhir keseluruhan tidak bergeser. Jika durasinya lebih pendek daripada jumlah bagian, sebagian bagian dapat memiliki durasi nol karena tidak ada pembagian integer-milidetik positif yang mungkin untuk semua bagian.

### Pemeriksaan keamanan karakter ASS

- **Fakta dari source lokal:** `caption-core/src/ass.rs::escape_text()` mengganti `{` dan `}` dengan tanda kurung, menormalkan CRLF, dan mengubah baris baru menjadi `\N`. `caption-core/tests/ass_tests.rs::braces_and_newlines_are_escaped` menguji kurung kurawal/baris baru, sedangkan `comma_in_text_is_preserved` menguji tanda koma.
- **Celah yang ditemukan:** backslash literal tidak di-escape; cabang default `escape_text()` menyalinnya. Teks seperti `\N` atau pola backslash ASS lain berpotensi diperlakukan sebagai perintah oleh renderer, dan `{}` juga tidak dipertahankan literal karena diubah menjadi `()`. Tidak ada tes backslash pada suite yang diperiksa.
- **Batas perubahan:** editor memakai `input.value` dan teks pratinjau memakai DOM `textContent`, sehingga teks tidak dimasukkan sebagai HTML. Namun, itu tidak memperbaiki interpretasi ASS saat ekspor/burn. Backend/ASS tidak diubah sesuai batas tahap ini; backslash ASS tetap merupakan risiko yang belum terselesaikan.
- Tanda baca biasa tidak dihapus oleh fungsi edit; hanya whitespace tepi di-trim dan whitespace internal dipakai sebagai pemisah kata.

### File yang berubah

- `src/index.html`: tombol `Edit Teks` dan dialog native.
- `src/css/app.css`: tata letak aksi subtitle dan gaya dialog/list yang dapat di-scroll.
- `src/js/main.js`: render dan kontrol editor, integrasi state/pratinjau, snapshot asli, serta konfirmasi transkripsi ulang.
- `src/js/state.js`: `originalWords` dan flag `edited`.
- `src/js/wordEdit.js`: `editWord`, `splitWord`, `mergeWithNext`, `deleteWord`, dan `resetWords`.
- `src/tests/wordEdit.test.mjs`: tes unit logika kata.
- `REPORT.md`: laporan tahap ini.
- Tidak ada perubahan backend, dependency, burn, preset, ASS, atau format kata.

### Hal yang tidak yakin atau belum diuji

- Node tidak tersedia dan dilarang dijalankan; tes `src/tests/wordEdit.test.mjs` serta tes pratinjau tidak dijalankan. Uji interaksi modal, fokus, Escape, tab, dan perilaku pointer/keyboard juga belum dilakukan manual.
- Belum diuji secara visual pada jendela 1100x700, WebView2, atau browser. CSS membatasi tinggi modal dan membuat daftar scroll, tetapi hasil tampilannya masih perlu diverifikasi langsung.
- Untuk rentang waktu yang sangat pendek dengan banyak hasil split, beberapa bagian dapat memperoleh rentang nol milidetik; batas keseluruhan tetap sama dan urutan tidak terbalik.
- Backslash ASS belum aman pada backend yang ada; ini temuan nyata dari source, bukan dugaan bahwa escaping sudah lengkap.

### Hal yang sengaja tidak diubah

- Tidak mengubah Rust, `caption-core/src/ass.rs`, tes ASS, command Tauri, preview renderer, burn, preset, atau format `{ text, start_ms, end_ms }`.
- Tidak membuang tanda baca dari teks yang dikoreksi.
- Tidak menambah dependency dan tidak menjalankan git commit.

### Langkah uji manual

1. Buat subtitle, buka `Edit Teks`, lalu pastikan dialog dapat di-scroll dan tombol edit hanya aktif ketika kata tersedia serta aplikasi tidak sibuk.
2. Klik waktu beberapa kata dan pastikan video/audio melompat ke awal kata tersebut; uji tampilan waktu dalam format `mm:ss.d`.
3. Ubah satu kata, tekan Enter, lalu uji simpan saat fokus pindah. Pastikan Enter berpindah ke baris berikutnya dan preview langsung mengikuti `state.words`.
4. Ubah satu kata menjadi dua atau lebih kata dengan spasi. Periksa urutan, tidak ada waktu yang tumpang-tindih/terbalik, dan awal/akhir rentang total tetap sama.
5. Kosongkan satu kotak dan uji tombol Hapus serta Gabung dengan berikutnya. Pastikan gabung tidak menyisipkan spasi dan baris terakhir tidak memiliki tombol gabung.
6. Edit beberapa kata, tekan `Kembalikan ke hasil asli`, batalkan konfirmasi lalu konfirmasi; pastikan hanya pilihan konfirmasi yang memulihkan snapshot asli.
7. Jalankan Buat Subtitle lagi setelah edit dan uji kedua pilihan konfirmasi. Ganti media/model dan pastikan kata serta snapshot lama dibuang.
8. Ekspor `.ass` dan buat video setelah mengedit untuk memastikan hasil mengikuti kata yang tampil di editor. Uji `{}`, baris baru, backslash seperti `\N`, dan tanda baca; backslash harus dianggap belum aman sampai ada perbaikan backend terpisah.
9. Tutup dengan Escape dan tombol Tutup; pastikan fokus kembali ke `Edit Teks`. Periksa modal pada jendela 1100x700.

## Perbaikan - Burn kedua gagal

### Akar masalah dan diagnosis

- **Fakta — pesan error:** `burn_video` di `src-tauri/src/commands.rs` mengembalikan `Result<String, String>`, sementara `createVideo()` sebelumnya membaca `error.message`. Penolakan Tauri berbentuk string tidak memiliki properti itu, sehingga UI menampilkan `undefined`. `errorMessage(e)` kini menangani string, `Error`, objek `{ message }`, dan nilai kosong; seluruh penggunaan `.message` untuk menampilkan error di `src/js/main.js` diganti dengan helper tersebut. Pesan backend tidak diubah.
- **Fakta — status BurnManager:** `burn_video()` mengambil lease dari `BurnManager::reserve()` sebelum memanggil `spawn_blocking`. Lease dipindahkan ke closure; `Drop for BurnLease` memanggil `mark_finished()` lalu `BurnManager::release()`. Karena itu return sukses, return error (termasuk batal), dan unwind panic dari closure tetap menjatuhkan lease. Untuk panic, `spawn_blocking` mengembalikan JoinError setelah unwind; status aktif sudah dilepas oleh Drop. Tidak ditemukan jalur return normal yang melewati Drop lease.
- **Fakta — tes status:** ditambahkan tes lease dilepas setelah hasil error dan setelah panic simulasi. Keduanya lulus. Tes panic memang menulis pesan panic simulasi ke output test harness, lalu ditangkap oleh `catch_unwind`.
- **Fakta — file output yang terbuka:** `showBurnedPreview()` menetapkan `video.src` ke hasil `convertFileSrc(path)` dan memanggil `video.load()`. Sebelum perbaikan, `createVideo()` tidak melepaskan hasil itu sebelum burn berikutnya. Pada Windows, menulis ulang file yang masih dibuka WebView dapat gagal; ini penjelasan yang sesuai dengan laporan pengguna, tetapi pemblokiran file yang spesifik belum direproduksi langsung. Sebelum burn baru, UI kini memanggil `showSourcePreview()` bila pratinjau hasil aktif; fungsi tersebut melakukan `pause()`, `removeAttribute("src")`, dan `load()` sebelum memuat kembali media sumber.
- **Fakta — listener dan state UI:** `burnVideo()` di `src/js/api.js` sudah melepas listener `burn-progress` di `finally`; tidak ditemukan akumulasi listener pada jalur resolve/reject. `createVideo()` juga sudah mengatur ulang `preparingVideo`, `burnActive`, `burnCancelRequested`, dan `burnProgressStatus` di `finally`. Saat hasil lama dilepas, `showSourcePreview()` mengosongkan `burnedVideoPath` dan menonaktifkan `burnedPreviewActive`; hasil baru mengisi path kembali hanya setelah sukses.
- **Fakta — folder temp:** `run_ffmpeg_burn()` memanggil `WorkDirGuard::cleanup()` sesudah proses berhasil maupun gagal; `WorkDirGuard::drop()` mencoba menghapus direktori sebagai fallback, termasuk saat unwind. Pada pemeriksaan `%TEMP%` saat ini, tidak ditemukan direktori `caption-burn-*`. Ini hanya menyatakan kondisi saat pemeriksaan, bukan riwayat run sebelumnya atau direktori temp di lokasi lain.
- **Kesimpulan:** manager tidak menunjukkan bug pelepasan status pada alur yang diperiksa. Akar yang terbukti pada pesan `undefined` adalah asumsi bentuk error UI; jalur hasil lama tetap dimuat dan merupakan penyebab yang masuk akal untuk kegagalan overwrite kedua di Windows, tetapi perlu uji aplikasi untuk memastikan korelasinya dengan laporan pengguna.

### File yang berubah

- `src/js/main.js`: helper `errorMessage`, pemakaian di semua tampilan error, dan pelepasan hasil video sebelumnya sebelum burn baru.
- `src-tauri/src/commands_tests.rs`: tes pelepasan lease manager sesudah error dan panic.
- `REPORT.md`: temuan dan langkah verifikasi.
- Tidak ada perubahan pada runtime backend, command burn, format output, atau dependency.

### Hasil pemeriksaan

- **PASS:** `cargo fmt --all -- --check`.
- **PASS:** `cargo test -p caption-tauri --lib burn_manager_releases_lease -- --nocapture` — 2 lulus, 0 gagal.
- **PASS:** `git diff --check`.
- Node tidak dijalankan sesuai instruksi. Tidak ada uji burn UI/WebView atau overwrite file nyata pada Windows dalam sesi ini.

### Hal yang tidak yakin atau belum diuji

- Belum dapat memastikan lewat reproduksi bahwa file lock Windows adalah satu-satunya alasan burn kedua gagal; perubahan UI mengatasi skenario path hasil yang masih dimuat.
- Belum diuji burn dua kali ke path sama, burn dua kali ke nama berbeda, pembatalan, kegagalan FFmpeg, dan panic pada task yang menjalankan FFmpeg.
- Pemeriksaan temp dilakukan pada `%TEMP%` mesin saat ini dan menemukan nol direktori `caption-burn-*`; direktori yang sudah terhapus atau berada di lokasi lain tidak dapat disimpulkan.
- Tidak menjalankan seluruh suite Rust atau Clippy; validasi Rust dibatasi ke format dan dua tes manager yang baru.

### Langkah uji manual

1. Jalankan aplikasi, pilih video dan preset, lalu buat burn ke `hasil-sama.mp4`. Setelah sukses, pastikan video hasil tampil di preview.
2. Tanpa mengganti media, buat burn kedua dan pilih `hasil-sama.mp4` lagi; setujui overwrite. Pastikan progress selesai, file tertimpa, pesan sukses menampilkan path, dan UI tidak menampilkan `undefined`.
3. Ulangi dua burn berturut-turut dengan nama output berbeda. Pastikan kedua file tersimpan dan hasil kedua tampil.
4. Uji error burn yang disengaja (mis. input tidak tersedia) dan pembatalan; pastikan pesan string backend tampil utuh, tombol sibuk pulih, dan hasil lama tidak dilaporkan sebagai hasil baru.
5. Setelah sukses/gagal/batal, periksa `%TEMP%` untuk memastikan tidak ada direktori `caption-burn-*` yang tertinggal.

## Tahap 4.0 - Ekspor Multi-Format

### Ringkasan

Footer kini menyediakan pilihan ASS, SRT, VTT, TXT, dan JSON; dialog simpan memakai nama dasar media serta ekstensi/filter sesuai format. Command `write_export_file` sekarang merender format terpilih dari kata dan preset yang sama sebelum menulis file. ASS tetap memakai renderer preset, sedangkan format teks memakai renderer yang sudah ada di `caption-core/src/exporter.rs`; tidak ada dependency baru.

### File yang berubah

- `src/index.html`: dropdown format di footer dan label tombol ekspor generik.
- `src/css/app.css`: gaya dropdown agar konsisten dengan kontrol aplikasi.
- `src/js/api.js`: wrapper dialog dan command untuk kelima format.
- `src/js/main.js`: format terpilih, nama file, dan pesan status/error dinamis.
- `src-tauri/src/commands.rs`: renderer format ekspor bersama dan command penyimpanan.
- `src-tauri/src/commands_tests.rs`: tes semua format dan format tak dikenal.
- `REPORT.md`: laporan tahap ini.
- `caption-core/src/exporter.rs` dibaca saja, tidak diubah.

### Output mentah

- `cargo fmt --all -- --check` � lulus (exit code 0).
- `cargo test -p caption-tauri --lib write_export_file_for_format` � `2 passed; 0 failed`.
- `cargo test --workspace --no-default-features` � selesai dengan exit code 0; `caption-core` 100 lulus, `caption-tauri` 30 lulus dan 1 diabaikan.
- `cargo clippy --workspace --all-targets -- -D warnings` � gagal pada `src-tauri/src/commands_tests.rs:149`, lint `clippy::redundant_closure_call` pada tes BurnManager yang sudah ada sebelum perubahan ekspor ini.
- `cargo clippy --workspace --all-targets -- -D warnings -A clippy::redundant_closure_call` � lulus (exit code 0); tidak ada lint tambahan.
- `git diff --check` � exit code 0; Git memberi peringatan normal bahwa file UI CRLF akan dinormalisasi menjadi LF pada operasi Git berikutnya.

### Hal yang belum diuji

- Belum dijalankan secara manual di aplikasi Tauri/WebView; dialog native, ekstensi otomatis, dan isi file hasil perlu diperiksa pada Windows.
- Tidak menjalankan tes JavaScript atau Node.
- Clippy persis sesuai perintah wajib belum bersih karena lint tes lama yang disebut di atas; tes itu tidak diubah karena tidak terkait dengan ekspor.

### Langkah uji manual

1. Jalankan aplikasi, pilih media, lalu buat subtitle.
2. Ekspor masing-masing ASS, SRT, VTT, TXT, dan JSON. Pastikan dialog menyarankan nama media dengan ekstensi yang sesuai.
3. Buka hasilnya: ASS harus berisi gaya preset; SRT/VTT harus berisi timestamp dan teks grup; TXT harus berisi satu baris per grup; JSON harus berisi grup beserta kata dan waktunya.
4. Batalkan dialog simpan dan pastikan tidak ada file yang dibuat serta pesan status menyebut format yang dipilih.
5. Uji jalur ASS lama terhadap preset dan pilihan kata-per-baris yang sama untuk memastikan hasil styling tetap benar.

## Tahap 3.5 - Editor teks panel

### Ringkasan

Fakta: editor teks dipindahkan dari dialog modal menjadi tab Teks di panel kanan, berdampingan dengan tab Gaya. Daftar mendukung tampilan Kata, 3 Kata, 5 Kata, dan Kalimat; mode Kalimat memakai `groupWords(words, "sentence")` yang sudah ada di `src/js/preview.js`, sementara pengelompokan lain hanya membentuk tampilan dan tidak mengubah `state.words` atau pengaturan kata-per-baris preset. Setiap baris memiliki waktu, tombol Putar baris, dan textarea; mode Kata juga memiliki aksi gabung dan hapus. Penyimpanan teks kelompok menggunakan `applyGroupText()` di `src/js/wordEdit.js`, lalu memperbarui kata, status edited, dan preview.

### File yang berubah

- `src/index.html` — menghapus dialog editor lama dan menambahkan tab Gaya/Teks, toolbar mode, toggle Ikuti pemutaran, pesan kosong, daftar baris, serta tombol untuk memulihkan hasil asli.
- `src/css/app.css` — menambahkan gaya tab dan editor, termasuk gaya baris aktif, textarea, tombol waktu/play, dan aturan `[hidden]` untuk daftar yang memakai grid.
- `src/js/main.js` — menambahkan perender kelompok editor, penyimpanan teks kelompok, navigasi/fokus keyboard, seek/play baris, sinkronisasi baris aktif, serta read-only saat proses berjalan.
- `src/js/wordEdit.js` — menambahkan `applyGroupText()` beserta tokenisasi whitespace, pencocokan LCS, pembagian rentang waktu, dan validasi hasil tanpa mengubah masukan.
- `src/tests/wordEdit.test.mjs` — menambahkan pengujian kasus edit kelompok di samping tes fungsi edit kata yang sudah ada.
- `REPORT.md` — menambahkan bagian ini di bagian paling bawah; isi sebelumnya dipertahankan.

### Aturan `applyGroupText()` dan kasus tepi

Fakta: input dipisah menurut whitespace dan token kosong dibuang. LCS mencocokkan token yang sama persis dan mempertahankan `start_ms`/`end_ms` token tersebut; blok pengganti membagi gabungan waktu token lama menurut jumlah huruf Unicode, dengan fallback jumlah karakter Unicode saat token tidak memiliki huruf. Penghapusan token tidak menggeser waktu kata yang tersisa, dan teks kosong menghapus seluruh kelompok. Tes yang ditulis mencakup ganti satu token, token sama dengan waktu tetap, sisipan di tengah/ujung, penghapusan tengah/ujung, penggabungan/pemecahan jumlah token, Unicode, whitespace, rentang total, non-overlap, dan imutabilitas.

Pilihan implementasi untuk sisipan murni: bila ada jeda waktu di antara token lama yang cocok, waktu jeda itu dibagi proporsional untuk token sisipan. Bila sisipan berada di awal/akhir tanpa jeda kosong, token baru diberi rentang berdurasi nol pada batas kata tetangga agar waktu token identik tetap tidak berubah. Ini pilihan eksplisit untuk memenuhi syarat mempertahankan waktu token sama persis; belum diverifikasi sebagai preferensi UX.

### Perilaku fokus dan scroll

Fakta: tombol Edit Teks mengaktifkan tab Teks dan memfokuskan daftar. Enter menyimpan, mengelompokkan ulang, memfokuskan baris berikutnya yang memuat indeks kata setelah hasil edit, lalu seek ke awal baris itu tanpa memanggil play/pause. Bila tidak ada baris berikutnya, fokus kembali ke kelompok yang memuat awal hasil edit. Ctrl+Enter menyimpan lalu memutar kelompok; Escape mengembalikan teks textarea ke nilai sebelum edit; Shift+Enter dicegah sehingga tidak membuat baris baru. Setelah commit, daftar dirender ulang dan digulir ke kelompok yang memuat kata pertama hasil edit. Ikuti pemutaran aktif secara default; scroll mengikuti baris aktif, tetapi tidak saat textarea sedang fokus. Indeks terakhir yang benar-benar diikuti disimpan terpisah dari baris aktif agar scroll dapat menyusul lagi setelah fokus textarea berakhir.

### Verifikasi, hal yang belum diuji, dan ketidakpastian

- Fakta: browser smoke test dengan halaman lokal dan data transkripsi tiruan menghasilkan 48 baris mode Kata, 16 baris mode 3 Kata, 10 baris mode 5 Kata, dan 6 baris mode Kalimat. Setelah mengganti tiga token menjadi empat dan menekan Enter pada mode 3 Kata, daftar menghasilkan 17 baris dan textarea baris kedua menerima fokus. Tab Gaya/Teks tidak memakai elemen `<dialog>` dan preview tetap terlihat.
- Fakta: uji browser terhadap helper `applyGroupText()` memastikan sisipan tengah memakai celah waktu `[200, 300]`, masukan tetap sama, dan hasil tidak overlap.
- Fakta: `git diff --check` lulus; Git menampilkan peringatan normal tentang normalisasi CRLF pada file yang berubah.
- Fakta: `node --test src/tests/wordEdit.test.mjs` tidak dijalankan karena perintah `node` tidak ditemukan di PATH. Tes unit ditulis, tetapi belum dijalankan.
- Belum diuji: playback baris sampai jeda otomatis pada akhir kelompok, pembatalan row-play dengan pause/seek manual, perilaku Escape/Shift+Enter di WebView Tauri, read-only saat burn benar-benar berjalan, konfirmasi pemulihan hasil asli, dan transkripsi ulang setelah edit.
- Dugaan: perilaku fokus dan pemutaran di WebView Tauri mengikuti hasil browser lokal. Browser smoke test memakai mode simulasi, bukan burn atau transkripsi backend.
- Belum diuji: kasus waktu input yang sudah saling overlap atau tidak berurutan; helper melempar `RangeError` ketika hasilnya tidak berurutan, tetapi perilaku itu tidak dicakup tes baru.

### Hal yang sengaja tidak diubah

Fakta: tidak ada perubahan backend, Rust, ASS, burn, preset, maupun format kata `{text, start_ms, end_ms}`. Perubahan mode tampilan tidak mengubah nilai words-per-line preset. Tidak ada library baru dan tidak ada commit. Berkas lokal atau laporan lain yang sudah ada di worktree tidak dibersihkan atau diubah sebagai bagian tugas ini.

### Langkah uji manual

1. Jalankan aplikasi Tauri, pilih media, buat subtitle, lalu buka Edit Teks. Pastikan video dan kontrol play/seek tetap dapat dipakai saat tab Teks terbuka.
2. Pilih Kata, 3 Kata, 5 Kata, dan Kalimat; pastikan mode hanya mengubah pengelompokan tampilan. Pada Kata, uji Gabung dengan berikutnya dan Hapus.
3. Ubah teks kelompok, lalu uji Enter, Shift+Enter, Escape, dan Ctrl+Enter. Periksa fokus baris berikutnya dan posisi video setelah Enter; periksa pemutaran baris berhenti di akhir kelompok.
4. Putar video saat Ikuti pemutaran aktif, fokuskan textarea ketika video berjalan, lalu blur textarea. Pastikan baris aktif dan scroll kembali mengikuti pemutaran. Uji toggle Ikuti pemutaran.
5. Saat transkripsi dan burn berjalan, pastikan textarea read-only dan kontrol editor yang mengubah/memutar teks terkunci.
6. Edit beberapa kata, tekan Kembalikan ke hasil asli dan konfirmasi; pastikan teks asli kembali. Uji konfirmasi transkripsi ulang setelah edit.
