# REPORT_BATCH5

## Tugas 0 — Commit Batch 4

### File dan baris yang diubah
- `src/js/preview.js`: 67-101 dan 145-230.
- `src/tests/preview.test.mjs`: 5-12 dan 130-209.
- `caption-core/src/preset_render.rs`: 198, 220, 263-297.
- `caption-core/tests/ass_preset_tests.rs`: 200-239.
- Commit: `1bc3836 fix(preview): align preview scaling and ASS glow`.
- Commit hanya memuat empat file tersebut; `REPORT.md` tidak ikut.

### Diff mentah
```diff
diff --git a/caption-core/src/preset_render.rs b/caption-core/src/preset_render.rs
index 96742a3..1ca17e5 100644
--- a/caption-core/src/preset_render.rs
+++ b/caption-core/src/preset_render.rs
@@ -195,7 +195,7 @@ pub fn render_ass_with_preset(
         match preset.emphasis {
             Emphasis::None | Emphasis::Fill => {
                 if glow_layer {
-                    render_glow_events(&mut out, preset, group);
+                    render_glow_events(&mut out, preset, group, &geometry);
                 }
                 let text = match preset.emphasis {
                     Emphasis::Fill => build_fill_text(preset, group, &geometry),
@@ -217,7 +217,7 @@ pub fn render_ass_with_preset(
                         group.words.last().unwrap().end_ms
                     };
                     if glow_layer {
-                        render_glow_single(&mut out, preset, group, start, end);
+                        render_glow_single(&mut out, preset, group, start, end, &geometry);
                     }
                     let text = build_group_text(preset, group, Some(i));
                     out.push_str(&format!(
@@ -260,26 +260,37 @@ fn build_fill_text(preset: &StylePreset, group: &Group, _geom: &VideoGeometry) -
     out.trim_end().to_owned()
 }
 
-fn render_glow_events(out: &mut String, preset: &StylePreset, group: &Group) {
+fn render_glow_events(
+    out: &mut String,
+    preset: &StylePreset,
+    group: &Group,
+    geometry: &VideoGeometry,
+) {
     let g = preset.glow.as_ref().unwrap();
-    let scale = 1.0_f64; // glow radius sudah dalam px preset
     let c = ass_color_from_hex(&g.color).unwrap_or("&H00000000&".into());
-    let r = (f64::from(g.radius) * scale).round() as u32;
+    let r = (f64::from(g.radius) * geometry.scale()).round() as u32;
     let text = build_group_text(preset, group, None);
     out.push_str(&format!(
-        "Dialogue: 0,{},{},Default,,0,0,0,,{{\\bord{r}\\blur{r}\\c{c}}}{text}\n",
+        "Dialogue: 0,{},{},Default,,0,0,0,,{{\\bord{r}\\blur{r}\\c{c}\\3c{c}}}{text}\n",
         timestamp(group.start_ms()),
         timestamp(group.end_ms()),
     ));
 }
 
-fn render_glow_single(out: &mut String, preset: &StylePreset, group: &Group, start: u32, end: u32) {
+fn render_glow_single(
+    out: &mut String,
+    preset: &StylePreset,
+    group: &Group,
+    start: u32,
+    end: u32,
+    geometry: &VideoGeometry,
+) {
     let g = preset.glow.as_ref().unwrap();
     let c = ass_color_from_hex(&g.color).unwrap_or("&H00000000&".into());
-    let r = g.radius;
+    let r = (f64::from(g.radius) * geometry.scale()).round() as u32;
     let text = build_group_text(preset, group, None);
     out.push_str(&format!(
-        "Dialogue: 0,{},{},Default,,0,0,0,,{{\\bord{r}\\blur{r}\\c{c}}}{text}\n",
+        "Dialogue: 0,{},{},Default,,0,0,0,,{{\\bord{r}\\blur{r}\\c{c}\\3c{c}}}{text}\n",
         timestamp(start),
         timestamp(end),
     ));
diff --git a/caption-core/tests/ass_preset_tests.rs b/caption-core/tests/ass_preset_tests.rs
index d9c4eaa..eb4abd8 100644
--- a/caption-core/tests/ass_preset_tests.rs
+++ b/caption-core/tests/ass_preset_tests.rs
@@ -201,6 +201,10 @@ fn glow_adds_lower_layer_events() {
     let p = find_preset("baris-neon-mint").unwrap();
     let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
     let events = dialogue_lines(&ass);
+    let glow_event = events
+        .iter()
+        .find(|e| e.starts_with("Dialogue: 0,"))
+        .unwrap();
     let layer0 = events
         .iter()
         .filter(|e| e.starts_with("Dialogue: 0,"))
@@ -211,6 +215,24 @@ fn glow_adds_lower_layer_events() {
     assert_eq!(layer0, 1, "glow = 1 event layer 0");
     assert_eq!(layer1, 1, "teks utama = 1 event layer 1");
+    assert!(glow_event.contains("\\bord12\\blur12"));
+    assert!(glow_event.contains("\\c&H00C0FF5C&\\3c&H00C0FF5C&"));
+}
+
+#[test]
+fn glow_radius_scales_for_720x1280_and_sets_secondary_color() {
+    let p = find_preset("baris-neon-mint").unwrap();
+    let ass = render_ass_with_preset(&words_3(), &p, small_portrait(), &PresetOptions::default());
+    let glow_event = dialogue_lines(&ass)
+        .into_iter()
+        .find(|line| line.starts_with("Dialogue: 0,"))
+        .unwrap();
+
+    assert!(glow_event.contains("\\bord8\\blur8"), "{glow_event}");
+    assert!(
+        glow_event.contains("\\c&H00C0FF5C&\\3c&H00C0FF5C&"),
+        "{glow_event}"
+    );
 }
 
 #[test]
diff --git a/src/js/preview.js b/src/js/preview.js
index c24e7a4..7d7d949 100644
--- a/src/js/preview.js
+++ b/src/js/preview.js
@@ -64,6 +64,41 @@ export function getVideoRect(stage, video) {
   };
 }
 
+export function computePreviewMetrics({ rect, videoWidth, videoHeight, preset, settings = {} }) {
+  const hasVideoDimensions =
+    Number.isFinite(videoWidth) &&
+    videoWidth > 0 &&
+    Number.isFinite(videoHeight) &&
+    videoHeight > 0;
+  const vw = hasVideoDimensions ? videoWidth : 1080;
+  const vh = hasVideoDimensions ? videoHeight : 1920;
+  const assScale = Math.min(vw, vh) / 1080;
+  const previewScale = rect.scale * assScale;
+
+  const rawFontSize = Number(preset.font?.size ?? settings.fontSize);
+  const fontSize = Number.isFinite(rawFontSize) && rawFontSize > 0 ? rawFontSize : 20;
+  const rawOutline = Number(preset.outline ?? settings.outline);
+  const outline = Number.isFinite(rawOutline) && rawOutline >= 0 ? rawOutline : 0;
+  const rawShadow = Number(preset.shadow ?? settings.shadow);
+  const shadow = Number.isFinite(rawShadow) && rawShadow > 0 ? rawShadow * previewScale : 0;
+
+  return {
+    assScale,
+    previewScale,
+    fontSize: fontSize * previewScale,
+    outline: outline * previewScale,
+    shadow,
+    letterSpacing: Number(preset.font?.letter_spacing ?? 0) * previewScale,
+    boxPadding: Number(preset.box?.padding ?? 0) * previewScale,
+    marginLr: Number(preset.margin_lr ?? 0) * previewScale,
+    marginV:
+      vh > vw
+        ? Number(preset.margin_v ?? 0) * previewScale
+        : rect.height * 0.08,
+    glowRadius: Number(preset.glow?.radius ?? 0) * previewScale,
+  };
+}
+
 export function groupWords(words, n) {
   if (![1, 2, 3, "sentence"].includes(n)) {
     throw new RangeError("Jumlah kata per baris harus 1, 2, 3, atau sentence.");
@@ -110,29 +145,14 @@ export function renderPreview(container, words, currentMs, settings) {
   const video = document.getElementById('video-player');
   const activeVideo = video && !video.hidden ? video : null;
   const videoRect = getVideoRect(stage, activeVideo);
-  const scale = videoRect.scale;
-  const marginV = (preset.margin_v ?? 0) * scale;
-  const marginLr = (preset.margin_lr ?? 0) * scale;
-
-  const rawFontSize = Number(font?.size ?? settings.fontSize);
-  const baseFontSize = Number.isFinite(rawFontSize) && rawFontSize > 0 ? rawFontSize : 20;
-  const scaledFontSize = Math.max(12, baseFontSize * scale);
-
-  console.log("renderPreview geometry", {
-    videoWidth: activeVideo?.videoWidth ?? 0,
-    videoHeight: activeVideo?.videoHeight ?? 0,
-    stageClientWidth: stage.clientWidth,
-    stageClientHeight: stage.clientHeight,
-    scale,
-    scaledFontSize,
+  const metrics = computePreviewMetrics({
+    rect: videoRect,
+    videoWidth: activeVideo?.videoWidth,
+    videoHeight: activeVideo?.videoHeight,
+    preset,
+    settings,
   });
-
-  const rawOutline = Number(preset.outline ?? settings.outline);
-  const validOutline = Number.isFinite(rawOutline) && rawOutline >= 0 ? rawOutline : 0;
-  const scaledOutline = Math.max(0, validOutline * scale);
-
-  const shadow = preset.shadow ?? settings.shadow;
-
   // --- Ukuran dan posisi area video ---
   container.style.position = "absolute";
   container.style.left = `${videoRect.left}px`;
@@ -147,24 +167,24 @@ export function renderPreview(container, words, currentMs, settings) {
   container.style.justifyContent = "center";
   container.style.alignItems = "flex-end";
   container.style.textAlign = "center";
-  container.style.padding = `0 ${marginLr}px ${marginV}px`;
+  container.style.padding = `0 ${metrics.marginLr}px ${metrics.marginV}px`;
   container.style.pointerEvents = "none";
   container.style.zIndex = "10";
 
   // --- Font & Warna Dasar ---
   container.style.fontFamily = font ? `"${font.family}", Arial, sans-serif` : settings.font;
-  container.style.fontSize = `${scaledFontSize}px`;
+  container.style.fontSize = `${metrics.fontSize}px`;
   container.style.fontWeight = font ? (font.bold ? "700" : "400") : "";
   container.style.fontStyle = font ? (font.italic ? "italic" : "normal") : "";
-  container.style.letterSpacing = font ? `${(font.letter_spacing || 0) * scale}px` : "";
+  container.style.letterSpacing = font ? `${metrics.letterSpacing}px` : "";
   container.style.color = preset.colors?.text ?? textColor;
   container.style.textTransform = font?.uppercase ? "uppercase" : "none";
 
   // --- Outline & Glow (Hanya jika tidak ada kotak) ---
   if (!hasBox) {
-    container.style.webkitTextStroke = `${scaledOutline}px ${preset.colors?.outline ?? outlineColor}`;
-    if (glow) { container.style.textShadow = `0 0 ${Math.max(4, glow.radius * scale)}px ${glow.color}`; } else if (shadow > 0) {
-      container.style.textShadow = `0 ${shadow * scale}px ${shadow * 2 * scale}px ${shadowColor}`;
+    container.style.webkitTextStroke = `${metrics.outline}px ${preset.colors?.outline ?? outlineColor}`;
+    if (glow) { container.style.textShadow = `0 0 ${metrics.glowRadius}px ${glow.color}`; } else if (metrics.shadow > 0) {
+      container.style.textShadow = `0 ${metrics.shadow}px ${metrics.shadow * 2}px ${shadowColor}`;
     } else {
       container.style.textShadow = "none";
     }
@@ -207,7 +227,7 @@ export function renderPreview(container, words, currentMs, settings) {
 
   if (hasBox) {
     content.style.backgroundColor = preset.box.color;
-    content.style.padding = `${(preset.box.padding ?? 0) * scale}px`;
+    content.style.padding = `${metrics.boxPadding}px`;
     content.style.borderRadius = "0";
   }
diff --git a/src/tests/preview.test.mjs b/src/tests/preview.test.mjs
index 9efd066..7d13e7d 100644
--- a/src/tests/preview.test.mjs
+++ b/src/tests/preview.test.mjs
@@ -1,7 +1,12 @@
 import assert from "node:assert/strict";
 import test from "node:test";
 
-import { getVideoRect, groupWords, loadPreviewFont } from "../js/preview.js";
+import {
+  computePreviewMetrics,
+  getVideoRect,
+  groupWords,
+  loadPreviewFont,
+} from "../js/preview.js";
 
 const words = ["satu", "dua", "tiga", "empat", "lima"].map((text, index) => ({
   text,
@@ -122,3 +127,83 @@ test("memuat preset font dan memeriksa semua family sebelum menyelesaikan", asyn
     }
   }
 });
+
+const metricPreset = {
+  font: { size: 72, letter_spacing: 2 },
+  outline: 6,
+  shadow: 3,
+  box: { padding: 18 },
+  margin_v: 450,
+  margin_lr: 80,
+  glow: { radius: 12 },
+};
+
+test("menghitung preview metrics pada geometri 1080x1920", () => {
+  const metrics = computePreviewMetrics({
+    rect: { scale: 0.5, height: 960 },
+    videoWidth: 1080,
+    videoHeight: 1920,
+    preset: metricPreset,
+  });
+
+  assert.equal(metrics.assScale, 1);
+  assert.equal(metrics.previewScale, 0.5);
+  assert.equal(metrics.fontSize, 36);
+  assert.equal(metrics.outline, 3);
+  assert.equal(metrics.shadow, 1.5);
+  assert.equal(metrics.letterSpacing, 1);
+  assert.equal(metrics.boxPadding, 9);
+  assert.equal(metrics.marginLr, 40);
+  assert.equal(metrics.marginV, 225);
+  assert.equal(metrics.glowRadius, 6);
+});
+
+test("memakai ukuran referensi portrait untuk mode audio", () => {
+  const metrics = computePreviewMetrics({
+    rect: { scale: 0.5, height: 960 },
+    videoWidth: 0,
+    videoHeight: 0,
+    preset: metricPreset,
+  });
+
+  assert.equal(metrics.assScale, 1);
+  assert.equal(metrics.previewScale, 0.5);
+  assert.equal(metrics.marginV, 225);
+});
+
+test("menghitung assScale 0.6667 pada video 720x1280", () => {
+  const metrics = computePreviewMetrics({
+    rect: { scale: 0.5, height: 640 },
+    videoWidth: 720,
+    videoHeight: 1280,
+    preset: metricPreset,
+  });
+
+  assert.ok(Math.abs(metrics.assScale - 0.6667) < 0.0001);
+  assert.ok(Math.abs(metrics.previewScale - (0.5 * (720 / 1080))) < 0.0001);
+  assert.ok(Math.abs(metrics.marginV - 150) < 0.0001);
+});
+
+test("marginV landscape memakai 8 persen tinggi rect", () => {
+  const metrics = computePreviewMetrics({
+    rect: { scale: 0.5, height: 540 },
+    videoWidth: 1920,
+    videoHeight: 1080,
+    preset: metricPreset,
+  });
+
+  assert.equal(metrics.assScale, 1);
+  assert.equal(metrics.marginV, 540 * 0.08);
+});
+
+test("video persegi memakai aturan marginV landscape", () => {
+  const metrics = computePreviewMetrics({
+    rect: { scale: 0.5, height: 400 },
+    videoWidth: 1080,
+    videoHeight: 1080,
+    preset: metricPreset,
+  });
+
+  assert.equal(metrics.assScale, 1);
+  assert.equal(metrics.marginV, 400 * 0.08);
+});
```

### Output mentah validasi
Percobaan awal sebelum perbaikan Tugas 1:
```text
node : The term 'node' is not recognized as the name of a cmdlet, function, script file, or operable program. Check
the spelling of the name, or if a path was included, verify that the path is correct and try again.
At line:2 char:64
+ ... Output '=== node --test src/tests/preview.test.mjs ==='; node --test  ...
+                                                              ~~~~
    + CategoryInfo          : ObjectNotFound: (node) [], CommandNotFoundException
    + FullyQualifiedErrorId : CommandNotFoundException
```
```text
cargo fmt --all -- --check
EXIT=0
```
```text
    Checking caption-tauri v0.1.0 (E:\Projects\caption-maker\src-tauri)
error: try not to call a closure in the expression where it is declared
   --> src-tauri\src\commands_tests.rs:150:36
    |
150 |       let result: Result<(), &str> = (|| {
    |  ____________________________________^
151 | |         let _lease = manager.reserve().unwrap();
152 | |         Err("simulated burn failure")
153 | |     })();
    | |________^
    |
    = help: for further information visit https://rust-lang.github.io/rust-clippy/rust-1.100.0/index.html#redundant_closure_call
    = note: `-D clippy::redundant-closure-call` implied by `-D warnings`
    = help: to override `-D warnings` add `#[allow(clippy::redundant_closure_call)]`
help: try doing something like
    |
150 ~     let result: Result<(), &str> = {
151 +         let _lease = manager.reserve().unwrap();
152 +         Err("simulated burn failure")
153 ~     };
    |
error: could not compile `caption-tauri` (lib test) due to 1 previous error
EXIT=101
```
Setelah perbaikan Clippy pada Tugas 1, validasi yang dijalankan ulang sebelum commit:
```text
cargo fmt --all -- --check
EXIT=0
```
```text
    Checking caption-tauri v0.1.0 (E:\Projects\caption-maker\src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2m 10s
EXIT=0
```
```text
cargo test --workspace --no-default-features
EXIT=0
```
Baris hasil tes dari validasi akhir dicantumkan pada bagian Validasi akhir.

### Tidak bisa dijalankan / tidak diverifikasi
- `node --test` tidak dapat dijalankan karena `node` tidak ada di PATH; tidak diunduh.
- Percobaan awal Clippy gagal pada `commands_tests.rs:150`, sehingga Batch 4 belum dikomit pada percobaan tersebut. Setelah Tugas 1 memperbaiki penyebab itu, formatting, Clippy, dan tes Rust lulus; Batch 4 kemudian dikomit.

## Tugas 1 — Clippy

### File dan baris yang diubah
- `src-tauri/src/commands_tests.rs`: 150-153.
- Commit: `99496e5 fix(tests): simplify burn manager error block`.

### Diff mentah
```diff
diff --git a/src-tauri/src/commands_tests.rs b/src-tauri/src/commands_tests.rs
index 6e8051c..cf2e111 100644
--- a/src-tauri/src/commands_tests.rs
+++ b/src-tauri/src/commands_tests.rs
@@ -147,10 +147,10 @@ fn transcribe_manager_rejects_parallel_transcriptions() {
 #[test]
 fn burn_manager_releases_lease_after_error() {
     let manager = BurnManager::default();
-    let result: Result<(), &str> = (|| {
+    let result: Result<(), &str> = {
         let _lease = manager.reserve().unwrap();
         Err("simulated burn failure")
-    })();
+    };
 
     assert_eq!(result, Err("simulated burn failure"));
     assert!(manager.reserve().is_ok());
```

### Output mentah validasi
```text
cargo fmt --all -- --check
EXIT=0
```
```text
    Checking caption-tauri v0.1.0 (E:\Projects\caption-maker\src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 15s
EXIT=0
```
```text
test commands_tests::burn_manager_releases_lease_after_error ... ok
test result: ok. 32 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 1.40s
EXIT=0
```

### Tidak bisa dijalankan / tidak diverifikasi
- Perilaku tes diverifikasi oleh `cargo test`; tes integrasi burn yang memerlukan FFmpeg sidecar ditandai ignored oleh tes.

## Tugas 2 — Lebar bungkus teks

### File dan baris yang diubah
- `src/css/app.css`: 1472.
- Commit: `5e7e375 fix(preview): match subtitle content width to ASS margins`.

### Diff mentah
```diff
diff --git a/src/css/app.css b/src/css/app.css
index 2573b36..2ec9045 100644
--- a/src/css/app.css
+++ b/src/css/app.css
@@ -1469,7 +1469,7 @@ details.advanced-settings .advanced-content {
 
 .subtitle-content {
   display: inline-block !important;
-  max-width: 90% !important;
+  max-width: 100% !important;
   word-wrap: break-word !important;
   position: relative;
   z-index: 2;
```

### Output mentah validasi
```text
git diff --check -- src/css/app.css
EXIT=0
```

### Tidak bisa dijalankan / tidak diverifikasi
- Tes JavaScript dicantumkan pada Validasi akhir; tidak tersedia `node` di PATH.

## Tugas 3 — Shadow keras seperti ASS

### File dan baris yang diubah
- `src/js/preview.js`: 187.
- Commit: `364d259 fix(preview): match hard shadow to ASS offsets`.

### Diff mentah
```diff
diff --git a/src/js/preview.js b/src/js/preview.js
index 7d7d949..c8e09d4 100644
--- a/src/js/preview.js
+++ b/src/js/preview.js
@@ -184,7 +184,7 @@ export function renderPreview(container, words, currentMs, settings) {
   if (!hasBox) {
     container.style.webkitTextStroke = `${metrics.outline}px ${preset.colors?.outline ?? outlineColor}`;
     if (glow) { container.style.textShadow = `0 0 ${metrics.glowRadius}px ${glow.color}`; } else if (metrics.shadow > 0) {
-      container.style.textShadow = `0 ${metrics.shadow}px ${metrics.shadow * 2}px ${shadowColor}`;
+      container.style.textShadow = `${metrics.shadow}px ${metrics.shadow}px 0 ${shadowColor}`;
     } else {
       container.style.textShadow = "none";
     }
```

### Output mentah validasi
```text
git diff --check -- src/js/preview.js
EXIT=0
```
```text
E:\Projects\caption-maker\src\js\preview.js:186:    if (glow) { container.style.textShadow = `0 0 ${metrics.glowRadius}px ${glow.color}`; } else if (metrics.shadow > 0) {
E:\Projects\caption-maker\src\js\preview.js:187:      container.style.textShadow = `${metrics.shadow}px ${metrics.shadow}px 0 ${shadowColor}`;
```

### Tidak bisa dijalankan / tidak diverifikasi
- `node --test` tidak dapat dijalankan karena `node` tidak ada di PATH.
- Perubahan tidak mengubah cabang glow; baris glow pada keluaran `rg` tetap `0 0 ...`.

## Tugas 4 — Outline di luar huruf

### File dan baris yang diubah
- `src/js/preview.js`: 185-194.
- Commit: `34bf849 fix(preview): paint outline outside text fill`.

### Diff mentah
```diff
diff --git a/src/js/preview.js b/src/js/preview.js
index c8e09d4..c0d3b54 100644
--- a/src/js/preview.js
+++ b/src/js/preview.js
@@ -182,7 +182,8 @@ export function renderPreview(container, words, currentMs, settings) {
 
   // --- Outline & Glow (Hanya jika tidak ada kotak) ---
   if (!hasBox) {
-    container.style.webkitTextStroke = `${metrics.outline}px ${preset.colors?.outline ?? outlineColor}`;
+    container.style.webkitTextStroke = `${metrics.outline * 2}px ${preset.colors?.outline ?? outlineColor}`;
+    container.style.paintOrder = "stroke fill";
     if (glow) { container.style.textShadow = `0 0 ${metrics.glowRadius}px ${glow.color}`; } else if (metrics.shadow > 0) {
       container.style.textShadow = `${metrics.shadow}px ${metrics.shadow}px 0 ${shadowColor}`;
     } else {
@@ -190,6 +191,7 @@ export function renderPreview(container, words, currentMs, settings) {
     }
   } else {
     container.style.webkitTextStroke = "0px";
+    container.style.paintOrder = "normal";
     container.style.textShadow = "none";
   }
```

### Output mentah validasi
```text
git diff --check -- src/js/preview.js
EXIT=0
```

### Tidak bisa dijalankan / tidak diverifikasi
- Tidak ada tes murni untuk urutan paint dan lebar stroke; perilaku ini berada pada DOM/CSS. Tidak ditambahkan harness DOM.
- `node --test` tidak dapat dijalankan karena `node` tidak ada di PATH.

## Tugas 5 — Verifikasi ASS (read-only)

### File dan baris yang diubah
- Tidak ada file source yang diubah.
- Keluaran CLI: `out/baris-isi-kuning.ass` dan `out/baris-neon-mint.ass`.

### Diff mentah
```text
Tidak ada diff source untuk tugas read-only ini.
```

### Output mentah validasi
Perintah:
```text
.\target\debug\caption-cli.exe --preset baris-isi-kuning --words-json src/mock/words.json --size 1080x1920 -o out/baris-isi-kuning.ass
words=48 groups=16 output=out/baris-isi-kuning.ass
EXIT=0
```
```text
.\target\debug\caption-cli.exe --preset baris-neon-mint --words-json src/mock/words.json --size 1080x1920 -o out/baris-neon-mint.ass
words=48 groups=16 output=out/baris-neon-mint.ass
EXIT=0
```
`baris-isi-kuning`:
```text
Style: Default,Poppins,68,&H00FFFFFF&,&H00FFFFFF&,&H00000000&,&H7F000000&,-1,0,0,0,100,100,0,0,1,7,3,2,80,80,450,1
Dialogue: 1,0:00:00.12,0:00:01.40,Default,,0,0,0,,{\kf24}lagi {\k3}{\kf35}cari {\k3}{\kf63}kaution
Dialogue: 1,0:00:01.40,0:00:02.55,Default,,0,0,0,,{\kf54}ringan {\k12}{\kf21}tapi {\k14}{\kf14}bikin
Dialogue: 1,0:00:02.68,0:00:03.96,Default,,0,0,0,,{\kf36}kulit {\k2}{\kf46}flawless {\k25}{\kf19}ini
PrimaryColour=&H00FFFFFF&
SecondaryColour=&H00FFFFFF&
PrimaryEqualsSecondary=True
DialogueHasColorTag=False
ContainsLiteralFFE14D=False
ContainsASSColor004DE1FF=False
```
`baris-neon-mint`:
```text
Style: Default,Poppins ExtraBold,72,&H00C0FF5C&,&H00C0FF5C&,&H00203800&,&HFF000000&,0,0,0,0,100,100,2,0,1,6,0,2,80,80,450,1
Dialogue: 0,0:00:00.12,0:00:01.40,Default,,0,0,0,,{\bord12\blur12\c&H00C0FF5C&\3c&H00C0FF5C&}LAGI CARI KAUTION
Dialogue: 1,0:00:00.12,0:00:01.40,Default,,0,0,0,,LAGI CARI KAUTION
Dialogue: 0,0:00:01.40,0:00:02.55,Default,,0,0,0,,{\bord12\blur12\c&H00C0FF5C&\3c&H00C0FF5C&}RINGAN TAPI BIKIN
```
Jawaban berdasarkan isi file ASS:
```text
(a) YA
(b) TIDAK
(c) TIDAK
```

### Tidak bisa dijalankan / tidak diverifikasi
- File ASS diperiksa sebagai teks; tidak dilakukan penilaian visual.

## Tugas 6 — Housekeeping

### File dan baris yang diubah
- `.gitignore`: 12.
- Tidak ada file `.bak`, `fix_api.js`, atau `fix_api.py` yang dihapus.

### Diff mentah
```diff
diff --git a/.gitignore b/.gitignore
index 2eae6c1..2a6808a 100644
--- a/.gitignore
+++ b/.gitignore
@@ -9,3 +9,4 @@
 /src-tauri/target/
 /src-tauri/gen/
 src-tauri/binaries/*.exe
+*.bak
```

### Output mentah validasi
```text
.gitignore:12:*.bak	src-tauri/tauri.conf.json.bak
.gitignore:12:*.bak	src/js/preview.js.bak
True
True
True
True
```

### Tidak bisa dijalankan / tidak diverifikasi
- Keberadaan file `.bak`, `fix_api.js`, dan `fix_api.py` diverifikasi dengan `Test-Path`; tidak ada file yang dihapus.

## Validasi akhir

### `node --test src/tests/preview.test.mjs`
```text
node : The term 'node' is not recognized as the name of a cmdlet, function, script file, or operable program. Check
the spelling of the name, or if a path was included, verify that the path is correct and try again.
At line:2 char:1
+ node --test src/tests/preview.test.mjs
+ ~~~~
    + CategoryInfo          : ObjectNotFound: (node:String) [], CommandNotFoundException
    + FullyQualifiedErrorId : CommandNotFoundException
```

### `cargo fmt --all -- --check`
```text
EXIT=0
```

### `cargo clippy --workspace --all-targets -- -D warnings`
```text
    Checking caption-tauri v0.1.0 (E:\Projects\caption-maker\src-tauri)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 08s
EXIT=0
```

### `cargo test --workspace --no-default-features`
Baris hasil mentah; tidak ada nama tes gagal:
```text
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 20 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
test result: ok. 32 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 2.02s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Tidak bisa dijalankan / tidak diverifikasi
- `node --test` tidak berjalan karena Node.js tidak ditemukan di PATH; Node tidak diunduh.
- Tugas 4 tidak diuji di browser/Tauri; tidak ada tes DOM dalam rangkaian yang tersedia.
- Tidak dilakukan penilaian visual terhadap preview atau hasil ASS.
- Tidak ada push yang dilakukan.
