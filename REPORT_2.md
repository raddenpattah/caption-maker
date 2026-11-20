# Laporan Fase 2 dan Perbaikan Core

## 1. Ringkasan

- Baseline CLI sebelum perubahan berhasil: 92 token, 48 kata, 24 grup.
- README diperbarui dengan roadmap sesuai status Fase 0-1 dan catatan batas.
- RawToken kini mempertahankan bytes; karakter UTF-8 lintas token pulih saat merge.
- CLI menerima `--prompt` dan `--threads`, serta mencetak pengukuran RTF.
- Workflow Windows untuk tes dan build Whisper ditambahkan; eksekusi CI belum diverifikasi.
- Semua pemeriksaan format, Clippy, tes default/no-default, dan pembandingan ASS lokal lulus.

## 2. Checklist tugas

| Tugas | Status | Bukti |
|---|---|---|
| 0 — Baseline regresi | **PASS** | `cargo run --release --bin caption-cli -- samples/test.wav --model models/ggml-small.bin --words 2 --highlight -o /tmp/out_before.ass`; menghasilkan `tokens=92 words=48 groups=24`. `nproc` menghasilkan `2`. |
| A — README Roadmap | **PASS** | Bagian `## Roadmap` ditambahkan; hanya core engine Fase 0-1 diberi `[x]`. |
| B1 — Bytes UTF-8 | **PASS** | `split_utf8_bytes_are_decoded_after_merging` lulus untuk `0xC3 0xA9`; `real_sample_fixture_merges_into_expected_word_count` tetap lulus dengan fixture lama tanpa `bytes` dan menghasilkan 48 kata. |
| B2 — Opsi dan timing CLI | **PASS** | `cargo run --bin caption-cli -- --help` menampilkan `--prompt` dan `--threads`; run nyata mencetak metrik RTF. README CLI diperbarui. |
| C — CI Windows | **PASS** untuk konfigurasi; **NOT RUN** untuk eksekusi GitHub Actions | `.github/workflows/ci.yml` memiliki job `test` dan `whisper-build` pada `windows-latest`. Kode workflow tidak bisa dieksekusi dari Codespace. |
| D — Verifikasi lokal | **PASS** | fmt, kedua Clippy, kedua test suite, CLI pasca-perubahan dan `diff` selesai dengan exit status 0. |

## 3. Output mentah

Semua perintah di bawah dijalankan dari root workspace. Output fmt kosong; perintah selesai dengan exit status 0.

### `cargo fmt --all -- --check`

```text
```

### `cargo clippy --workspace --all-targets -- -D warnings`

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.29s
```

### `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
```

### `cargo test --workspace`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running unittests src/lib.rs (target/debug/build/caption-core/8394430a313c9b62/out/caption_core-8394430a313c9b62)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/caption-cli.rs (target/debug/build/caption-core/af980ef2fa6aebbe/out/caption_cli-af980ef2fa6aebbe)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/ass_tests.rs (target/debug/build/caption-core/95bfad3f47e57438/out/ass_tests-95bfad3f47e57438)

running 14 tests
test braces_and_newlines_are_escaped ... ok
test comma_in_text_is_preserved ... ok
test dialogue_uses_first_start_and_last_end ... ok
test empty_groups_only_emit_header ... ok
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
test fewer_words_than_group_size_stays_in_one_group ... ok
test empty_input_returns_no_groups ... ok
test seven_words_are_grouped_with_remainders_for_each_size ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/merge_tests.rs (target/debug/build/caption-core/1d6b83f26ffc9bdc/out/merge_tests-1d6b83f26ffc9bdc)

running 8 tests
test bpe_fragments_joined ... ok
test empty_input_returns_no_words ... ok
test invalid_timestamps_are_repaired ... ok
test overlapping_words_are_shifted_forward ... ok
test punctuation_attached_and_spacing_starts_words ... ok
test special_tokens_dropped ... ok
test real_sample_fixture_merges_into_expected_word_count ... ok
test split_utf8_bytes_are_decoded_after_merging ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests caption_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### `cargo test --workspace --no-default-features`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.06s
     Running unittests src/lib.rs (target/debug/build/caption-core/f9705d6660ebd458/out/caption_core-f9705d6660ebd458)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/ass_tests.rs (target/debug/build/caption-core/fa537377552b483a/out/ass_tests-fa537377552b483a)

running 14 tests
test braces_and_newlines_are_escaped ... ok
test comma_in_text_is_preserved ... ok
test dialogue_uses_first_start_and_last_end ... ok
test empty_groups_only_emit_header ... ok
test groups_from_grouping_api_render_in_word_order ... ok
test header_contains_required_sections_and_event_columns ... ok
test highlight_inserts_gap_tag ... ok
test highlight_rounding_does_not_accumulate_drift ... ok
test highlight_uses_exact_cumulative_word_durations ... ok
test plain_mode_has_no_karaoke_tags ... ok
test real_sample_fixture_matches_golden_ass ... ok
test style_line_maps_colors_to_ass_bgr_order ... ok
test timestamp_format_basic ... ok
test timestamp_rounds_not_truncates ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/audio_tests.rs (target/debug/build/caption-core/2f4e718c3473aa1d/out/audio_tests-2f4e718c3473aa1d)

running 2 tests
test rejects_wav_with_wrong_channel_count_or_sample_rate ... ok
test loads_16khz_mono_i16_as_normalized_f32 ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/group_tests.rs (target/debug/build/caption-core/fa0f30a99e19dbcb/out/group_tests-fa0f30a99e19dbcb)

running 3 tests
test fewer_words_than_group_size_stays_in_one_group ... ok
test empty_input_returns_no_groups ... ok
test seven_words_are_grouped_with_remainders_for_each_size ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/merge_tests.rs (target/debug/build/caption-core/126953c38ffc9bdc/out/merge_tests-126953c38ffc9bdc)

running 8 tests
test bpe_fragments_joined ... ok
test empty_input_returns_no_words ... ok
test invalid_timestamps_are_repaired ... ok
test overlapping_words_are_shifted_forward ... ok
test punctuation_attached_and_spacing_starts_words ... ok
test real_sample_fixture_merges_into_expected_word_count ... ok
test special_tokens_dropped ... ok
test split_utf8_bytes_are_decoded_after_merging ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests caption_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### CLI pasca-perubahan

Perintah:

```sh
cargo run --release --bin caption-cli -- samples/test.wav --model models/ggml-small.bin --words 2 --highlight --threads 2 -o /tmp/out_after.ass
```

Output gabungan stdout/stderr:

```text
    Finished `release` profile [optimized] target(s) in 0.14s
     Running `target/release/caption-cli samples/test.wav --model models/ggml-small.bin --words 2 --highlight --threads 2 -o /tmp/out_after.ass`
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
whisper_init_state: compute buffer (cross) =   6.18 MB
whisper_init_state: compute buffer (decode) =   97.27 MB
tokens=92 words=48 groups=24 output=/tmp/out_after.ass
audio=23.185s transcribe=17.974s rtf=0.775 threads=2
```

### `diff /tmp/out_before.ass /tmp/out_after.ass`

```text
```

`diff` menghasilkan output kosong dan exit status 0; kedua file ASS identik.

## 4. Pengukuran kecepatan

- Audio: `23.185` detik.
- Waktu transkripsi: `17.974` detik.
- RTF: `0.775` (`transcribe / audio`).
- Thread: `2`.
- `nproc`: `2`.
- Baseline dan run pasca-perubahan sama-sama menghasilkan 92 token, 48 kata, dan 24 grup.

## 5. Deviasi dari instruksi

Tidak ada.

## 6. Masalah dan ketidakpastian

- Status eksekusi GitHub Actions: **NOT RUN**. Workflow Windows belum diverifikasi hijau; Codespace tidak menjalankan runner `windows-latest`.
- Build `whisper-build` mungkin memerlukan CMake dan/atau libclang pada runner Windows. Ini belum diverifikasi; workflow tidak menambahkan langkah spekulatif.
- Hasil ASS hanya dibandingkan byte demi byte; VLC/Aegisub tidak dijalankan dan verifikasi visual tidak dilakukan.

## 7. Yang belum dikerjakan dan saran langkah berikutnya

- Tunggu atau picu workflow GitHub Actions pada push/PR untuk memverifikasi job Windows.
- Jika `whisper-build` gagal karena prasyarat native Windows, tambahkan hanya langkah dependensi yang ditunjukkan oleh log kegagalan.
- Verifikasi visual `.ass` di VLC/Aegisub tetap belum dilakukan.
