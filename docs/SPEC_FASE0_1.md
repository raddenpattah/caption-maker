# Spesifikasi Teknis Fase 0 dan Fase 1: Caption Maker

## 1. Struktur workspace dan crate

Gunakan **Cargo workspace**, dengan core engine dipisah dari Tauri. Codespaces itu Linux dan dependensi Tauri (webkit2gtk dkk.) tidak perlu terpasang hanya untuk menguji engine. `src-tauri` baru menjadi dependen `caption-core` di Fase 3.

```text
caption-maker/
├── Cargo.toml                  # [workspace] members = ["caption-core"]  (src-tauri ditambah di Fase 3)
├── caption-core/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs              # re-export API publik
│   │   ├── error.rs            # enum CaptionError (thiserror)
│   │   ├── model.rs            # Word, Group, RawToken, opsi-opsi
│   │   ├── audio.rs            # extract_audio (FFmpeg), load_wav_16k_mono
│   │   ├── whisper.rs          # transcribe (di balik feature "whisper")
│   │   ├── merge.rs            # merge_tokens_to_words
│   │   ├── group.rs            # group_words
│   │   ├── ass.rs              # write_ass, render_ass
│   │   └── bin/caption-cli.rs  # CLI untuk Fase 0
│   └── tests/
│       ├── fixtures/           # tokens_id_sample.json, golden .ass
│       ├── merge_tests.rs
│       ├── group_tests.rs
│       └── ass_tests.rs
└── src-tauri/ ...              # Fase 3
```

**Cargo.toml core (garis besar):**

```toml
[features]
default = ["whisper"]
whisper = ["dep:whisper-rs"]   # matikan untuk menjalankan unit test tanpa compile whisper.cpp

[dependencies]
whisper-rs = { version = "=0.14.x", optional = true }  # pin versi persis, API-nya sering berubah
hound      = "3"            # baca WAV
thiserror  = "1"
serde      = { version = "1", features = ["derive"] }
serde_json = "1"            # fixture + --dump-tokens
clap       = { version = "4", features = ["derive"] }  # CLI

[dev-dependencies]
pretty_assertions = "1"
```

Build whisper-rs butuh `cmake` dan `clang`/`libclang`. Di Codespaces: `sudo apt install cmake clang`. Dengan feature flag, `cargo test --no-default-features` jalan cepat dan dipakai juga di CI.

## 2. Skema data

```rust
// model.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Word {
    pub text: String,   // sudah termasuk tanda baca yang menempel, TANPA spasi di depan/belakang
    pub start_ms: u32,
    pub end_ms: u32,    // invarian: end_ms > start_ms
}

#[derive(Debug, Clone, PartialEq)]
pub struct Group { pub words: Vec<Word> }   // 1..=3 kata
impl Group {
    pub fn start_ms(&self) -> u32 { self.words.first().unwrap().start_ms }
    pub fn end_ms(&self) -> u32 { self.words.last().unwrap().end_ms }
}

/// Token mentah dari whisper, sebelum di-merge. Dipisah dari Word supaya merge bisa dites dari fixture JSON.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawToken {
    pub text: String,   // bisa diawali spasi → penanda awal kata baru
    pub t0_ms: i64,     // bisa -1 / tidak valid dari whisper
    pub t1_ms: i64,
    pub id: i32,
    pub p: f32,
    pub is_special: bool, // [_BEG_], [_TT_xxx], <|endoftext|>, dst.
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum WordsPerLine { One = 1, Two = 2, Three = 3 }

pub struct TranscribeOptions {
    pub model_path: PathBuf,
    pub language: String,                    // default "id"
    pub threads: i32,                        // default: min(8, jumlah core)
    pub initial_prompt: Option<String>,      // opsional, bantu ejaan/nama
    pub on_progress: Option<Box<dyn Fn(i32) + Send>>, // 0..=100, dipakai UI Fase 3
}

pub struct AssStyle {
    pub font_name: String,       // "Arial"
    pub font_size: u32,          // 64
    pub primary: Rgb,            // warna highlight (terisi)
    pub secondary: Rgb,          // warna sebelum terisi
    pub outline_color: Rgb,
    pub outline: f32,
    pub shadow: f32,
    pub alignment: u8,           // numpad 1–9; default 2 (bawah tengah)
    pub margin_v: u32,
}

pub struct AssOptions {
    pub style: AssStyle,
    pub highlight: bool,         // true → tag \kf
    pub play_res_x: u32,         // 1920
    pub play_res_y: u32,         // 1080
}
```

## 3. Signature fungsi

```rust
// audio.rs
pub fn extract_audio(ffmpeg: &Path, input: &Path, out_wav: &Path) -> Result<(), CaptionError>;
// ffmpeg -y -i <input> -vn -ac 1 -ar 16000 -c:a pcm_s16le <out_wav>
pub fn load_wav_16k_mono(path: &Path) -> Result<Vec<f32>, CaptionError>;
// validasi 16 kHz/mono, konversi i16 → f32 (/ 32768.0)

// whisper.rs  (#[cfg(feature = "whisper")])
pub fn transcribe(samples: &[f32], opts: &TranscribeOptions)
    -> Result<Vec<RawToken>, CaptionError>;

// merge.rs
pub fn merge_tokens_to_words(tokens: &[RawToken]) -> Vec<Word>;

// group.rs
pub fn group_words(words: &[Word], per_line: WordsPerLine) -> Vec<Group>;

// ass.rs
pub fn write_ass<W: std::io::Write>(groups: &[Group], opts: &AssOptions, out: &mut W)
    -> std::io::Result<()>;
pub fn render_ass(groups: &[Group], opts: &AssOptions) -> String;  // wrapper untuk test
```

`transcribe` sengaja mengembalikan `Vec<RawToken>`, bukan `Vec<Word>`. Hasilnya bisa dibekukan ke JSON (`--dump-tokens`) lalu dipakai sebagai fixture, jadi merge, group, dan ASS bisa diuji tanpa model sama sekali.

## 4. Konfigurasi whisper-rs untuk timing level token

```rust
let ctx = WhisperContext::new_with_params(
    model_path_str, WhisperContextParameters::default())?;
let mut state = ctx.create_state()?;

let mut p = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
p.set_language(Some("id"));
p.set_translate(false);          // tidak ada terjemahan
p.set_token_timestamps(true);    // WAJIB: mengisi t0/t1 per token
p.set_split_on_word(true);       // pecah segmen di batas kata, bukan di tengah token
p.set_n_threads(opts.threads);
p.set_print_progress(false);
p.set_print_realtime(false);
p.set_print_special(false);
p.set_print_timestamps(false);
// opsional: p.set_initial_prompt(...); p.set_no_context(true) bila terjadi halusinasi berulang
// opsional: progress callback → opts.on_progress

state.full(p, samples)?;
```

Membaca token (API 0.11–0.13; di 0.14+ ada iterator `state.as_iter()` dan metode segmen/token, jadi sesuaikan dengan versi yang dipin dan **verifikasi lewat source/docs crate, jangan menebak**):

```rust
for s in 0..state.full_n_segments()? {
    for t in 0..state.full_n_tokens(s)? {
        let bytes = state.full_get_token_bytes(s, t)?;   // bukan _text, lihat catatan UTF-8
        let d = state.full_get_token_data(s, t)?;        // d.t0, d.t1 dalam satuan 10 ms
        let text = String::from_utf8_lossy(&bytes).into_owned();
        // is_special: d.id >= ctx.token_eot()  (token [_BEG_], [_TT_*], dll.)
        out.push(RawToken { text, t0_ms: d.t0 * 10, t1_ms: d.t1 * 10, id: d.id, p: d.p, is_special });
    }
}
```

Catatan penting:

- **Satuan waktu** `t0`/`t1` adalah sentidetik (10 ms), jadi kalikan 10 untuk milidetik.
- **UTF-8**: satu karakter multibyte bisa terbelah di dua token BPE. Fungsi `_text` bisa gagal untuk token seperti itu, jadi ambil bytes. Idealnya akumulasi bytes per kata baru di-decode di `merge`.
- **Akurasi**: timing token dari whisper.cpp berbasis heuristik, bukan forced alignment. Nilai `-1` atau t0 = t1 bisa muncul, jadi `merge` harus defensif. Opsi DTW untuk timing lebih presisi adalah kandidat optimasi pasca-v0.1.0, jangan masuk Fase 0.
- Audio harus **f32, 16 kHz, mono**.

## 5. Aturan `merge_tokens_to_words`

1. Buang token `is_special`.
2. Token yang teksnya **diawali spasi** memulai kata baru. Token tanpa spasi di depan **ditempel** ke kata sebelumnya. Ini menyatukan potongan BPE (`" Mul"` + `"do"` + `"on,"`) dan membuat tanda baca menempel pada kata.
3. `start_ms` = `t0` token pertama kata, `end_ms` = `t1` token terakhir.
4. Normalisasi: trim spasi; kata kosong dibuang.
5. Perbaikan waktu: `start >= end` kata sebelumnya (geser, jangan tumpang tindih); bila `end <= start`, set `end = start + 10` atau pakai start kata berikutnya sebagai batas; t0/t1 negatif diisi dari tetangga.

## 6. Aturan `group_words` dan `write_ass`

- `group_words`: `words.chunks(n)`. Sisa di akhir menjadi grup lebih pendek. Input kosong menghasilkan `vec![]`.
- Setiap `Dialogue`: start = awal kata pertama, end = akhir kata terakhir.
- Format waktu ASS: `H:MM:SS.cc` (sentidetik, **dibulatkan**, bukan dipotong).
- Highlight (`\kf`): durasi kata dihitung dari **batas kumulatif relatif awal grup**, bukan `(end-start)/10` per kata yang dipotong. Dengan cara kumulatif, galat pembulatan tidak menumpuk dalam satu baris. Hasilnya sama dengan rumus di brief untuk kasus bulat.
- Jeda antar kata dalam satu grup diisi tag `{\k<gap>}` kosong supaya pengisian warna tidak maju terlalu cepat.
- Teks: `\N` untuk newline; karakter `{` dan `}` diganti `(` `)`, karena bisa terbaca sebagai blok override. Koma aman di field Text karena field itu terakhir.

**Jebakan warna `\kf`:** pada karaoke ASS, `SecondaryColour` adalah warna *sebelum* terisi dan `PrimaryColour` adalah warna *setelah* terisi (highlight). Di UI, "warna highlight" dipetakan ke Primary dan "warna teks dasar" ke Secondary. Kalau highlight mati, Primary dipakai sebagai warna teks biasa, jadi `write_ass` perlu menyesuaikan.

## 7. Contoh `.ass` minimal yang valid

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
Dialogue: 0,0:00:01.00,0:00:02.20,Default,,0,0,0,,{\kf50}Halo {\kf70}dunia,
Dialogue: 0,0:00:02.40,0:00:03.60,Default,,0,0,0,,{\kf60}selamat {\kf60}datang
Dialogue: 0,0:00:03.80,0:00:04.50,Default,,0,0,0,,{\kf15}di {\kf55}kanal
```

Warna ASS berformat `&HAABBGGRR`, jadi `&H0000FFFF` itu kuning dan `&H00FFFFFF` putih.

Data sumber (grouping 2 kata): Halo 1000–1500, dunia, 1500–2200, selamat 2400–3000, datang 3000–3600, di 3800–3950, kanal 3950–4500.

## 8. Rencana unit test

**`ass_tests.rs` (`write_ass`):**

| Test | Yang diverifikasi |
|---|---|
| `timestamp_format_basic` | 1000 ms → `0:00:01.00`; 3_725_450 ms → `1:02:05.45` |
| `timestamp_rounds_not_truncates` | 1005 ms → `0:00:01.01` (aturan pembulatan konsisten) |
| `header_contains_required_sections` | ada `[Script Info]`, `[V4+ Styles]`, `[Events]`, baris `Format:` dengan jumlah kolom benar |
| `style_line_maps_colors` | Primary = highlight, Secondary = teks dasar; urutan BGR benar |
| `plain_mode_no_k_tags` | `highlight=false` → tidak ada `\k` sama sekali |
| `highlight_kf_durations` | contoh di atas menghasilkan `\kf50`, `\kf70`, dst. persis |
| `highlight_no_rounding_drift` | 3 kata masing-masing 333 ms: total `\kf` = durasi baris ±1 cs |
| `highlight_inserts_gap_tag` | jeda 200 ms di dalam grup → `{\k20}` |
| `escape_braces_and_newline` | `{` `}` diganti; `\n` → `\N` |
| `comma_in_text_preserved` | `dunia,` tetap utuh di field Text |
| `dialogue_start_end_from_group` | start kata pertama, end kata terakhir |
| `empty_groups_only_header` | tanpa grup → header + `[Events]` + baris Format, tanpa Dialogue |
| `golden_file` | `render_ass` dari fixture sama dengan `fixtures/expected.ass` |

**`group_tests.rs`:** `n=1/2/3` pada 7 kata (sisa grup terakhir), `empty_input`, `fewer_words_than_n`, urutan dan isi kata tidak berubah.

**`merge_tests.rs`:** `bpe_fragments_joined` (`" Mul"`, `"do"`, `"on,"` → satu kata), `punctuation_attached`, `special_tokens_dropped`, `invalid_t0_t1_repaired`, `no_overlap_between_words`, `empty_input`, serta satu tes fixture nyata dari `--dump-tokens`.

## 9. CLI dan kriteria selesai

```text
caption-cli <input> --model ggml-small.bin --words 2 [--highlight] [-o out.ass]
            [--ffmpeg /path/ffmpeg] [--dump-tokens tokens.json]
```

**Fase 0 selesai bila:** satu file audio Indonesia menghasilkan `.ass` yang terbuka di VLC atau Aegisub, kata-kata muncul sesuai ucapan, dan mode highlight mengisi warna kata per kata. Token mentah dari run itu disimpan sebagai fixture.

**Fase 1 selesai bila:** `cargo test --no-default-features` hijau tanpa model, semua fungsi di atas terpisah rapi, dan CLI end-to-end jalan.

## 10. Keputusan terbuka

1. **Pemecahan grup di jeda panjang.** Brief meminta grouping murni 1/2/3 kata, jadi grup bisa melintasi jeda panjang. Tambahkan `max_gap_ms: Option<u32>` (default `None`, perilaku sesuai brief) agar bisa dipakai nanti tanpa mengubah API.
2. **Versi `whisper-rs`.** API pembacaan segmen/token berubah antar versi minor. Pin versi persis di Fase 0 dan sesuaikan kode pemanggilan dengan versi itu.
