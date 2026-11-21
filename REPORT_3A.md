# Laporan Fase 3A — UI statis dan backend tiruan

## 1. Ringkasan

- Frontend HTML/CSS/JavaScript ES modules statis dibuat di `src/`; tidak ada framework, build step, Tauri, atau dependency baru.
- CLI menerima `--dump-words <path>` dan menulis array JSON pretty hasil merge. Tes serialisasi baru lulus.
- Run Whisper small pada `samples/test.wav` menghasilkan 92 token dan 48 kata. Hasilnya disimpan sebagai `src/mock/words.json`.
- API frontend tetap berupa mock. Preview mengelompokkan kata dan mensimulasikan progressive fill dengan HTML/CSS; ekspor `.ass` nyata belum dibuat.
- Perubahan dibuat dalam commit per tugas:
  - T1 `91e41d4` — `feat(cli): dump merged words as JSON`
  - T2 `e6dbad5` — `chore(ui): scaffold offline frontend structure`
  - T3 `bb04476` — `feat(ui): add offline mock backend API`
  - T4 `0bcfff3` — `feat(ui): build offline subtitle workspace`
  - T5 `bfdb1e2` — `feat(ui): render grouped karaoke preview`
  - T6 `973359e` — `test(ui): cover preview word grouping`
  - Koreksi lokalisasi T4 `0a34ff0` — `fix(ui): localize status and font labels`

## 2. Checklist tugas

| Tugas | Status | Bukti |
|---|---|---|
| 1 — Data nyata untuk mock | **PASS** | `--dump-words` menulis JSON pretty; tes `merged_words_serialize_as_pretty_word_array` lulus; run aktual mencetak `tokens=92 words=48 groups=24`; fixture berisi 48 objek dengan `text`, `start_ms`, `end_ms`. |
| 2 — Struktur frontend | **PASS** | Semua file yang diminta ada di `src/`; cara menjalankan dicantumkan di `src/README.md`. |
| 3 — API mock | **PASS** | Kelima fungsi API tersedia dan masing-masing memiliki komentar `MOCK - diganti invoke Tauri di 3B`; data mock dimuat dari path lokal. |
| 4 — UI | **PASS** | Layout gelap, zona drop/pilih file, elemen media lokal, timer internal, kontrol pengaturan, panduan model, progress, batal, dan ekspor nonaktif sampai hasil ada tersedia di sumber. Endpoint statis HTTP 200. |
| 5 — Preview karaoke | **PASS** | `groupWords` murni untuk 1/2/3 kata dan renderer HTML/CSS mengisi kata progresif sesuai waktu/pengaturan. README menegaskan preview bukan renderer ASS. |
| 6 — Tes ringan | **PASS** | `node --test src/tests/preview.test.mjs`: 6 tes lulus, 0 gagal. Node v24.22.2 tersedia. |
| 7 — Verifikasi server/offline | **PASS** | Semua 7 resource mengembalikan HTTP 200; grep tidak menemukan URL eksternal; server dihentikan setelah pemeriksaan. |
| 8 — Laporan | **PASS** | Laporan ini dibuat dan dikomit. |

## 3. Bukti run CLI dan keluaran mentah

Perintah untuk menghasilkan fixture:

```sh
cargo run --release --bin caption-cli -- samples/test.wav --model models/ggml-small.bin --words 2 --highlight -o /tmp/caption-maker-fase3a.ass --dump-words src/mock/words.json
```

Baris ringkasan stdout/stderr dari run aktual:

```text
tokens=92 words=48 groups=24 output=/tmp/caption-maker-fase3a.ass
audio=23.185s transcribe=60.883s rtf=2.626 threads=2
```

JSON diperiksa dengan Python: 48 kata, dan setiap objek memiliki tepat tiga field yang diminta. Audio dan model berada di luar perubahan yang dikomit.

### `cargo fmt --all -- --check`

```text
```

Exit status 0; stdout/stderr kosong.

### `cargo clippy --workspace --all-targets -- -D warnings`

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
```

### `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`

```text
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.02s
```

### `cargo test --workspace`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.07s
     Running unittests src/lib.rs (target/debug/build/caption-core/9234424914fe8a0d/out/caption_core-9234424914fe8a0d)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/bin/caption-cli.rs (target/debug/build/caption-core/411e2faef00dc41a/out/caption_cli-411e2faef00dc41a)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/ass_tests.rs (target/debug/build/caption-core/a56b3cdbe7535006/out/ass_tests-a56b3cdbe7535006)

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

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/audio_tests.rs (target/debug/build/caption-core/96f9aa9dc4f9957d/out/audio_tests-96f9aa9dc4f9957d)

running 2 tests
test loads_16khz_mono_i16_as_normalized_f32 ... ok
     Running tests/group_tests.rs (target/debug/build/caption-core/1ac7fe48c95f715e/out/group_tests-1ac7fe48c95f715e)
test rejects_wav_with_wrong_channel_count_or_sample_rate ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 3 tests
test empty_input_returns_no_groups ... ok
test fewer_words_than_group_size_stays_in_one_group ... ok
test seven_words_are_grouped_with_remainders_for_each_size ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/merge_tests.rs (target/debug/build/caption-core/d4718a35d0f41292/out/merge_tests-d4718a35d0f41292)

running 9 tests
test bpe_fragments_joined ... ok
test empty_input_returns_no_words ... ok
test invalid_timestamps_are_repaired ... ok
test merged_words_serialize_as_pretty_word_array ... ok
test overlapping_words_are_shifted_forward ... ok
test punctuation_attached_and_spacing_starts_words ... ok
test real_sample_fixture_merges_into_expected_word_count ... ok
test special_tokens_dropped ... ok
test split_utf8_bytes_are_decoded_after_merging ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests caption_core

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### `cargo test --workspace --no-default-features`

```text
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.02s
     Running unittests src/lib.rs (target/debug/build/caption-core/369740b0079d94c5/out/caption_core-369740b0079d94c5)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/ass_tests.rs (target/debug/build/caption-core/00afb6aa6c29e5bf/out/ass_tests-00afb6aa6c29e5bf)

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

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/audio_tests.rs (target/debug/build/caption-core/fc5071ebc6c60831/out/audio_tests-fc5071ebc6c60831)

running 2 tests
test loads_16khz_mono_i16_as_normalized_f32 ... ok
test rejects_wav_with_wrong_channel_count_or_sample_rate ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/group_tests.rs (target/debug/build/caption-core/6e3d72fa547484a6/out/group_tests-6e3d72fa547484a6)
test fewer_words_than_group_size_stays_in_one_group ... ok
test seven_words_are_grouped_with_remainders_for_each_size ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 9 tests
test bpe_fragments_joined ... ok
test empty_input_returns_no_words ... ok
test invalid_timestamps_are_repaired ... ok
test merged_words_serialize_as_pretty_word_array ... ok
test overlapping_words_are_shifted_forward ... ok
test punctuation_attached_and_spacing_starts_words ... ok
test special_tokens_dropped ... ok
   Doc-tests caption_core
test split_utf8_bytes_are_decoded_after_merging ... ok
test real_sample_fixture_merges_into_expected_word_count ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### Curl resource statis

Perintah: `for path in index.html css/app.css js/main.js js/api.js js/preview.js js/state.js mock/words.json; do curl -sS -o /dev/null -w "%{http_code} %{url_effective}\n" "http://127.0.0.1:8080/$path"; done`

```text
200 http://127.0.0.1:8080/index.html
200 http://127.0.0.1:8080/css/app.css
200 http://127.0.0.1:8080/js/main.js
200 http://127.0.0.1:8080/js/api.js
200 http://127.0.0.1:8080/js/preview.js
200 http://127.0.0.1:8080/js/state.js
200 http://127.0.0.1:8080/mock/words.json
```

### Grep URL eksternal

Pemeriksaan mengecualikan `localhost` dan `127.0.0.1`, karena URL lokal dipakai di petunjuk server. Output perintah:

```text
Tidak ada URL eksternal
```

### `node --test src/tests/preview.test.mjs`

```text
✔ mengelompokkan satu kata per baris (1.622385ms)
✔ mengelompokkan dua kata per baris (0.255387ms)
✔ mengelompokkan tiga kata per baris (0.244666ms)
✔ mengembalikan array kosong untuk masukan kosong (0.168635ms)
✔ mempertahankan urutan kata dan tidak mengubah masukan (0.376341ms)
✔ menolak jumlah kata per baris di luar 1 sampai 3 (0.406658ms)
ℹ tests 6
ℹ suites 0
ℹ pass 6
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 92.425669
```

## 4. File tree `src/`

```text
src/
├── README.md
├── css/
│   └── app.css
├── index.html
├── js/
│   ├── api.js
│   ├── main.js
│   ├── preview.js
│   └── state.js
├── mock/
│   └── words.json
└── tests/
    └── preview.test.mjs
```

## 5. Deviasi

- Tidak ada deviasi dari batas lingkup. Tidak ada dependency Tauri/Cargo baru, tidak ada perubahan selain tambahan CLI/test yang diizinkan pada `caption-core`, dan tidak ada fitur roadmap Tahap 2–3.
- Server pengembangan yang digunakan hanya `python3 -m http.server`; tidak ada build step atau panggilan jaringan eksternal.

## 6. Masalah, keterbatasan, dan ketidakpastian

- **NOT RUN — penilaian visual.** Tampilan tidak dibuka/ditinjau di browser secara visual. Verifikasi HTTP dan tes JavaScript bukan bukti tampilan atau interaksi UI benar secara visual.
- **NOT RUN — validasi interaksi media di browser.** Pemutaran dan pembacaan durasi bergantung pada dukungan codec browser; hanya endpoint statis yang diuji.
- Backend seluruhnya mock: data `words.json` ditampilkan tanpa mentranskripsi file yang dipilih. Impor model belum memasang model. `small` ditandai terpasang hanya pada daftar mock.
- Ekspor `.ass` nyata sengaja tidak ada; tombol hanya menampilkan bahwa ekspor nyata dihubungkan pada Fase 3B.
- CLI memakai model small di CPU pada run fixture; transkripsi memerlukan sekitar 60.883 detik untuk audio 23.185 detik pada run ini.

## 7. Saran langkah berikutnya

1. Pada Fase 3B, sambungkan API frontend ke command Tauri untuk daftar/impor model, pemilihan media, transkripsi/progress, dan ekspor `.ass` nyata.
2. Tinjau layout, kontras, keyboard, sinkronisasi video/audio, serta codec di browser target Windows; lakukan penilaian visual secara eksplisit.
3. Setelah ekspor native tersedia, validasi file `.ass` hasilnya dengan pemutar subtitle dan pastikan preview browser tidak dianggap representasi renderer ASS.
