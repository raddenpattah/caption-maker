# Report: preset kata-kotak-merah

## A. Data preset dan validasi

File/baris: `caption-core/presets/kata-kotak-merah.json:18,21,27`.
`validate_preset` menerima preset melalui `builtin_presets()`; tes
`all_builtin_presets_valid` memanggil validasi untuk setiap preset.
Tidak ada perubahan pada nilai gaya selain animation dan bayangan yang diminta.

Diff:
```diff
@@
-    "shadow": "#00000000"
+    "shadow": "#000000A0"
@@
-  "shadow": 0,
+  "shadow": 6,
@@
   "emphasis_scale": null,
+  "animation": "pop",
   "words_per_line": 1,
```

## B. Mock JavaScript

File/baris: `src/js/api.js:241-245`.

Diff:
```diff
@@
-      colors: { text: "#FFFFFF", highlight: "#FFE14D", outline: "#8B0A3C", shadow: "#00000000" },
-      outline: 18, shadow: 0, box: { color: "#8B0A3C", padding: 22 }, margin_v: 450, margin_lr: 80,
-      glow: null, emphasis: "none", emphasis_scale: null, animation: "none",
+      colors: { text: "#FFFFFF", highlight: "#FFE14D", outline: "#8B0A3C", shadow: "#000000A0" },
+      outline: 18, shadow: 6, box: { color: "#8B0A3C", padding: 22 }, margin_v: 450, margin_lr: 80,
+      glow: null, emphasis: "none", emphasis_scale: null, animation: "pop",
```

## C. Tes

File/baris:
- `caption-core/tests/preset_tests.rs:114-123`: himpunan preset beranimasi pop kini tepat `{kata-tebal-putih, kata-kotak-merah}`.
- `src-tauri/src/commands_tests.rs:284-292`: serialisasi daftar preset memakai himpunan pop yang sama.
- `caption-core/tests/ass_preset_tests.rs:393-421`: tes baru memastikan BorderStyle 3, Shadow 6, BackColour dari `ass_color_from_hex("#000000A0")`, dan tag pop di awal setiap Dialogue.
- `src/tests/preview.test.mjs`: dicari referensi preset/animasi tersebut; tidak ditemukan, jadi tidak diubah.

Diff:
```diff
@@ caption-core/tests/preset_tests.rs
-fn only_kata_tebal_putih_has_pop_animation() {
+fn kata_one_word_presets_have_pop_animation() {
@@
-            if preset.id == "kata-tebal-putih" {
+            if ["kata-tebal-putih", "kata-kotak-merah"].contains(&preset.id.as_str()) {
                 Animation::Pop
@@ src-tauri/src/commands_tests.rs
-        let expected = if preset.id == "kata-tebal-putih" {
+        let expected = if ["kata-tebal-putih", "kata-kotak-merah"].contains(&preset.id.as_str()) {
             Animation::Pop
@@ caption-core/tests/ass_preset_tests.rs
+#[test]
+fn kata_kotak_merah_pop_and_box_shadow_are_rendered() {
+    let preset = find_preset("kata-kotak-merah").unwrap();
+    let ass = render_ass_with_preset(&words_3(), &preset, portrait(), &PresetOptions::default());
+    let style = ass
+        .lines()
+        .find(|line| line.starts_with("Style: Default,"))
+        .unwrap();
+    let fields: Vec<_> = style.splitn(23, ',').collect();
+    assert_eq!(fields[15], "3", "kotak harus memakai BorderStyle 3");
+    assert_eq!(fields[17], "6", "Shadow harus 6");
+    assert_eq!(
+        fields[6],
+        ass_color_from_hex("#000000A0").unwrap(),
+        "BackColour harus berasal dari warna shadow preset"
+    );
+
+    let expected_pop = r"{\fscx85\fscy85\t(0,90,\fscx108\fscy108)\t(90,160,\fscx100\fscy100)}";
+    let events = dialogue_lines(&ass);
+    assert_eq!(events.len(), 3);
+    for event in events {
+        let text = event.splitn(10, ',').nth(9).unwrap();
+        assert!(
+            text.starts_with(expected_pop),
+            "setiap event kotak merah harus dimulai dengan tag pop: {event}"
+        );
+    }
+}
```

Tes berubah karena `kata-kotak-merah` kini juga memakai animation `pop`; tes ASS baru memeriksa aturan perubahan yang diminta. Tes `preview.test.mjs` tidak mengunci daftar preset pop.

## D. Dokumentasi preset

File/baris: `PRESETS.md:71-90`. Berkas ini sudah tidak terlacak di Git saat mulai dikerjakan; hanya entri dua preset berikut yang diedit.

`PRESETS.md` untracked sehingga tidak muncul di `git diff`. Berikut perubahan baris yang diterapkan:
```diff
 ## Kata Kotak Merah (id: `kata-kotak-merah`)
- **Colors**: text `#FFFFFF`; highlight `#FFE14D`; outline `#8B0A3C`; shadow `#00000000`
- **Outline**: 18; **Shadow**: 0; **Blur**: 0
+ **Colors**: text `#FFFFFF`; highlight `#FFE14D`; outline `#8B0A3C`; shadow `#000000A0`
+ **Outline**: 18; **Shadow**: 6; **Blur**: 0
 - **Glow**: `null`
 - **Emphasis**: `none`; emphasis_scale `null`
+ **Animation**: `pop`
 ## Kata Tebal Putih (id: `kata-tebal-putih`)
 - **Glow**: `null`
 - **Emphasis**: `none`; emphasis_scale `null`
+ **Animation**: `pop`
```

## E. Bukti CLI dan frame

### ASS

Perintah:
```text
cargo run -p caption-core --features whisper --bin caption-cli -- --words-json src\mock\words.json --preset kata-kotak-merah --size 1080x1920 -o target\kotak-proof\kotak.ass
```

Output mentah:
```text
Compiling caption-core v0.1.0 (E:\Projects\caption-maker\caption-core)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 15.04s
Running `target\debug\caption-cli.exe --words-json src\mock\words.json --preset kata-kotak-merah --size 1080x1920 -o target\kotak-proof\kotak.ass`
words=48 groups=48 output=target\kotak-proof\kotak.ass
```

Baris Style dan tiga Dialogue pertama, diambil langsung dari ASS:
```text
Style: Default,Poppins ExtraBold,88,&H00FFFFFF&,&H00FFFFFF&,&H003C0A8B&,&H5F000000&,0,0,0,0,100,100,0,0,3,22,6,2,80,80,450,1
Dialogue: 1,0:00:00.12,0:00:00.36,Default,,0,0,0,,{\fscx85\fscy85\t(0,90,\fscx108\fscy108)\t(90,160,\fscx100\fscy100)}LAGI
Dialogue: 1,0:00:00.39,0:00:00.74,Default,,0,0,0,,{\fscx85\fscy85\t(0,90,\fscx108\fscy108)\t(90,160,\fscx100\fscy100)}CARI
Dialogue: 1,0:00:00.77,0:00:01.40,Default,,0,0,0,,{\fscx85\fscy85\t(0,90,\fscx108\fscy108)\t(90,160,\fscx100\fscy100)}KAUTION
```

### Render PNG

Render memakai `--render-frame`; input FFmpeg adalah `color=c=0x202020`, yaitu latar abu-abu gelap `#202020`. Perintah di bawah mengalihkan stderr FFmpeg agar output stdout CLI tetap terbaca.

Perintah:
```text
target\debug\caption-cli.exe --words-json src\mock\words.json --preset kata-kotak-merah --size 1080x1920 --render-frame E:\Projects\caption-maker\target\kotak-proof\kotak-120.png --at-ms 120 2>$null
target\debug\caption-cli.exe --words-json src\mock\words.json --preset kata-kotak-merah --size 1080x1920 --render-frame E:\Projects\caption-maker\target\kotak-proof\kotak-200.png --at-ms 200 2>$null
target\debug\caption-cli.exe --words-json src\mock\words.json --preset kata-kotak-merah --size 1080x1920 --render-frame E:\Projects\caption-maker\target\kotak-proof\kotak-280.png --at-ms 280 2>$null
```

Output mentah:
```text
frame output=E:\Projects\caption-maker\target\kotak-proof\kotak-120.png
exit: 0
frame output=E:\Projects\caption-maker\target\kotak-proof\kotak-200.png
exit: 0
frame output=E:\Projects\caption-maker\target\kotak-proof\kotak-280.png
exit: 0
```

Path PNG:
```text
E:\Projects\caption-maker\target\kotak-proof\kotak-120.png
E:\Projects\caption-maker\target\kotak-proof\kotak-200.png
E:\Projects\caption-maker\target\kotak-proof\kotak-280.png
```

## F. Validasi

Perintah:
```text
node --check src\js\api.js
node --test src\tests\*.test.mjs
cargo fmt --all -- --check
cargo test --workspace --no-default-features
cargo clippy --workspace --all-targets -- -D warnings
git diff --stat
```

Output mentah `node --check`:
```text
api.js exit: 0
```

Output mentah `node --test`:
```text
✔ menghitung skala animasi pop sesuai keyframe (1.6982ms)
✔ menginterpolasi animasi pop untuk durasi 100 ms (0.4035ms)
✔ mengelompokkan satu kata per baris (1.1401ms)
✔ mengelompokkan dua kata per baris (0.6734ms)
✔ mengelompokkan tiga kata per baris (0.5712ms)
✔ mengembalikan array kosong untuk masukan kosong (0.3525ms)
✔ mempertahankan urutan kata dan tidak mengubah masukan (16.4522ms)
✔ menolak jumlah kata per baris di luar 1 sampai 3 (0.6035ms)
✔ menghitung rect video portrait di stage landscape (0.4222ms)
✔ menghitung rect video landscape (1.3782ms)
✔ memakai ukuran referensi portrait jika videoWidth nol (0.2431ms)
✔ menghitung bitmap canvas untuk rect portrait (1.1474ms)
✔ menghitung bitmap canvas untuk rect landscape (0.2461ms)
✔ menghitung bitmap canvas dengan DPR 2 (0.1079ms)
✔ memetakan setiap family libass ke file font yang sesuai (0.5963ms)
✔ menghitung anchor tengah caption relatif terhadap videoRect (0.2076ms)
✔ membatasi anchor agar seluruh blok dan efeknya berada di area video (0.1739ms)
✔ mengubah drag pixel menjadi persen dan membatasi posisi akhir (0.1415ms)
✔ memuat preset font dan memeriksa semua family sebelum menyelesaikan (0.5571ms)
✔ menghitung preview metrics pada geometri 1080x1920 (0.2592ms)
✔ memakai ukuran referensi portrait untuk mode audio (0.0937ms)
✔ menghitung assScale 0.6667 pada video 720x1280 (0.1044ms)
✔ marginV landscape memakai 8 persen tinggi rect (0.104ms)
✔ video persegi memakai aturan marginV landscape (0.0819ms)
✔ mengedit teks memangkas spasi tepi tanpa mengubah masukan (3.5148ms)
✔ menggabungkan dengan kata berikutnya tanpa spasi (0.5248ms)
✔ menghapus kata menghasilkan daftar baru (0.2393ms)
✔ teks dengan spasi dipecah proporsional tanpa celah atau tumpang tindih (1.4677ms)
✔ teks kosong pada edit menghapus kata (0.8678ms)
✔ mengembalikan hasil asli dengan salinan objek baru (0.2951ms)
✔ mengganti satu kata dengan kata lain tanpa mengubah waktunya (0.5362ms)
✔ mempertahankan waktu token yang sama saat jumlah kata tidak berubah (0.2489ms)
✔ menyisipkan kata di tengah dengan membagi jeda tetangga (0.357ms)
✔ menyisipkan kata di ujung kelompok tanpa mengubah waktu token yang sama (0.297ms)
✔ menghapus kata tengah atau ujung tanpa menggeser waktu kata yang tersisa (0.214ms)
✔ mengganti beberapa kata dengan satu kata dalam rentang waktu yang sama (0.2922ms)
✔ mengganti satu kata dengan beberapa kata secara proporsional (0.2665ms)
✔ membagi waktu pengganti menurut jumlah huruf Unicode (3.3498ms)
✔ teks kelompok kosong menghapus semua kata kelompok (0.2123ms)
✔ tokenisasi mengabaikan spasi ganda dan spasi tepi (0.2112ms)
✔ rentang kelompok pengganti mempertahankan waktu total dan tidak tumpang tindih (0.2506ms)
✔ applyGroupText tidak memutasi daftar kata masukan (2.0107ms)
✔ menolak gabung kata terakhir dan indeks yang tidak valid (0.6748ms)
ℹ tests 43
ℹ suites 0
ℹ pass 43
ℹ fail 0
ℹ cancelled 0
ℹ skipped 0
ℹ todo 0
ℹ duration_ms 335.6477
```

Output mentah `cargo fmt --all -- --check`:
```text
```

Output mentah `cargo test --workspace --no-default-features --quiet` (format ringkas; perintah penuh yang diminta juga berhasil di atas):
```text
running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 26 tests
..........................
test result: ok. 26 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 14 tests
..............
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 2 tests
..
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0.00s


running 8 tests
........
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 9 tests
.........
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0.00s


running 12 tests
............
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0.06s


running 25 tests
.........................
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0.01s


running 38 tests
.........i............................
test result: ok. 37 passed; 0 failed; 1 ignored; 0 measured; 0.35s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

Output mentah `cargo clippy --workspace --all-targets -- -D warnings`:
```text
Checking caption-core v0.1.0 (E:\Projects\caption-maker\caption-core)
Checking caption-tauri v0.1.0 (E:\Projects\caption-maker\src-tauri)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 32.10s
```

Output mentah `git diff --stat`:
```text
warning: in the working copy of 'src/js/api.js', CRLF will be replaced by LF the next time Git touches it
 caption-core/presets/kata-kotak-merah.json |  5 +++--
 caption-core/tests/ass_preset_tests.rs     | 29 +++++++++++++++++++++++++++++
 caption-core/tests/preset_tests.rs         |  4 ++--
 src-tauri/src/commands_tests.rs            |  2 +-
 src/js/api.js                              |  6 +++---
 5 files changed, 38 insertions(+), 8 deletions(-)
```

## Tidak bisa dijalankan / tidak diverifikasi

- App desktop tidak dijalankan untuk memeriksa tampilan kotak dan bayangan.
- Perilaku pop pada kotak belum dilihat atau diuji secara visual di aplikasi desktop. PNG dibuat sebagai bukti render pada waktu 120, 200, dan 280 ms, tetapi tidak dipakai untuk menyatakan verifikasi visual.
- Satu tes Tauri yang memerlukan sidecar FFmpeg dengan libass dan libx264 berstatus ignored pada `cargo test`.
