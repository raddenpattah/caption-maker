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
