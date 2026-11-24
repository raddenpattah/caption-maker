# Catatan Video Preview dan Gaya Caption

Dokumen ini merangkum jalur preview video dan renderer gaya caption yang ada di source saat diperiksa. Isinya ditujukan sebagai panduan sebelum mengubah ukuran atau penempatan caption. Ini bukan spesifikasi desain baru dan tidak mengubah perilaku aplikasi.

## Ringkasan jalur preview

1. `src/index.html` menyediakan `#preview-stage`, elemen media `#video-player`/`#audio-player`, dan `#preview-subtitle` sebagai lapisan caption.
2. `src/css/app.css` mengatur stage sebagai wadah relatif yang terpotong (`overflow: hidden`), dan elemen media menggunakan `object-fit: contain`.
3. `src/js/main.js` merespons perubahan waktu dengan `renderPreviewText()`. Fungsi itu mengambil preset aktif, menunggu font preset dimuat melalui `loadPreviewFont()`, kemudian memanggil `renderPreview()`.
4. `src/js/preview.js` menghitung persegi panjang video yang tampil, metrik skala, posisi container, dan elemen tiap kata.
5. Untuk video hasil burn, `showBurnedPreview()` menyembunyikan `#preview-subtitle`; `renderPreviewText()` juga langsung kembali jika `burnedPreviewActive`. Jadi preview burn menampilkan video yang caption-nya sudah tertanam, bukan overlay CSS kedua.
6. Ketika ukuran stage berubah, `ResizeObserver` di `src/js/main.js` memanggil ulang `renderPreviewText()` selama preview sumber aktif.

## Geometri video dan koordinat

### Ukuran stage dan video yang benar-benar tampil

`getVideoRect(stage, video)` di `src/js/preview.js` menghitung bentuk `contain`:

- Menggunakan `video.videoWidth` dan `video.videoHeight` jika kedua dimensi positif.
- Jika dimensi video tidak tersedia (misalnya audio), memakai referensi `1080 x 1920`.
- `scale = min(stage.clientWidth / videoWidth, stage.clientHeight / videoHeight)`.
- `width` dan `height` adalah ukuran video sesudah diskalakan.
- `left` dan `top` memusatkan persegi panjang video di stage.

`renderPreview()` meletakkan `#preview-subtitle` dengan `position: absolute` pada `left`, `top`, `width`, dan `height` hasil `getVideoRect()`. Artinya, offset caption seharusnya dihitung relatif ke area gambar video ini, bukan selalu relatif ke seluruh stage. Stage dapat memiliki ruang kosong letterbox/pillarbox ketika rasio stage dan video berbeda.

`src/js/main.js` juga mengatur `aspect-ratio` stage dari dimensi video pada `updateStageAspectRatio()`. Karena CSS media tetap memakai `object-fit: contain`, perubahan layout/aspect ratio harus diuji baik sebelum maupun sesudah metadata video tersedia.

### Skala preview dan skala ASS

`computePreviewMetrics()` di `src/js/preview.js` menggunakan:

- `assScale = min(videoWidth, videoHeight) / 1080`.
- Untuk dimensi yang tidak tersedia, referensinya `1080 x 1920`, sehingga `assScale = 1`.
- `previewScale = rect.scale * assScale`, dengan `rect.scale` berasal dari ukuran gambar yang pas di stage.

Preview mengalikan ukuran font, outline, shadow, letter spacing, padding kotak, margin kiri/kanan, dan radius glow dengan `previewScale`. Nilai margin bawah (`marginV`) berbeda menurut orientasi:

- Portrait: `preset.margin_v * previewScale`.
- Landscape atau persegi: `rect.height * 0.08`; ini sudah merupakan ukuran pixel layar dan tidak dikalikan lagi.

Perhitungan ASS terkait berada di `caption-core/src/preset_render.rs`, fungsi `build_style_line()`:

- PlayResX/PlayResY diisi dengan ukuran video `VideoGeometry` yang diberikan kepada renderer.
- Ukuran font, outline, shadow, letter spacing, padding kotak, dan margin kiri/kanan dikalikan `VideoGeometry::scale()`.
- `VideoGeometry::scale()` adalah sisi pendek video dibagi 1080.
- Untuk portrait, margin vertikal adalah `preset.margin_v * scale`.
- Untuk landscape/persegi, margin vertikal adalah 8% tinggi geometri video.

Karena preview memakai `rect.scale * assScale`, sedangkan ASS memakai skala relatif geometri video, jangan mencampur nilai pixel preview dan nilai unit ASS. Sebelum mengubah posisi, pastikan formula yang dipakai tetap mengonversi nilai preset ke sistem koordinat yang sesuai.

## Penempatan caption saat ini

### Preview CSS

`renderPreview()` di `src/js/preview.js` melakukan hal-hal berikut:

- Container preview memenuhi persegi panjang gambar video (`left/top/width/height` dari `getVideoRect()`).
- Container memakai flex, `justifyContent = "center"` dan `alignItems = "flex-end"`.
- Padding kiri/kanan memakai `metrics.marginLr`; padding bawah memakai `metrics.marginV`.
- Konten berada di tengah secara horizontal dan bagian bawah container menjadi dasar penempatan vertikal.
- `pointerEvents = "none"` agar layer caption tidak mengambil klik video/kontrol.
- CSS awal `.subtitle-layer` memberi posisi fallback, tetapi posisi inline yang ditetapkan `renderPreview()` mengambil alih nilai left/top/width/bottom dan transform yang relevan.

Saat ini preset bawaan yang diperiksa semuanya memakai `alignment: 2`, yang berarti posisi bawah-tengah ASS. Namun preview tidak membaca `preset.alignment` untuk menentukan alignment; ia selalu menempatkan konten di tengah bawah. Jika nilai alignment preset kelak berubah, renderer ASS dan preview dapat berbeda kecuali logika preview juga diselaraskan.

### Kontrol posisi lama

`position-select` masih ada di `src/index.html`, dan nilainya disalin ke `settings.position` di `src/js/main.js`. Namun pada source saat ini:

- `renderPreview()` tidak membaca `settings.position`.
- Tidak ditemukan assignment `data-position` pada `#preview-subtitle`.
- CSS `.subtitle-layer[data-position="top|middle|bottom"]` hanya berlaku jika atribut itu disetel.
- Penempatan inline dari `renderPreview()` menetapkan posisi berdasarkan area video dan margin bawah.

Jangan menganggap dropdown posisi saat ini sebagai sumber posisi caption yang aktif sebelum menelusuri jalur setting dan atribut DOM. Mengubah aturan CSS `[data-position]` saja mungkin tidak mengubah posisi preview karena style inline dan struktur container digunakan untuk penempatan utama.

## Gaya caption dan efek

Preset yang dikirim ke frontend melalui `PresetSummary` dalam `src-tauri/src/commands.rs` mencakup font, warna, outline, shadow, margin, glow, emphasis, emphasis scale, box, dan jumlah kata per baris. Data mock browser berada di `src/js/api.js`; bila mengubah field gaya preset, periksa kedua jalur agar uji Tauri dan mode browser konsisten.

`renderPreview()` memakai:

- Font family/size/bold/italic/uppercase dari preset; `loadPreviewFont()` menunggu font sebelum render, kemudian render ulang.
- Warna teks/highlight/outline/shadow dari preset.
- Outline melalui `-webkit-text-stroke`.
- Shadow keras dengan offset X/Y sama dan blur nol, kecuali glow digunakan.
- Glow CSS melalui `text-shadow` dengan radius terukur.
- Box sebagai `backgroundColor` pada `.subtitle-content`, padding `preset.box.padding * previewScale`, dan radius 0.
- Emphasis kata aktif melalui fill, scale, underline, atau warna aktif menurut `preset.emphasis`.

Renderer ASS di `caption-core/src/preset_render.rs` mempunyai representasi yang serupa tetapi tidak identik:

- `build_style_line()` memetakan box ke `BorderStyle=3`; tanpa box, outline memakai `BorderStyle=1`.
- Layer glow dibuat pada Dialogue layer 0. Radius glow dikalikan `geometry.scale()` dan tag warna memuat `\\c` serta `\\3c`.
- Dialogue utama ditulis pada layer 1.
- Emphasis dan transform teks diterjemahkan menjadi override tags ASS.
- Alignment diambil dari `preset.alignment`; preview sekarang tetap menempatkan caption bawah-tengah.

CSS preview adalah pendekatan browser terhadap ASS/libass, bukan hasil render libass. Perubahan stroke, line-wrap, font metrics, spacing, shadow, glow, box, alignment, atau margin dapat menghasilkan perbedaan di antara preview dan burn meskipun nilai preset sama.

## Checklist sebelum mengubah posisi gaya caption

1. **Tentukan sistem koordinat yang dimaksud.** Pastikan posisi relatif terhadap stage, gambar video yang benar-benar tampil (`videoRect`), atau output video ASS. Untuk menyamai hasil burn, gunakan area gambar dan rasio aktual, bukan ukuran seluruh panel.
2. **Periksa geometri dan orientasi.** Uji video portrait di stage landscape, video landscape, video persegi, ukuran stage kecil/besar, dan audio/tidak adanya dimensi video. Pertimbangkan rotasi video: backend memproses dimensi hasil probe/rotasi; browser melaporkan dimensi intrinsic media.
3. **Pertahankan satu sumber perhitungan skala.** Jangan menghitung ulang skala berdasarkan `stage.clientWidth / videoWidth` secara terpisah. Gunakan `getVideoRect()` dan `computePreviewMetrics()` agar padding, margin, font, outline, shadow, glow, serta box mengikuti geometri yang sama.
4. **Jangan mencampur margin bawah portrait dan landscape.** Portrait memakai nilai preset yang diskalakan; landscape/persegi memakai 8% tinggi area video. Perubahan posisi harus memutuskan apakah aturan ini tetap berlaku dan menyamakan frontend/ASS bila diperlukan.
5. **Selaraskan alignment ASS dengan DOM.** `preset.alignment` menentukan Alignment ASS; preview saat ini flex-center/bottom. Jika alignment bukan bawah-tengah, tentukan pemetaan alignment numpad 1-9 ke anchor horizontal/vertikal, lalu implementasikan konsisten tanpa mengandalkan CSS selector yang atributnya tidak pernah disetel.
6. **Bedakan margin dan padding.** `margin_lr` adalah jarak tepi caption ke sisi video. Pada preview, ia menjadi padding container kiri/kanan; pada ASS, ia menjadi MarginL/MarginR style. `margin_v` adalah margin penempatan vertikal, bukan padding box.
7. **Perhatikan batas teks dan wrapping.** `.subtitle-content` memiliki batas lebar 100%; padding container mengurangi ruang teks. Perubahan lebar/posisi dapat mengubah line-wrap, terutama pada panel kecil dan font besar. Jangan mengubah max-width atau padding tanpa menguji preset kata/baris dan box.
8. **Jaga efek tepi.** Outline digambar dengan stroke browser; box memakai background dan padding; glow dan shadow punya jalur berbeda. Perubahan anchor/container dapat memotong stroke/glow karena stage `overflow: hidden`, terutama dekat tepi video.
9. **Jaga z-index dan input.** Preview subtitle harus tetap berada di atas video, di bawah elemen UI yang memang ditumpuk di atasnya, dan `pointer-events: none` harus dipertahankan agar seek/play tetap dapat digunakan.
10. **Jaga lifecycle render.** Perubahan ukuran harus memicu render ulang melalui `ResizeObserver`; perubahan preset, media, waktu, atau font juga harus memperbarui layer. Jangan membuat observer ganda atau merender overlay ketika burned preview aktif.
11. **Uji pergantian source/burn.** Preview sumber membutuhkan overlay CSS; video hasil burn sudah memiliki caption tertanam dan `#preview-subtitle` harus tetap tersembunyi untuk menghindari caption ganda.
12. **Pastikan jalur data setara.** Periksa nilai preset di JSON, DTO Rust, mock `api.js`, dan renderer. Jangan menambah nilai UI yang hanya diterapkan pada preview tetapi tidak diteruskan ke ASS bila tujuannya mengubah posisi hasil ekspor/burn.
13. **Jangan bergantung pada kontrol lama tanpa verifikasi.** `settings.position` ada, tetapi belum digunakan oleh `renderPreview()`. Putuskan secara eksplisit apakah posisi baru berasal dari field preset, kontrol pengguna, atau keduanya; tentukan prioritas saat keduanya tersedia.
14. **Verifikasi lewat angka dan keluaran, bukan hanya kesan visual.** Catat `videoRect`, `previewScale`, margin, dan posisi aktual; uji `computePreviewMetrics()` untuk portrait, landscape, persegi, video resolusi berbeda, serta fallback audio. Untuk burn, periksa baris `Style:`/PlayRes dan hasil ASS atau frame render.
15. **Periksa CSS override.** `src/css/app.css` mendefinisikan `.preview-stage`/`.subtitle-layer` di beberapa bagian, termasuk aturan layout akhir dan aturan `.subtitle-layer`/`.subtitle-content` dekat akhir file. Style inline di `renderPreview()` juga mengalahkan deklarasi stylesheet tertentu. Saat mengubah posisi, cari semua selector yang cocok dan pastikan tidak ada override tak disengaja.

## File dan simbol utama untuk ditelusuri

- `src/js/preview.js`: `getVideoRect()`, `computePreviewMetrics()`, `renderPreview()`, dan `loadPreviewFont()`.
- `src/js/main.js`: `renderPreviewText()`, `observePreviewStageResize()`, `updateStageAspectRatio()`, `showBurnedPreview()`, dan `showSourcePreview()`.
- `src/css/app.css`: `.preview-stage`, `.media-player`, `.subtitle-layer`, `.subtitle-content`, `.karaoke-word`, serta aturan layout/responsif preview.
- `caption-core/src/preset_render.rs`: `VideoGeometry`, `build_style_line()`, `render_ass_with_preset()`, `render_glow_events()`, dan `render_glow_single()`.
- `caption-core/src/video.rs`: geometri/probe video, build argumen filter ASS, serta font yang disiapkan untuk burn.
- `src-tauri/src/commands.rs`: DTO `PresetSummary` dan pemetaan daftar preset ke frontend.
- `caption-core/presets/*.json`: sumber nilai gaya, alignment, margin, dan efek preset.
- `src/tests/preview.test.mjs`: tes unit geometri video, metrik, skala/margin, serta pemuatan font.

## Ruang lingkup pemeriksaan

Dokumen ini dibuat dari pembacaan source saat ini. Tidak ada source, CSS, preset, test, backend, maupun konfigurasi aplikasi yang diubah. Tidak ada build atau test yang dijalankan untuk membuat catatan ini.

---

# Kajian Kelayakan: Posisi Caption Draggable

Kajian ini membahas rancangan untuk memindahkan seluruh blok caption di preview dengan drag. Ini adalah pemeriksaan source dan simulasi teknis saja; tidak ada implementasi yang dilakukan dalam tahap ini.

## 1. Kondisi source sekarang

- `#preview-stage` di `src/index.html` berisi video/audio, `#preview-subtitle`, dan watermark. `.subtitle-content` dibuat ulang oleh `renderPreview()` di `src/js/preview.js` untuk setiap grup kata aktif. Kata-kata tetap berupa child `.karaoke-word`, sehingga gaya dan emphasis berada pada span kata.
- `getVideoRect(stage, video)` memakai `clientWidth/clientHeight` stage dan dimensi intrinsic video. Skala contain adalah `min(stageWidth/videoWidth, stageHeight/videoHeight)`; hasil `{left, top, width, height, scale}` memusatkan gambar di dalam stage. Dimensi fallback untuk audio adalah 1080x1920.
- `computePreviewMetrics()` menghitung `assScale = min(vw, vh) / 1080`, lalu `previewScale = rect.scale * assScale`. Font, outline, shadow, letter spacing, padding box, margin kiri/kanan, dan radius glow mengikuti skala itu. Margin bawah portrait adalah `preset.margin_v * previewScale`; landscape/persegi memakai 8% tinggi gambar preview.
- `renderPreview()` membuat `#preview-subtitle` memenuhi `videoRect`, memakai flex tengah-bawah, dan mengatur padding kiri/kanan serta bawah dari metrics. Posisi yang dihasilkan mengikuti preset margin bawah, bukan koordinat drag.
- `renderPreview()` menetapkan inline style pada container setiap render, lalu mengganti `innerHTML` dan membuat ulang `.subtitle-content` serta `.karaoke-word`. `updateClock()` memanggil `renderPreviewText()` ketika waktu berubah; resize stage juga memicu render ulang. Karena itu perubahan posisi yang hanya disimpan pada style DOM sementara dapat tertimpa pada timeupdate atau pergantian grup.
- `#preview-subtitle` mendapat `pointer-events: none` secara inline. Elemen transport video berada di luar stage; tag `<video>` tidak memiliki native controls di markup. Untuk menangkap drag caption, bagian blok yang dapat menjadi target pointer harus menerima pointer event, tetapi area kosong lain pada stage tetap sebaiknya meneruskan klik.
- Saat burned preview aktif, `showBurnedPreview()` menyembunyikan `#preview-subtitle`; `renderPreviewText()` juga tidak menggambar overlay dalam mode tersebut. Perilaku ini perlu tetap dijaga.
- Posisi/configuration yang ada bukan penyimpanan koordinat: `position-select` disalin ke `settings.position`, tetapi `renderPreview()` tidak memakai setting itu. `createState()` di `src/js/state.js` dan `selectedPresetId` di `src/js/main.js` adalah state runtime. Saya tidak menemukan penyimpanan posisi drag persisten. Preset bawaan menyimpan `alignment`, `margin_v`, dan `margin_lr`; belum ada `x/y` persentase.

## 2. File dan fungsi yang perlu diubah

Tidak ada perubahan dilakukan sekarang. Untuk implementasi di tahap berikutnya, cakupan minimum yang mungkin diperlukan:

- `src/index.html`: hanya jika diperlukan affordance atau keterangan aksesibilitas untuk area drag; struktur `#preview-subtitle` dan `.subtitle-content` sudah tersedia.
- `src/js/preview.js`:
  - `getVideoRect()` tetap menjadi sumber batas dan rasio gambar.
  - `computePreviewMetrics()` tetap menghitung ukuran gaya/margin, bukan lokasi drag.
  - `renderPreview()` perlu menerima posisi opsional dan menempatkan `.subtitle-content` dengan anchor tengah relatif terhadap videoRect, sambil tidak mengubah span kata atau style preset.
- `src/js/main.js`:
  - `renderPreviewText()` perlu mengirim posisi aktif ke `renderPreview()`.
  - Listener pointer drag sebaiknya dipasang sekali pada elemen yang stabil, bukan pada `.subtitle-content` yang dibuang dan dibuat ulang.
  - `observePreviewStageResize()`/render ulang harus menerapkan ulang posisi relatif yang tersimpan.
  - Jalur `showBurnedPreview()` tetap mempertahankan subtitle overlay tersembunyi.
- `src/css/app.css`: selector pointer-event pada layer/konten, cursor, dan pencegahan seleksi teks saat drag. Jangan mengaktifkan hit-testing pada seluruh stage/layer bila hanya blok caption yang harus draggable.
- Persistensi: `src/js/state.js` atau state UI terdekat dapat memegang koordinat runtime. Jika posisi harus bertahan setelah aplikasi ditutup, perlu dipilih mekanisme penyimpanan baru; saat ini tidak ada mekanisme preferensi posisi yang ditemukan.
- Integrasi hasil ASS/burn:
  - `src-tauri/src/commands.rs` untuk menerima dan meneruskan posisi opsional pada jalur render/burn yang sesuai.
  - `caption-core/src/preset_render.rs` untuk menghasilkan posisi pada setiap Dialogue.
  - Tipe/payload frontend di `src/js/api.js` dan request `src/js/main.js` perlu dikirimkan konsisten.
  - Preset JSON tidak perlu diubah untuk posisi pengguna jika posisi diperlakukan sebagai override runtime, bukan perubahan gaya bawaan.

Daftar ini adalah rancangan cakupan, bukan instruksi untuk menambah seluruh file sekaligus. Tahap pertama bisa dibatasi pada tiga file frontend (`main.js`, `preview.js`, `app.css`) dan tidak menyentuh ASS.

## 3. Desain interaksi drag

Rancangan minimal yang kompatibel dengan render berulang:

1. Gunakan Pointer Events (`pointerdown`, `pointermove`, `pointerup`, `pointercancel`) agar satu jalur menangani mouse dan dapat diperluas ke pointer lain.
2. Pada `pointerdown`, mulai drag hanya bila target berada di dalam `#preview-subtitle .subtitle-content`, ada grup caption aktif, dan bukan burned preview. Simpan posisi pointer awal dan koordinat anchor awal.
3. Gunakan capture pada elemen stabil seperti `#preview-stage` agar `pointermove`/`pointerup` tetap diterima jika pointer meninggalkan teks atau konten dirender ulang. Jangan memasang listener baru pada child untuk setiap frame.
4. Abaikan gerakan kurang dari threshold kecil (misalnya 3-5 CSS px) bila perlu membedakan klik dari drag. Setelah drag dimulai, cegah seleksi teks dan atur cursor `grabbing`; pada kondisi normal gunakan `grab`.
5. Untuk setiap gerakan, hitung posisi anchor baru berdasarkan selisih pointer, bukan mengubah ukuran atau style kata. Update hanya posisi container/konten sehingga emphasis, text, font, outline, shadow, box, glow, dan timing tetap utuh.
6. Ketika pointer dilepas atau dibatalkan, lepaskan capture dan simpan koordinat hasil clamp. Mouseup di luar konten tidak boleh meninggalkan status drag tertahan.
7. Drag hanya dibatasi saat pointerdown pada blok caption. Area video lain tidak boleh diambil alih event; tombol transport berada di luar stage dan tetap dapat digunakan.
8. State drag harus berada di luar node `.subtitle-content`, karena `renderPreview()` mengganti node tersebut. Pada setiap render/timeupdate, koordinat tersimpan dipakai kembali. Drag tidak mengubah `currentMs`, words, atau state playback.

Catatan konflik event: parent saat ini `pointer-events:none`. Anak `.subtitle-content` dapat dibuat sebagai target pointer secara eksplisit, sementara parent listener menerima bubbling dari target tersebut; alternatifnya delegasi pada stage perlu memeriksa `event.target.closest(".subtitle-content")`. Pastikan dengan browser/WebView test bahwa pilihan CSS ini bekerja pada target platform. Jangan mengubah seluruh `#preview-stage` menjadi draggable, karena itu akan mengganggu interaksi video.

## 4. Desain koordinat

Simpan satu anchor tengah blok sebagai `xPercent` dan `yPercent`, keduanya 0..100 dan relatif terhadap `videoRect`, bukan stage. Posisi awal yang aman adalah mempertahankan letak preview lama: setelah blok dirender dan dimensinya tersedia, ukur pusat blok saat ini, lalu konversikan pusat itu menjadi persentase terhadap `videoRect`. Ini menghindari perubahan posisi awal ketika fitur diperkenalkan.

Untuk koordinat pointer klien `(clientX, clientY)` dan area video `R = {left, top, width, height}`:

```text
xPercent = 100 * (clientX - R.left) / R.width
yPercent = 100 * (clientY - R.top) / R.height
```

Untuk drag, lebih stabil menghitung perpindahan dari pointerdown:

```text
xNext = xStart + 100 * (clientX - pointerDownX) / R.width
yNext = yStart + 100 * (clientY - pointerDownY) / R.height
```

Kemudian clamp berdasarkan dimensi aktual blok agar seluruh blok tetap di dalam gambar:

```text
xMin = 100 * blockWidth / (2 * R.width)
xMax = 100 - xMin
yMin = 100 * blockHeight / (2 * R.height)
yMax = 100 - yMin
x = clamp(xNext, xMin, xMax)
y = clamp(yNext, yMin, yMax)
```

Clamp anchor ke rentang 0..100 saja tidak cukup: pusat di tepi masih membuat setengah blok berada di luar gambar. Karena teks/grup aktif dapat memiliki ukuran berbeda, batas clamp harus memakai ukuran `.subtitle-content` aktual yang sedang tampil; saat grup berubah atau stage resize, pertahankan posisi persentase tersimpan sejauh mungkin dan clamp posisi render untuk dimensi blok terkini. Jika blok lebih besar daripada area video, tidak mungkin sekaligus mempertahankan ukurannya dan seluruhnya berada di dalam area; perilaku overflow tersebut perlu diputuskan (misalnya batasi wrap/tinggi tanpa memperkecil font, atau tampilkan batas posisi terdekat).

Ukuran gaya tidak boleh diubah saat drag. Untuk anchor berbasis konten, `.subtitle-content` dapat ditempatkan absolut pada `left: xPercent%`, `top: yPercent%`, lalu `transform: translate(-50%, -50%)`; outer layer tetap mengikuti videoRect dan tetap membawa font/outline/shadow global. Perlu diperiksa bahwa padding outer layer saat ini (margin kiri/kanan dan bawah) tidak mengubah containing block yang menjadi dasar persentase. Implementasi harus menjaga batas konten horizontal dari `margin_lr` tanpa mempersempit atau menskalakan font secara tidak sengaja.

**Persistensi yang direkomendasikan bertahap:**

- Preview-only: simpan koordinat runtime terpisah dari preset, keyed by preset ID (opsional juga media/proyek jika kelak ada konsep proyek). Default diinisialisasi dari posisi preview yang sekarang.
- Burn: kirim koordinat opsional pada request render/burn. Jangan mengubah file preset bawaan atau memakai `settings.position` yang saat ini tidak digunakan.
- Persisten lintas sesi: belum ada penyimpanan setting aplikasi pada jalur yang diperiksa. Pilih dan dokumentasikan kebutuhan ini sebelum menambahkan localStorage/config file; untuk feasibility tahap awal, tidak perlu menambah persistensi lintas sesi.

## 5. Integrasi ASS

Renderer di `caption-core/src/preset_render.rs` mengeluarkan ukuran output melalui `PlayResX`/`PlayResY`; nilai geometry berasal dari jalur render. `build_style_line()` menentukan alignment dan margin. Event kata utama dan event glow terpisah dibuat di `render_ass_with_preset()`, `render_glow_events()`, dan `render_glow_single()`.

Rancangan posisi ASS yang sederhana:

1. Teruskan posisi user sebagai opsi opsional, misalnya `{ x_percent, y_percent }`, terpisah dari `StylePreset`. Posisi UI tidak perlu mengubah preset font/warna.
2. Konversikan ke koordinat PlayRes yang sama dengan video burn:

   ```text
   assX = round(clamp(xPercent, 0, 100) * geometry.width / 100)
   assY = round(clamp(yPercent, 0, 100) * geometry.height / 100)
   ```

3. Tambahkan override positioning di level Dialogue: `\\an5\\pos(assX,assY)`. `\\an5` menjadikan titik acuan ASS sebagai pusat blok, sesuai anchor preview. Posisi harus ditambahkan ke setiap event utama dan juga event glow layer 0 supaya kedua lapisan tetap menumpuk tepat.
4. Letakkan tag posisi sebelum teks yang sudah dibangun. Jangan mengubah tag emphasis per-kata (`\\c`, `\\u`, `\\fscx`, `\\fscy`) atau timing event.
5. Agar kompatibel dengan file/output lama, ketika posisi tidak diberikan jangan menambahkan tag posisi; biarkan `Style: Alignment` dan margin yang ada bekerja seperti sekarang. Jika koordinat diberikan, tetapkan aturan prioritas: posisi eksplisit menggantikan anchor alignment/margin untuk penempatan event, sedangkan semua nilai style lain tetap berasal dari preset.
6. Pastikan koordinat ASS memakai geometry aktual burn. Jalur burn mem-probe video; jalur export di `src/js/main.js` saat ini mengirim width/height 1080x1920, sehingga file ASS export dan burn tidak otomatis memakai geometry sumber yang sama. Bila posisi perlu sama pada export, perlu memeriksa serta menyelaraskan ukuran yang dikirim untuk export juga.

`\\pos`/`\\an` harus diuji dengan libass pada output render-frame; CSS preview dan libass membungkus teks/font secara berbeda. Persentase anchor dapat sama walaupun bounding box visualnya sedikit berbeda akibat font metrics, line-wrap, outline, dan box.

## 6. Risiko regresi

- **Render ulang menghapus transform DOM:** timeupdate dan pergantian grup membuat ulang konten. Posisi yang tidak disimpan di state akan hilang atau lompat.
- **Pointer event terblokir atau mengganggu video:** parent memiliki `pointer-events:none`; perubahan yang terlalu luas dapat membuat caption tidak bisa di-drag atau mengambil seluruh klik stage.
- **Anchor dan wrapping:** menggunakan pusat blok menggeser seluruh konten; kata yang membesar saat emphasis `scale_active`, outline/glow, dan box menambah visual extent. Clamp perlu mengukur batas luar aktual, bukan hanya ukuran teks tanpa efek.
- **Ukuran blok berubah menurut kata/baris:** sentence grouping, jumlah kata per baris, font, atau teks dapat mengubah lebar/tinggi. Posisi persen sama dapat memerlukan clamp berbeda pada grup tertentu.
- **Ruang kosong contain:** menggunakan koordinat stage alih-alih `videoRect` menyebabkan caption bergeser pada pillarbox/letterbox atau rasio portrait/landscape yang berbeda.
- **CSS override dan inline style:** `renderPreview()` menulis posisi container inline; perubahan stylesheet saja berisiko tidak berpengaruh. Beberapa selector preview juga didefinisikan ulang di bagian akhir `app.css`.
- **Efek yang melebar:** shadow, stroke, dan glow dapat melewati bounding box layout normal dan terpotong oleh `overflow:hidden` walaupun box konten sudah di-clamp.
- **ASS dual-layer:** posisi hanya pada Dialogue utama akan membuat glow layer tertinggal. Posisi harus konsisten di semua jalur event, termasuk emphasis yang menghasilkan event per kata.
- **Default lama berubah:** memulai semua posisi di (50%,50%) akan memindahkan caption dari default bawah. Inisialisasi dari bounding box lama menghindari perubahan visual awal.
- **Resize dan playback:** coordinate state harus tetap stabil ketika ResizeObserver memicu render dan ketika grup berubah; drag tidak boleh mengubah currentMs, play/pause, seek, atau animasi.
- **Tidak muat di video:** jika teks lebih besar dari area yang tersedia, syarat “seluruh blok di dalam gambar” dan “ukuran/gaya tidak berubah” tidak bisa dipenuhi bersamaan.
- **Preview/burn mismatch:** ukuran PlayRes export saat ini ditetapkan dari request frontend untuk 1080x1920, sedangkan burn memakai geometry video. Posisi persentase harus diterjemahkan menggunakan geometry jalur yang benar.
- **Aksesibilitas:** drag mouse saja tidak menyediakan alternatif keyboard/touch; perlu diputuskan apakah aksesibilitas keyboard diperlukan pada implementasi produk.

## 7. Rencana implementasi bertahap

### Tahap 1 — Preview-only, tanpa mengubah ASS

1. Tambahkan state posisi runtime untuk preset terpilih, dengan default hasil ukur posisi caption saat ini (bukan posisi tengah baru).
2. Ubah `renderPreview()` agar `.subtitle-content` diposisikan menurut anchor tengah dan persentase relatif terhadap `videoRect`; pertahankan semua child kata, gaya preset, dimensi font, scale, timing, dan style emphasis.
3. Tambahkan pointer handler tunggal/delegated pada elemen stabil. Drag hanya mulai dari blok caption; gunakan capture, clamp berdasarkan batas blok, dan update posisi tanpa mengubah playback.
4. Pastikan setiap render ulang dan ResizeObserver memakai koordinat runtime yang sama. Burned preview tetap tidak menampilkan overlay.
5. Tambahkan tes murni untuk konversi client coordinates ke persen, clamp berdasarkan ukuran blok, videoRect contain, dan perubahan rasio stage; lakukan browser test untuk pointer capture serta klik/seek.

### Tahap 2 — Penyimpanan posisi UI

1. Putuskan apakah posisi keyed per preset, per media/proyek, atau global. Rekomendasi minimum: keyed per preset dalam state runtime agar tidak mengubah source preset.
2. Uji perpindahan preset, media, resize, reload aplikasi, dan reset posisi. Bila posisi harus bertahan setelah restart, tentukan format dan lifecycle persistensi secara terpisah.
3. Tambahkan UI reset ke posisi default dan keyboard alternative hanya jika diputuskan sebagai kebutuhan; keduanya tidak diperlukan untuk proof of concept drag.

### Tahap 3 — Integrasi render ASS dan burn

1. Tambahkan tipe/payload posisi opsional pada jalur command frontend/Tauri, dan validasi persentase finite serta dalam rentang 0..100.
2. Teruskan opsi tersebut ke renderer ASS tanpa mengubah StylePreset atau perilaku ketika opsi tidak ada.
3. Tambahkan `\\an5\\pos(x,y)` pada semua Dialogue layer utama/glow jika posisi diberikan; gunakan geometry output aktual.
4. Uji golden/default tanpa posisi agar hasil ASS lama identik; uji posisi corner/center, portrait/landscape, glow, box, emphasis per kata, dan render-frame libass.
5. Uji export ASS terpisah dari burn dan tentukan dimensi geometri export yang benar sebelum menyatakan posisi keduanya identik.

## 8. Simulasi teknis

Contoh memakai video portrait 1080x1920 dan stage 800x600 CSS px:

1. `getVideoRect()`:

   ```text
   scale = min(800/1080, 600/1920)
         = min(0.74074, 0.3125)
         = 0.3125
   videoRect.width  = 1080 * 0.3125 = 337.5 px
   videoRect.height = 1920 * 0.3125 = 600 px
   videoRect.left   = (800 - 337.5) / 2 = 231.25 px
   videoRect.top    = (600 - 600) / 2 = 0 px
   ```

2. Skala metrik:

   ```text
   assScale     = min(1080, 1920) / 1080 = 1
   previewScale = 0.3125 * 1 = 0.3125
   ```

   Dengan `preset.margin_v = 450`, margin bawah preview portrait adalah `450 * 0.3125 = 140.625 px`. Misalkan tinggi blok yang sedang tampil 40 px; pusat awal vertikal kira-kira `600 - 140.625 - 40/2 = 439.375 px` dari atas videoRect. Pusat horizontal awal adalah tengah videoRect, yaitu `231.25 + 337.5/2 = 400 px`. Posisi awal terukur menjadi sekitar `(50%, 73.23%)`, bukan `(50%,50%)`.

3. Pengguna menggeser pusat blok ke posisi layar `(450, 400)`:

   ```text
   xPercent = (450 - 231.25) / 337.5 * 100 = 64.8148%
   yPercent = (400 - 0) / 600 * 100 = 66.6667%
   ```

   Jika lebar blok 160 px dan tingginya 40 px, batas anchor:

   ```text
   xMin = 160 / (2 * 337.5) * 100 = 23.7037%
   xMax = 76.2963%
   yMin = 40 / (2 * 600) * 100 = 3.3333%
   yMax = 96.6667%
   ```

   Nilai `(64.8148%, 66.6667%)` berada dalam batas, jadi tidak perlu clamp.

4. Konversi untuk ASS dengan PlayRes 1080x1920:

   ```text
   assX = round(64.8148 * 1080 / 100) = 700
   assY = round(66.6667 * 1920 / 100) = 1280
   ```

   Event ASS berposisi dengan center anchor `\\an5\\pos(700,1280)`. Preview memakai persentase yang sama di videoRect; resize stage mengubah pixel lokasi tetapi koordinat persen tetap. Ukuran font dan efek tidak berubah karena drag hanya mengubah anchor.

5. Jika stage kemudian menjadi 1000x800 dengan video portrait yang sama:

   ```text
   scale = min(1000/1080, 800/1920) = 0.4166667
   videoRect = 450 x 800 px, left = 275 px, top = 0 px
   lokasi anchor x = 275 + 0.648148 * 450 = 566.67 px
   lokasi anchor y = 0 + 0.666667 * 800 = 533.33 px
   ```

   Anchor tetap pada `(64.8148%, 66.6667%)` relatif terhadap gambar, sehingga tetap pada titik yang sama di video meski pixel preview berubah. Perhitungan dimensi blok dan clamp harus diulang untuk ukuran aktual yang baru.

Simulasi ini adalah rumus rancangan, bukan hasil browser runtime atau hasil libass. Tidak ada source/test diubah atau dijalankan untuk kajian ini.

---

# Implementasi Phase 1: Drag Caption Preview

Fase ini mengimplementasikan drag untuk overlay preview frontend saja. Integrasi ASS, backend, export, burn, preset, dan timing tidak diubah.

## File yang berubah

- `src/js/preview.js`
- `src/js/main.js`
- `src/css/app.css`
- `src/tests/preview.test.mjs`
- `report_video_preview.md` (bagian ini)

## Fungsi dan perilaku yang ditambahkan

- `getCaptionPosition(videoRect, blockRect)` menghitung titik tengah blok sebagai persentase terhadap persegi panjang video.
- `clampCaptionPosition(position, videoRect, blockRect, outset)` membatasi anchor agar dimensi blok plus ruang efek tetap berada di batas gambar.
- `moveCaptionPosition(...)` mengonversi delta pointer dalam pixel ke delta persentase, lalu melakukan clamp.
- `getCaptionVisualOutset(metrics)` menyediakan ruang tambahan konservatif untuk outline, shadow, dan glow.
- `applyCaptionPosition(content, position)` menempatkan `.subtitle-content` absolut dengan `translate(-50%, -50%)`; transform kata individual tidak disentuh.
- `renderPreview()` tetap membuat ulang kata dan style seperti sebelumnya, tetapi menerima `captionPosition` opsional dan menerapkannya pada blok. Lebar maksimum konten mempertahankan area teks yang tersedia setelah margin kiri/kanan.
- `renderPreviewText()` menyimpan posisi dalam `captionPositions` yang keyed per preset. Pada render pertama, posisi diukur dari layout lama sehingga letak awal tetap sama; render selanjutnya, pergantian grup, dan resize memakai kembali koordinat tersebut.
- `beginCaptionDrag()`, `moveCaptionDrag()`, dan `endCaptionDrag()` memakai Pointer Events, threshold 4 px, serta pointer capture pada `#preview-stage`. Drag hanya dimulai dari `.subtitle-content`; drag tidak mengubah waktu atau kata.
- CSS mengaktifkan hit-testing hanya pada `.subtitle-content.is-draggable`, memakai cursor `grab/grabbing`, dan mencegah seleksi teks selama drag.
- `#preview-subtitle` tetap disembunyikan oleh jalur burned preview yang sudah ada.
- Tes unit ditambahkan untuk menghitung anchor, clamp ukuran blok/efek, serta mengubah delta drag dari pixel ke persen.

## Hasil verifikasi yang dijalankan

- `git diff --check` — exit code 0. Git menampilkan peringatan normal tentang normalisasi CRLF pada beberapa file kerja yang sudah berubah.
- `node --test src/tests/preview.test.mjs` — tidak dapat berjalan: PowerShell melaporkan `node` tidak dikenali sebagai cmdlet/program. Unit test baru ditulis tetapi tidak dijalankan lewat Node.
- Browser smoke test pada halaman lokal memakai data transkripsi simulasi dan file video lokal:
  - Drag caption mengubah posisi dari sekitar `50.0035%, 73.993%` ke `58.9721%, 77.7767%`; slider waktu tetap `1` dan transform/color kata aktif `scale(1.18)`/warna highlight tidak berubah.
  - Drag ekstrem ke kiri, kanan, atas, dan bawah menghasilkan `inside: true` untuk semua pemeriksaan bounding box blok terhadap `#preview-subtitle`.
  - Seek ke teks grup berbeda (`LAGI CARI` dan `KAUTION RINGAN`) mempertahankan posisi yang sama pada percobaan tersebut.
  - ResizeObserver diuji dengan mengubah ukuran stage sementara melalui style browser dari 223x398 menjadi 298x248 px. Persentase posisi tetap `44.6243%, 94.4399%` dan blok terukur tetap berada di dalam videoRect. Perubahan viewport biasa saja tidak mengubah ukuran stage di layout browser ini; karena itu pengujian resize dilakukan dengan mengubah ukuran stage langsung.
  - Snapshot gaya sebelum/sesudah drag pada preset glow (Baris Neon Mint), box (Baris Kotak Kuning), serta Tekan Besar tidak menunjukkan perubahan style selama drag. Pada Tekan Besar, kata aktif dengan scale 1.18 dan warna highlight tetap memiliki transform/warna yang sama setelah drag.
  - Tombol play dan seek tetap enabled; seek interaktif digunakan dalam uji. Pemutaran aktual tidak terverifikasi: percobaan `play()` pada browser latar belakang menghasilkan `AbortError` karena media dihentikan oleh kebijakan browser.
- Hasil tersebut adalah smoke test browser pada halaman simulasi, bukan uji Tauri WebView atau pengujian semua file/preset.

## Belum diuji dan potensi masalah tersisa

- Burned preview tidak dapat dijalankan di browser mock. Jalur source yang ada menyembunyikan `#preview-subtitle` saat `burnedPreviewActive`, tetapi keadaan burn nyata belum diuji di Tauri.
- Tidak menjalankan rangkaian manual penuh di WebView, termasuk play/pause aktual, perpindahan preset berulang, drag saat pemutaran berlangsung, serta resize jendela aplikasi yang nyata.
- Posisi hanya disimpan dalam Map runtime per preset. Posisi tidak bertahan setelah reload atau aplikasi ditutup; tidak ada penyimpanan disk/localStorage yang ditambahkan.
- `getBoundingClientRect()` mengukur box layout dan transform kata, tetapi bukan extent efek visual secara persis. `getCaptionVisualOutset()` menambah margin konservatif dari outline/shadow/glow; hasil pixel sempurna pada semua browser/font belum dibuktikan.
- Jika tinggi blok beserta efeknya lebih besar dari area video, tidak mungkin memenuhi syarat tetap utuh dan seluruhnya berada di dalam area secara bersamaan. Data yang diuji tidak mencapai kondisi ini.
- Posisi dapat mengalami clamp ketika suatu grup tertentu lebih besar atau batas video berubah. Percobaan grup yang diuji mempertahankan koordinat, tetapi semua variasi teks tidak diuji.
- Unit test tidak dijalankan karena Node tidak tersedia. Tidak ada integrasi ASS/burn yang dilakukan.

---

# Phase 2: Integrasi Posisi Caption Preview ke ASS Render

## Status

**PARTIAL.** Posisi runtime dari preview kini diteruskan ke renderer ASS, diterapkan pada Dialogue utama dan glow, serta lolos unit test dan uji burn FFmpeg dengan video sintetis. Browser preview dan frame hasil libass dengan preset/kata yang sama juga diperiksa pada posisi tengah. Namun perbandingan itu memakai ASS CLI yang diberi tag posisi secara lokal, bukan alur burn dari Tauri WebView secara langsung; pengujian dengan video pengguna dan Tauri WebView belum dilakukan.

## Jalur render sebelum perubahan

- `src/js/main.js`: `captionPositions` adalah `Map` runtime dengan ID preset sebagai key. `renderPreviewText()` meneruskan koordinat `{x, y}` ke `renderPreview()`. Sebelum tahap ini, posisi tidak ikut dalam permintaan ekspor atau burn.
- `src/js/main.js`: `exportSubtitles()` memanggil `exportSubtitleFile()`; `createVideo()` memanggil `burnVideo()`.
- `src/js/api.js`: `exportSubtitleFile()` mengirim `req` ke command `write_export_file`; `burnVideo()` mengirim `req` ke command `burn_video`.
- `src-tauri/src/commands.rs`: `RenderRequest` dipakai untuk render dan ekspor ASS. `BurnRequest` diterima oleh `burn_video_impl()`, yang mem-probe video lalu membentuk `RenderRequest`.
- `render_ass_impl()` membentuk grup dan `PresetOptions`, lalu memanggil `caption_core::render_ass_with_preset()`. `render_export_impl()` memakai renderer yang sama untuk format ASS.
- `caption-core/src/preset_render.rs`: `render_ass_with_preset()` menulis Dialogue utama pada layer 1. `render_glow_events()` dan `render_glow_single()` menulis efek glow pada layer 0. Sebelum tahap ini, lokasi diatur oleh alignment dan margin Style, tanpa tag posisi Dialogue.

## File yang berubah pada Phase 2

- `src/js/main.js`: `exportSubtitles()` dan `createVideo()` menyertakan posisi preset aktif jika tersedia di `captionPositions`.
- `src/js/api.js`: `exportSubtitleFile()` dan `burnVideo()` memetakan `{x, y}` ke `caption_position: {x_percent, y_percent}`.
- `src-tauri/src/commands.rs`: menambahkan `CaptionPositionWire`, field opsional `caption_position` di `RenderRequest` dan `BurnRequest`, validasi, serta propagasi dari `burn_video_impl()`/jalur ekspor ke renderer.
- `caption-core/src/lib.rs`: mengekspor tipe `CaptionPosition`.
- `caption-core/src/preset_render.rs`: menghitung posisi ASS dan menambah override posisi pada Dialogue utama maupun glow.
- `src-tauri/src/commands_tests.rs`: tes kompatibilitas payload lama, validasi koordinat, kedua layer, serta fixture burn yang memakai posisi custom.
- `caption-core/tests/ass_preset_tests.rs`: tes koordinat posisi terhadap PlayRes dan pemeriksaan layer glow/utama.
- `report_video_preview.md`: bagian laporan Phase 2 ini.

## Konversi koordinat dan kompatibilitas

- Frontend mengirim persentase terhadap area gambar `videoRect`, bukan koordinat panel atau stage.
- Rust menerima nilai `x_percent` dan `y_percent`; kedua nilai harus finite dan berada dalam rentang 0–100. Nilai di luar rentang menghasilkan error, tidak diam-diam di-clamp.
- Renderer mengonversi `x = round(x_percent * PlayResX / 100)` dan `y = round(y_percent * PlayResY / 100)`, lalu menambahkan `{\an5\pos(x,y)}` pada setiap event. Anchor 5 membuat koordinat menjadi titik tengah blok, selaras dengan anchor drag preview.
- Prefix posisi yang sama diterapkan pada layer utama dan Dialogue glow. Geometri output dari burn berasal dari hasil probe video.
- Jika `caption_position` tidak ada, string override kosong sehingga Style alignment dan margin yang lama tetap menentukan letak. Field opsional juga memakai default saat payload lama tidak menyertakan key itu.
- Ekspor ASS ikut menerima koordinat. Dimensi permintaan ekspor tetap menggunakan nilai 1080×1920 yang sudah dipakai sebelumnya di `main.js`; dimensi ekspor untuk rasio video lain belum diuji. Format SRT/VTT/TXT/JSON tidak memakai posisi ASS.
- Tidak ada perubahan pada preset JSON, timing, playback, animasi, ukuran/gaya caption, ataupun penyimpanan posisi permanen. Posisi masih berupa `Map` runtime per preset.

## Hasil validasi

- `cargo fmt --all -- --check` — exit code 0.
- `cargo clippy --workspace --all-targets --no-default-features -- -D warnings` — exit code 0.
- `cargo test --workspace --no-default-features` — exit code 0; hasil mencakup 21 tes `ass_preset_tests`, 8 tes `group_tests`, dan 36 tes command lulus. Satu tes burn bawaan ditandai ignored secara default.
- `cargo test -p caption-tauri --lib burn_video_sidecar_integration -- --ignored --nocapture` dengan `CAPTION_TEST_FFMPEG` dan `CAPTION_TEST_FFPROBE` menunjuk executable di `target/debug` — `1 passed; 0 failed`. Tes membuat video `testsrc` 1080×1920 selama 2 detik, merender ASS dari `render_ass_impl()` dengan `kata-anton-kuning` dan posisi 50%,50%, lalu membakar MP4 melalui FFmpeg. Pemeriksaan output memastikan ukuran 1080×1920 dan durasi >=1,9 detik.
- `node --test src/tests/preview.test.mjs` — tidak berjalan: PowerShell menyatakan `node` tidak dikenali. Sebagai pemeriksaan interaksi terpisah, halaman lokal dibuka di browser dengan video test 1080×1920 dan mock words. Pada preset Kata Anton Kuning, drag ke `(25%,25%)`, `(75%,25%)`, `(75%,75%)`, `(25%,75%)`, lalu `(50%,50%)` terukur tepat pada persentase tersebut; `currentTime` tetap 0,2 detik dan teks aktif tetap `LAGI`.
- Untuk membandingkan titik tengah preview dan libass, ASS dari CLI dengan preset/kata mock yang sama diberi tag `{\an5\pos(540,960)}` pada PlayRes 1080×1920 dan dibakar ke video test yang sama. FFmpeg/libass selesai dengan exit code 0. Frame hasil render diperiksa: caption berada di titik tengah frame, sesuai ukuran anchor preview yang terukur 50%,50%. Tag posisi pada percobaan manual ini disisipkan ke ASS CLI secara lokal; CLI tidak menerima posisi custom. Karena itu percobaan frame bukan bukti langsung bahwa UI Tauri memanggil burn dengan payload drag tersebut.
- Log FFmpeg menyatakan font `Anton` dipetakan ke `ArialMT` pada mesin ini. Maka perbandingan ini menguji titik posisi, bukan kesamaan bentuk/ukuran font antara browser dan libass.
- File video/frame/ASS pembanding dibuat di `%TEMP%` dan dibersihkan setelah pemeriksaan.

## Belum diverifikasi / risiko tersisa

- Belum menjalankan aplikasi Tauri WebView dan menekan tombol burn melalui UI untuk membandingkan posisi drag terakhir dengan output dari video pengguna. Uji burn langsung menguji renderer dan pipeline FFmpeg dengan fixture sintetis; browser preview terpisah menguji interaksi drag.
- Tidak ada sumber video pengguna di fixture repo yang dipakai. Sinkronisasi audio/video nyata dan perilaku rotasi pada file pengguna belum diuji.
- Posisi custom pada ekspor ASS menggunakan PlayRes 1080×1920 yang telah menjadi default jalur ekspor; konsistensi ekspor untuk video landscape/persegi belum dibuktikan.
- Burned preview tetap mengikuti jalur lama yang menyembunyikan overlay CSS; perubahan ini tidak menyentuhnya. Kondisi UI Tauri setelah burn nyata belum diuji.
- Node tidak tersedia, sehingga unit test JavaScript tidak dijalankan. Tidak ada perubahan pada tes JavaScript.
- Posisi belum disimpan permanen; menutup atau memuat ulang aplikasi masih dapat menghilangkan Map runtime.

---

# Visual Parity Audit: Preview CSS vs ASS/libass

## Ruang lingkup dan tingkat kepastian

Audit ini membaca source frontend, renderer ASS, jalur font burn, dan preset yang ada. Tidak ada source, preset, playback, atau pipeline render yang diubah. Tidak dilakukan render ulang atau perbandingan screenshot untuk semua preset; tabel berikut membedakan fakta implementasi dari dugaan penyebab tampilan yang perlu dikonfirmasi dengan frame pasangan.

Catatan penting dari audit Phase 2: uji FFmpeg manual saat itu tidak memasang folder font aplikasi dan mencatat `Anton` fallback ke `ArialMT`. Jalur burn Tauri berbeda: `setup_work_dir()` menulis font bundled dan `build_burn_args()` menggunakan `fontsdir=fonts`. Jadi fallback pada uji manual itu **bukan bukti** bahwa burn aplikasi juga memakai font fallback.

## Tabel perbandingan visual

| Prioritas / aspek | Preview CSS | ASS/libass | Penyebab perbedaan dan apakah dapat disamakan di ASS | Rekomendasi aman dan file/fungsi |
|---|---|---|---|---|
| **P0 — Font family/fallback** | `app.css` mendaftarkan file TTF lokal lewat `@font-face`. `loadPreviewFont()` meminta browser memuat family preset; `renderPreview()` memakai stack `family, Arial, sans-serif`. Log “Preview font checks” hanya memeriksa ketersediaan family, bukan kesamaan bentuk glyph. | `build_style_line()` menulis nama `preset.font.family`. `video.rs` membundel Poppins, Anton, dan Bangers; `setup_work_dir()` mengekstraknya dan filter FFmpeg memakai `fontsdir=fonts`. | Nama/family dan file yang dipakai dirancang sepadan. Jika font tidak ditemukan atau nama internal TTF tidak cocok, fallback font engine dapat mengubah lebar, tinggi, dan bentuk teks. ASS dapat memilih family/font lain, tetapi tidak dapat mengoreksi metrik CSS yang berubah tanpa memastikan font yang sama lebih dulu. | **Verifikasi dahulu** log font selection dari burn Tauri dan browser dengan preset yang sama. Jangan mengganti font preset sebelum bukti fallback didapat. Jika mismatch terbukti, perbaiki pemetaan/penyediaan font di `caption-core/src/video.rs` atau `src/css/app.css`; renderer terkait `build_style_line()` dan `loadPreviewFont()`. Risiko tinggi karena perubahan font mengubah wrapping dan posisi visual. |
| **P1 — Font size dan weight** | `computePreviewMetrics()` mengalikan ukuran preset dengan `previewScale`; `renderPreview()` memasang nilai CSS px dan `fontWeight` 700/400. Preset `Poppins ExtraBold` memakai file ExtraBold yang didaftarkan pada weight 400; preset Poppins memakai file Bold pada weight 700. | `build_style_line()` mengalikan ukuran dengan geometri video lalu membulatkannya ke integer. `font.bold` menjadi nilai Bold ASS `-1` atau `0`. | Skala nominalnya sepadan, tetapi CSS px dan rasterisasi libass/font backend tidak identik. Pembulatan ASS dan metrik/font-weight aktual dapat mengubah tinggi dan lebar glyph. Sumber tidak menunjukkan satu faktor skala global yang jelas salah. | Pertama pastikan font sama; lalu bandingkan ukuran bounding box untuk geometri/PlayRes yang sama. Jika terbukti perlu, koreksi nilai ASS pada `caption-core/src/preset_render.rs::build_style_line()` saja. Risiko sedang karena ukuran memengaruhi wrapping dan margin efektif. |
| **P1 — Highlight kata aktif (`fill`)** | Untuk `emphasis === "fill"`, `renderPreview()` mengisi seluruh kata aktif dengan `colors.highlight`; kata lain tetap memakai warna teks. Perubahan terjadi saat kata aktif berdasarkan waktu. | `build_fill_text()` menghasilkan tag karaoke `\k`/`\kf`. `build_style_line()` mengatur `SecondaryColour` sama dengan `PrimaryColour`, dan `build_fill_text()` tidak menulis warna highlight. | Ini perbedaan source yang nyata: preset `baris-isi-kuning` meminta highlight kuning, tetapi jalur ASS `fill` tidak mengatur warna sekundernya ke kuning. Selain itu `\kf` memberi pengisian karaoke progresif, sedangkan preview menyorot satu kata penuh sekaligus. Mengubah `SecondaryColour` dapat menambahkan warna, tetapi tetap mempertahankan efek progresif yang tidak sama dengan preview. | Jika solusi disetujui, pertahankan preview dan ubah **ASS saja**: buat event per rentang waktu kata dengan kata aktif diberi `\c` highlight, memakai batas waktu kata yang sama. Lokasi: `caption-core/src/preset_render.rs::render_ass_with_preset()`, `build_fill_text()`/`build_group_text()`. Risiko sedang: khusus preset `fill`, tetapi event ASS bertambah dan harus diuji pada batas waktu kata. |
| **P1 — Glow dan shadow** | Tanpa box, preview menggambar stroke CSS lalu satu `text-shadow`: glow memakai `0 0 radius color`; shadow biasa memakai offset X/Y sama dengan blur 0. Pada cabang box, preview mematikan stroke dan shadow. | Shadow biasa ditulis sebagai `Shadow` dan `BackColour` pada Style. Glow membuat Dialogue tambahan di layer 0 dengan `\bord`, `\blur`, `\c`, `\3c`, sedangkan Dialogue utama tetap dirender pada layer 1. Nilai shadow Style tetap berasal dari preset. | `text-shadow` browser dan blur/outline event libass adalah model rasterisasi berbeda. Glow ASS memiliki lapisan teks tambahan; preview hanya satu shadow blur. Untuk kombinasi box+shadow, cabang CSS mematikan shadow tetapi ASS Style masih membawa nilai shadow; preset box yang ada saat ini memakai shadow 0 sehingga potensi itu belum menjadi bukti mismatch preset saat ini. | Render pasangan dari jalur Tauri memakai font bundled sebelum menala nilai. Bila halo yang berbeda terbukti, sesuaikan radius/tag/layer ASS saja pada `render_glow_events()`, `render_glow_single()`, atau `build_style_line()`. Risiko sedang–tinggi: glow juga bergantung pada outline, warna, ukuran glyph, dan layer order. Jangan mengubah preview. |
| **P2 — Ketebalan outline/stroke** | Preview memakai `-webkit-text-stroke` dengan lebar CSS `2 * metrics.outline` dan `paintOrder = "stroke fill"` ketika tanpa box. | ASS memakai `BorderStyle=1` dan `Outline = preset.outline * geometry.scale()`, dibulatkan. Pada preset box, renderer memakai `BorderStyle=3`, warna box, serta `Outline` dari padding box, bukan outline teks. | Secara geometri, lebar CSS dua kali nilai metrik menghasilkan stroke di kedua sisi kontur; ASS memakai nilai Outline-nya sendiri. Source tidak membuktikan bahwa faktor 2 tersebut dipilih khusus untuk menyamakan radius ASS. Antialiasing, hinting, dan definisi tepi glyph juga berbeda. | Bandingkan satu preset tanpa box dan satu preset box pada resolusi sama; jika perlu tune ASS per perilaku renderer di `build_style_line()`. Risiko tinggi bila mengubah formula global karena outline semua preset dan glow ikut terpengaruh. |
| **P2 — Warna teks/highlight selain `fill`** | Warna teks, fill span, dan warna highlight dipasang sebagai CSS color/text-fill. | Warna utama dikonversi ke urutan ASS BGR dengan alpha terbalik. `color_active`, `underline_active`, dan `scale_active` memakai override `\c` untuk highlight. | Jalur pemetaan utama tampak tersedia di kedua sisi; perbedaan yang mungkin berasal dari alpha blending, antialiasing, atau fallback font. `fill` tercatat terpisah di atas karena kasus itu tidak mengirim warna highlight. | Jangan ubah warna preset sebelum memeriksa nilai Style/Dialogue ASS dan hasil dengan font yang sama. Jika nilai ASS keliru, periksa `ass_color_from_hex()`/`build_group_text()` di `preset_render.rs`. Risiko rendah bila satu warna diperbaiki, tinggi bila konversi umum diubah. |
| **P2 — Background/box caption** | Untuk preset dengan box, `renderPreview()` memberi `backgroundColor`, padding dari `box.padding * previewScale`, dan radius 0 pada `.subtitle-content`. | `build_style_line()` memilih `BorderStyle=3`, warna box sebagai outline color, dan nilai `Outline` dari padding box yang diskalakan. | Kedua jalur memakai ukuran padding dan sudut tanpa radius, tetapi CSS menggambar latar pada box DOM sementara libass menggambar border-box per event. Metrik teks, line wrapping, pembulatan, dan perilaku box untuk beberapa baris bisa menghasilkan ukuran rectangle berbeda. | Bandingkan preset box satu baris dan multi-baris. Jika hanya ukuran rectangle berbeda, tune pemetaan padding/box pada `build_style_line()`; jangan mengubah struktur preset. Risiko sedang, terbatas pada preset box tetapi berpengaruh ke area visual besar. |
| **P2 — Transform/pembesaran kata aktif** | `scale_active` memakai `transform: scale(...)` pada span `inline-block`; transform visual tidak mengubah ukuran layout dasar kata. | `build_group_text()` memberi kata aktif tag `\fscx` dan `\fscy` di event per kata. Perubahan ukuran glyph terjadi di layout/rasterizer ASS. | CSS transform dan skala glyph ASS dapat menghasilkan bentuk/posisi baseline serta lebar visual berbeda; perbedaan layout paling tampak dekat batas baris atau ketika kata besar. Nilai persentase sama belum menjamin hasil raster identik. | Jika terkonfirmasi, tune ASS scale tag/anchor tanpa mengubah batas waktu event. Lokasi `build_group_text()` dan pembentukan event di `render_ass_with_preset()`. Risiko sedang karena scale dapat mengubah lebar visual atau overlap. |
| **P2 — Line spacing, wrapping, alignment** | `.subtitle-layer` memberi `line-height: 1.3` dan `text-wrap: balance`. Anak kata dibuat `inline-block` dengan spasi antar-span. Aturan `.subtitle-content` lama menyebut flex-wrap/gap, tetapi deklarasi akhir `display: inline-block !important` mengalahkannya; JS juga menetapkan max-width inline dengan `!important`. `letterSpacing` CSS tetap nilai pecahan setelah skala. Posisi drag memakai titik tengah blok. | ASS memakai `WrapStyle: 0`, `MarginL/R/V`, preset `Alignment`, dan metrik libass; line height tidak ditentukan eksplisit oleh renderer. Nilai `Spacing` ASS dibulatkan ke integer. Posisi custom memakai `\an5\pos(...)`, sehingga anchor tengah selaras dengan anchor drag, tetapi tidak memaksa pemenggalan baris yang sama. | Algoritma wrapping, ukuran glyph, line-height 1.3 CSS versus metrik baris libass, aturan balance, dan pembulatan letter spacing dapat membuat jumlah baris/lebar berbeda walaupun koordinat anchor 50% sama. Ini salah satu penyebab posisi terlihat berbeda secara visual tanpa koordinat drag yang salah. | Prioritaskan mengukur jumlah baris dan bounding box pada video/preset yang sama. Bila perlu, ubah ASS saja dengan pemenggalan eksplisit `\N` yang dihitung konsisten; jangan mengubah drag atau koordinat. Lokasi: `preview.js::renderPreview()`, aturan `.subtitle-layer`/`.subtitle-content` di `app.css`, dan `render_ass_with_preset()`/`build_style_line()`. Risiko tinggi untuk teks panjang dan banyak rasio video. |
| **P3 — Animasi/transisi antar kata** | Tidak ada transition khusus pada `.karaoke-word`; preview mengganti style kata aktif saat render waktu. `scale_active` berubah seketika ketika kata menjadi aktif. | Efek aktif biasa memakai Dialogue dengan batas waktu kata; `fill` memakai `\kf`, yang memiliki progres fill berdasarkan durasi karaoke. Renderer tidak menulis transform transition untuk pembesaran kata. | Selain `fill`, kedua sisi sama-sama berbasiskan batas waktu tetapi perubahan style dan penggambaran berlangsung di engine berbeda. `\kf` menambahkan animasi progresif yang tidak ada pada preview. | Jangan mengubah timing. Jika preview tetap menjadi acuan, samakan semantik ASS pada event per kata; pertahankan `start_ms/end_ms` yang sama. Lokasi event generation dan `build_fill_text()` di `preset_render.rs`. Risiko sedang, terutama pada transisi aktif/karaoke dan batas kata. |

## Rekomendasi berdasarkan prioritas

1. **Pastikan perbandingan font benar terlebih dahulu.** Jalankan burn aplikasi dengan preset Anton/Poppins dan periksa bahwa `fontsdir=fonts` benar-benar membuat libass memilih TTF bundled. Bandingkan dengan font browser yang terlapor; fallback pada tes CLI tanpa folder font tidak cukup untuk menyimpulkan bug Tauri.
2. **Perbaiki `fill` hanya di ASS.** Ini perbedaan logika yang dapat ditunjuk langsung dari source dan paling mungkin membuat preset highlight tampak berbeda. Pertahankan waktu kata; jangan menurunkan tampilan preview atau mengganti preset.
3. **Lakukan perbandingan frame terkontrol untuk outline, glow/shadow, box, dan scale.** Gunakan video, ukuran, kata, preset, waktu, posisi, serta font yang sama. Ubah satu variabel ASS tiap iterasi dan simpan golden/frame pembanding.
4. **Tackle wrapping setelah font dan ukuran glyph tervalidasi.** Perubahan wrapping dapat mengubah tinggi blok walaupun anchor tengah sudah benar; hindari mengubah drag/koordinat sebagai kompensasi untuk beda jumlah baris.
5. **Jangan mengubah timing, playback, CSS preview, preset JSON secara luas, atau posisi drag pada tahap implementasi parity.** Implementasi berikutnya sebaiknya terbatas pada ASS yang terbukti berbeda dan mendapat persetujuan terlebih dahulu.

## File yang kemungkinan perlu disentuh jika solusi disetujui

- `caption-core/src/preset_render.rs`: `build_style_line()`, `build_group_text()`, `build_fill_text()`, `render_glow_events()`, `render_glow_single()`, dan pemilihan event di `render_ass_with_preset()`.
- `caption-core/src/video.rs`: `BUNDLED_FONTS`, `setup_work_dir()`, dan `build_burn_args()` hanya jika verifikasi menunjukkan font tidak tersedia pada burn aplikasi.
- `src/js/preview.js` dan `src/css/app.css`: dipakai sebagai referensi metrik/face/wrapping; rekomendasi audit ini tidak mengusulkan perubahan preview karena preview harus dipertahankan.
- `caption-core/presets/*.json`: tidak perlu restrukturisasi; bila pengukuran membuktikan satu preset memiliki nilai visual yang salah, ubah hanya nilai preset terkait dan uji dampaknya.

## Risiko regresi dan hal yang belum dibuktikan

- Perubahan font/ukuran/outline dapat mengubah lebar, tinggi, wrapping, clamp posisi, dan seluruh preset yang berbagi renderer.
- Perubahan `fill` ke event per kata menambah jumlah Dialogue; perlu memastikan tidak ada celah/overlap di batas waktu kata dan glow tetap berada di bawah layer utama.
- Pemenggalan baris eksplisit dapat meningkatkan kemiripan untuk satu viewport tetapi menurunkannya pada resolusi/rasio atau font berbeda.
- Belum dibuat matriks render per preset dan belum dilakukan pixel-diff/screenshot pair dalam audit ini. Klaim perbedaan di tabel berasal dari source; dampak visual spesifik selain kasus `fill` adalah hipotesis yang perlu diuji sebelum implementasi.
- Tidak ada perintah build/test dijalankan karena tahap ini hanya audit source dan penambahan laporan.

---

# Validasi font dan metrik: Preview vs libass

## Metode dan kepastian

Pemeriksaan ini tidak mengubah source, CSS, preset, renderer, atau pipeline burn. Browser dijalankan dengan fixture video 1080x1920 dan kata yang sama dengan frame CLI. Font browser yang benar-benar dipakai diperiksa melalui Chromium DevTools Protocol (`CSS.getPlatformFontsForNode`), bukan hanya dari `font-family` CSS. Frame libass dibuat oleh `caption-cli --render-frame`; jalur ini memanggil `setup_work_dir()` dan memasang `fontsdir=fonts`, sama seperti konfigurasi FFmpeg pada burn Tauri.

Verifikasi frame CLI mereproduksi cara penyediaan font yang dipakai burn Tauri, tetapi saya tidak menjalankan tombol burn di Tauri WebView pada pemeriksaan ini. Jadi bukti pemilihan font libass berlaku untuk render CLI dengan jalur font yang sama, bukan pengamatan langsung atas proses burn dari UI.

## Font yang benar-benar dipilih

Browser memilih file font berikut pada semua 12 preset yang diuji; log pemilihan font libass pada frame CLI menunjukkan PostScript name yang sama:

| Family pada preset/ASS | File font yang dipakai | Browser / PostScript name libass | SHA-256 preview = bundled |
|---|---|---|---|
| Poppins ExtraBold | `Poppins-ExtraBold.ttf` | `Poppins-ExtraBold` | `F2AB17C1A63A0ECC12C2461848FC8A469395E3CD2D641803E889C643D9F958E1` |
| Poppins | `Poppins-Bold.ttf` | `Poppins-Bold` | `983676516167748B74DE6F4771FB384C664FD913ACB8B471122ECACF5DA5EA6C` |
| Anton | `Anton-Regular.ttf` | `Anton-Regular` | `A4BA3A92350EBB031DA0CB47630AC49EB265082CA1BC0450442F4A83AB947CAB` |
| Bangers | `Bangers-Regular.ttf` | `Bangers-Regular` | `4160A7311DE9342674CCE9160CDE9FCBB30F48190397D86FF1B70B455AF65824` |

Nama family internal font sesuai dengan family yang diminta preset/ASS. SHA-256 masing-masing file di `src/assets/fonts/` cocok byte-per-byte dengan file di `caption-core/assets/fonts/`. Dengan demikian tidak ditemukan fallback font atau file font berbeda pada browser dan frame libass yang diperiksa. CSS mendaftarkan file Poppins Bold pada weight 700, Poppins ExtraBold pada 400, serta Anton dan Bangers pada 400; nilai ASS `Bold` juga cocok: `-1` untuk Poppins Bold dan `0` untuk tiga file regular/ExtraBold. Untuk ExtraBold, bobot visual berasal dari file ExtraBold itu sendiri, bukan synthetic bold.

## Ukuran dan weight nominal

Pada video 1080x1920, ukuran `Fontsize` ASS sama dengan ukuran font preset karena skala geometri ASS bernilai 1. CSS memakai ukuran yang sama dikalikan skala preview; viewport yang diuji memiliki `videoRect` 535.75x954 px, sehingga ukuran computed di tabel lebih kecil secara proporsional. Nilai weight browser diambil dari computed style; nilai Bold ASS berasal dari `build_style_line()`.

| Preset | Font family | Ukuran preset / ASS | Ukuran computed CSS (px) | CSS weight | ASS Bold |
|---|---|---:|---:|---:|---:|
| `kata-tebal-putih` | Poppins ExtraBold | 96 | 47.6444 | 400 | 0 |
| `kata-kotak-merah` | Poppins ExtraBold | 88 | 43.6741 | 400 | 0 |
| `kata-anton-kuning` | Anton | 104 | 51.6148 | 400 | 0 |
| `kata-bangers` | Bangers | 100 | 49.6296 | 400 | 0 |
| `baris-kotak-hitam` | Poppins | 64 | 31.7630 | 700 | -1 |
| `baris-kotak-kuning` | Poppins | 64 | 31.7630 | 700 | -1 |
| `baris-neon-mint` | Poppins ExtraBold | 72 | 35.7333 | 400 | 0 |
| `baris-isi-kuning` | Poppins | 68 | 33.7481 | 700 | -1 |
| `tekan-warna-kuning` | Poppins ExtraBold | 78 | 38.7111 | 400 | 0 |
| `tekan-garis-bawah` | Anton | 92 | 45.6593 | 400 | 0 |
| `tekan-besar` | Poppins ExtraBold | 76 | 37.7185 | 400 | 0 |
| `tekan-ungu-kotak-gelap` | Poppins | 72 | 35.7333 | 700 | -1 |

## Bounding box hasil render

Pada sampel terkontrol `kata-anton-kuning`, teks `LAGI` sama di kedua renderer. Ukuran video dan teks sama, serta font file sama. Dengan mask luma yang sama untuk mengukur isi kuning (ambang 150), browser menghasilkan bbox 81x45 px pada preview. Dinormalisasi terhadap `videoRect`, hasilnya sekitar **163x91 px** pada resolusi 1080x1920. Frame libass menghasilkan bbox isi **95x51 px**. Jadi isi glyph yang tampak pada contoh ini berbeda ukuran meskipun family, file font, ukuran nominal 104, dan weight regular cocok. Mask ini bergantung pada ambang antialiasing, tetapi selisihnya cukup besar untuk tidak dijelaskan oleh pembulatan satu piksel.

Sebagai pemeriksaan luas, bbox visual seluruh caption juga diukur untuk 12 preset. Browser dinormalisasi dari screenshot preview ke 1080x1920; kolom ASS berasal dari frame CLI 1080x1920. Ukuran ini mencakup outline, shadow/glow, box, dan wrapping sehingga bukan pengukuran glyph murni:

| Preset | Browser ternormalisasi (px) | libass (px) |
|---|---:|---:|
| `kata-tebal-putih` | 232x87 | 140x59 |
| `kata-kotak-merah` | 248x159 | 160x132 |
| `kata-anton-kuning` | 181x109 | 116x74 |
| `kata-bangers` | 173x93 | 102x57 |
| `baris-kotak-hitam` | 540x207 | 344x104 |
| `baris-kotak-kuning` | 540x207 | 348x104 |
| `baris-neon-mint` | 387x161 | 486x84 |
| `baris-isi-kuning` | 292x159 | 335x58 |
| `tekan-warna-kuning` | 405x70 | 242x49 |
| `tekan-garis-bawah` | 339x107 | 209x71 |
| `tekan-besar` | 403x83 | 266x54 |
| `tekan-ungu-kotak-gelap` | 540x225 | 378x108 |

Perbedaan bbox visual pada tabel terukur, tetapi bukan bukti bahwa ukuran font adalah satu-satunya penyebab. Khusus preset baris/box, jumlah baris dan area latar juga memengaruhi bbox; browser dan ASS menggunakan aturan wrapping/layout yang berbeda. Pengukuran glyph `LAGI` di atas mengurangi faktor tersebut dan tetap menemukan selisih ukuran.

## Kesimpulan dan rekomendasi

- **Family/file:** sama pada browser dan libass frame CLI; tidak ada bukti fallback pada fixture ini. Jalur burn Tauri mengekstrak `BUNDLED_FONTS` melalui `setup_work_dir()`, lalu `build_burn_args()` memasang `fontsdir=fonts`.
- **Ukuran dan weight nominal:** cocok untuk semua preset yang diperiksa. Ukuran CSS sudah dinormalisasi berdasarkan ukuran `videoRect`; weight file Poppins/regular juga sesuai dengan flag ASS.
- **Metrik/render aktual:** bbox isi `LAGI` Anton berbeda sekitar 163x91 px di preview versus 95x51 px di libass. Jadi kesamaan file dan nilai nominal belum menghasilkan ukuran raster yang sama.
- **Penyebab yang terbukti:** bukan perbedaan file font atau fallback pada jalur CLI yang diuji. Yang terbukti adalah perbedaan hasil metrik/raster antara browser CSS dan libass.
- **Penyebab yang belum terisolasi:** belum dapat dipastikan apakah selisih terutama berasal dari interpretasi ukuran font ASS/libass, metrik ascent/cap-height, atau cara kedua engine merasterisasi glyph. Satu contoh belum cukup untuk menentukan faktor koreksi yang aman bagi semua family.
- **Langkah terkecil yang disarankan:** instrumentasikan/log pilihan font dan bandingkan sampel glyph yang sama pada burn Tauri nyata; lalu ulangi pengukuran glyph-only untuk keempat family. Jika ukuran ASS memang konsisten lebih kecil, evaluasi koreksi ukuran ASS secara terukur pada `caption-core/src/preset_render.rs::build_style_line()` saja. Jangan mengubah preview atau menerapkan faktor global berdasarkan satu preset.

## File dan fungsi terkait

- `src/css/app.css`: deklarasi `@font-face` untuk file font browser.
- `src/js/preview.js`: `loadPreviewFont()`, `computePreviewMetrics()`, dan `renderPreview()` memilih/mengukur font serta mengatur CSS.
- `caption-core/src/preset_render.rs`: `build_style_line()` menulis family, `Fontsize`, dan Bold ke ASS.
- `caption-core/src/video.rs`: `BUNDLED_FONTS`, `setup_work_dir()`, dan `build_burn_args()` menyediakan font dan `fontsdir=fonts`.
- `caption-core/src/bin/caption-cli.rs`: `render_frame()` menghasilkan frame pembanding dengan konfigurasi font tersebut.
- `src-tauri/src/commands.rs`: jalur burn memanggil `setup_work_dir()` sebelum menjalankan FFmpeg.

Tidak ada file source atau preset yang diubah. Burn melalui tombol aplikasi, video pengguna, resolusi landscape, dan ukuran preview lain belum diuji dalam validasi ini.

---

# Phase 3A - Pengukuran ukuran glyph Preview vs ASS/libass

## Fixture dan metode pengukuran

Semua fixture dibuat di `%TEMP%\caption-glyph-phase3a-20261124`, terpisah dari source produksi, lalu dibersihkan sesudah pengukuran. Empat file TTF diambil byte-for-byte dari `caption-core/assets/fonts/`; SHA-256-nya kembali cocok dengan file di `src/assets/fonts/`.

Fixture browser dibuat sebagai elemen DOM sementara di halaman aplikasi, tanpa mengubah file aplikasi. Fixture ASS dibuat sebagai skrip ASS minimal dan dirender langsung memakai `target/debug/ffmpeg.exe` dengan `fontsdir=fonts`. Keduanya menggunakan resolusi tepat 1080×1920, ukuran nominal 104 px, latar hitam, glyph putih, teks satu baris, tanpa letter spacing, outline, shadow, glow, box, atau emphasis. Posisi hanya dipakai untuk menaruh teks di tengah; ukuran bbox, bukan koordinatnya, yang dibandingkan.

Pemetaan family/weight yang diuji:

| Font | CSS family / weight | ASS Fontname / Bold | File yang dipilih |
|---|---|---|---|
| Poppins Bold | `Poppins`, 700 | `Poppins`, -1 | `Poppins-Bold.ttf` |
| Poppins ExtraBold | `Poppins ExtraBold`, 400 | `Poppins ExtraBold`, 0 | `Poppins-ExtraBold.ttf` |
| Anton Regular | `Anton`, 400 | `Anton`, 0 | `Anton-Regular.ttf` |
| Bangers Regular | `Bangers`, 400 | `Bangers`, 0 | `Bangers-Regular.ttf` |

Browser menangkap PNG 1080×1920 pada `devicePixelRatio=1`, jadi normalisasi ke resolusi video adalah 1:1. Chromium `CSS.getPlatformFontsForNode` melaporkan PostScript name `Poppins-Bold`, `Poppins-ExtraBold`, `Anton-Regular`, dan `Bangers-Regular`; `document.fonts.load()` berhasil untuk setiap family. Log debug FFmpeg/libass memilih PostScript name yang sama untuk seluruh skrip ASS.

Untuk kedua jenis PNG, FFmpeg menerapkan masker identik: konversi grayscale, ambang `Y > 127`, lalu `bbox` pada piksel di atas ambang. Ini mengukur isi glyph putih dan mengecualikan latar hitam; efek caption memang tidak ada. Ambang yang sama membuat definisi tepi glyph konsisten, walaupun piksel antialias di bawah ambang tidak ikut dihitung.

## Hasil bbox dan rasio Preview/libass

| Font | Teks | Preview W×H | libass W×H | Rasio W | Rasio H |
|---|---|---:|---:|---:|---:|
| Poppins Bold | LAGI | 224×75 | 126×42 | 1.778 | 1.786 |
| Poppins Bold | MULAI | 313×74 | 178×41 | 1.758 | 1.805 |
| Poppins Bold | VIDEO | 317×75 | 179×42 | 1.771 | 1.786 |
| Poppins ExtraBold | LAGI | 229×75 | 130×42 | 1.762 | 1.786 |
| Poppins ExtraBold | MULAI | 323×75 | 182×42 | 1.775 | 1.786 |
| Poppins ExtraBold | VIDEO | 322×76 | 182×42 | 1.769 | 1.810 |
| Anton Regular | LAGI | 160×91 | 92×53 | 1.739 | 1.717 |
| Anton Regular | MULAI | 236×90 | 136×52 | 1.735 | 1.731 |
| Anton Regular | VIDEO | 213×91 | 123×53 | 1.732 | 1.717 |
| Bangers Regular | LAGI | 158×80 | 90×45 | 1.756 | 1.778 |
| Bangers Regular | MULAI | 218×80 | 124×46 | 1.758 | 1.739 |
| Bangers Regular | VIDEO | 194×79 | 110×44 | 1.764 | 1.795 |

Rasio rata-rata per family:

| Font | Rata-rata rasio W | Rata-rata rasio H |
|---|---:|---:|
| Poppins Bold | 1.769 | 1.792 |
| Poppins ExtraBold | 1.769 | 1.794 |
| Anton Regular | 1.735 | 1.722 |
| Bangers Regular | 1.759 | 1.771 |
| Semua sampel | 1.758 | 1.770 |

## Kesimpulan dan rekomendasi

- **Perbedaan konsisten? CONFIRMED.** Pada 12 pasangan font/teks, bbox Preview selalu lebih besar daripada libass, baik lebar maupun tinggi. Rata-rata Preview/libass sekitar 1.76× untuk lebar dan 1.77× untuk tinggi.
- **Perlu faktor berbeda per family?** Belum ada bukti kuat untuk faktor per-family: rata-rata lebar per family berada di rentang 1.735–1.769 dan tinggi 1.722–1.794. Selisih kecil tersebut dapat dipengaruhi metrik glyph dan ambang antialias; satu faktor global lebih didukung oleh sampel ini.
- **Perlu koreksi ukuran ASS?** Jika sasaran adalah menyamakan ukuran glyph dengan Preview pada fixture ini, ya: `Fontsize=104` di ASS menghasilkan glyph lebih kecil dan perlu kalibrasi naik sekitar faktor 1.76. Ini hasil pengukuran, bukan perubahan yang diterapkan.
- **Cap-height/ascent/descent atau rasterisasi?** Lebar dan tinggi sama-sama berbeda hampir dengan faktor yang sama. Ini lebih mengindikasikan perbedaan skala efektif font-size antara CSS px dan ukuran ASS/libass daripada perbedaan cap-height saja. Bbox tidak cukup untuk memisahkan kontribusi ascent/descent, shaping/kerning, atau rasterisasi; posisi baseline juga bukan sasaran ukur fase ini.
- **Implementasi terkecil yang disarankan:** bila koreksi disetujui, kalibrasikan ukuran ASS di satu titik, `caption-core/src/preset_render.rs::build_style_line()`, tanpa mengubah CSS preview. Sebelum menetapkan faktor global permanen, ulangi fixture pada beberapa ukuran nominal (misalnya 72, 104, dan 144), lalu konfirmasi lewat burn Tauri nyata; fase ini hanya menguji ukuran 104 dan frame FFmpeg terisolasi.

Tidak ada perubahan pada `preview.js`, `app.css`, preset, renderer, atau pipeline burn. Status `CONFIRMED` berlaku untuk selisih ukuran glyph pada fixture terkontrol ini; faktor koreksi universal dan penyebab internal tepatnya belum dikonfirmasi.

---

# Phase 3B - Validasi linearitas faktor koreksi ukuran ASS

## Metode dan kontrol fixture

Fixture Phase 3B dibuat di `%TEMP%\caption-glyph-phase3b-20261124`, tidak di source produksi. Untuk masing-masing empat font, tiga ukuran (72, 104, 144 px), dan tiga teks (LAGI, MULAI, VIDEO), dibuat satu PNG browser dan satu PNG ASS/libass: total **36 pasangan / 72 PNG**. Semua PNG diverifikasi berukuran tepat 1080×1920.

Kondisi kedua renderer sama: latar hitam, glyph putih, ukuran CSS `font-size` sama dengan ASS `Fontsize`, teks satu baris, tanpa outline, shadow, glow, box, letter spacing, atau emphasis. Posisi hanya anchor tengah dan tidak digunakan sebagai ukuran. Browser berjalan pada `devicePixelRatio=1`, sehingga screenshot 1080×1920 langsung cocok dengan resolusi video tanpa faktor normalisasi tambahan. Weight yang dipakai: Poppins Bold CSS 700 / ASS Bold -1; Poppins ExtraBold CSS 400 / ASS Bold 0; Anton dan Bangers CSS 400 / ASS Bold 0.

`document.fonts.load()` berhasil untuk setiap ukuran/family. Chromium `CSS.getPlatformFontsForNode` dan log debug `fontselect` libass memilih PostScript font yang dimaksud. SHA-256 file preview dan bundled kembali cocok untuk keempat font. Frame ASS dirender dengan FFmpeg/libass dan `fontsdir=fonts`; burn tombol Tauri tidak dijalankan pada fase ini.

Kedua set PNG diproses memakai masker grayscale yang sama: `Y > 127`, kemudian `bbox` menghitung piksel isi glyph. Ini mengecualikan efek karena fixture tidak mengandung efek, dan menerapkan batas antialias yang sama ke kedua renderer.

## Hasil 36 pasangan

| Font | Ukuran | Teks | Preview W×H | libass W×H | Rasio W | Rasio H |
|---|---:|---|---:|---:|---:|---:|
| Poppins Bold | 72 | LAGI | 155×52 | 88×30 | 1.7614 | 1.7333 |
| Poppins Bold | 72 | MULAI | 217×52 | 122×29 | 1.7787 | 1.7931 |
| Poppins Bold | 72 | VIDEO | 219×52 | 124×30 | 1.7661 | 1.7333 |
| Poppins Bold | 104 | LAGI | 224×75 | 126×42 | 1.7778 | 1.7857 |
| Poppins Bold | 104 | MULAI | 313×74 | 178×41 | 1.7584 | 1.8049 |
| Poppins Bold | 104 | VIDEO | 317×75 | 179×42 | 1.7709 | 1.7857 |
| Poppins Bold | 144 | LAGI | 309×103 | 176×58 | 1.7557 | 1.7759 |
| Poppins Bold | 144 | MULAI | 433×102 | 246×58 | 1.7602 | 1.7586 |
| Poppins Bold | 144 | VIDEO | 438×104 | 248×58 | 1.7661 | 1.7931 |
| Poppins ExtraBold | 72 | LAGI | 159×52 | 90×30 | 1.7667 | 1.7333 |
| Poppins ExtraBold | 72 | MULAI | 223×52 | 126×29 | 1.7698 | 1.7931 |
| Poppins ExtraBold | 72 | VIDEO | 223×53 | 126×30 | 1.7698 | 1.7667 |
| Poppins ExtraBold | 104 | LAGI | 229×75 | 130×42 | 1.7615 | 1.7857 |
| Poppins ExtraBold | 104 | MULAI | 323×75 | 182×42 | 1.7747 | 1.7857 |
| Poppins ExtraBold | 104 | VIDEO | 322×76 | 182×42 | 1.7692 | 1.8095 |
| Poppins ExtraBold | 144 | LAGI | 317×104 | 180×58 | 1.7611 | 1.7931 |
| Poppins ExtraBold | 144 | MULAI | 445×103 | 253×58 | 1.7589 | 1.7759 |
| Poppins ExtraBold | 144 | VIDEO | 445×104 | 252×59 | 1.7659 | 1.7627 |
| Anton Regular | 72 | LAGI | 111×64 | 64×37 | 1.7344 | 1.7297 |
| Anton Regular | 72 | MULAI | 164×63 | 94×36 | 1.7447 | 1.7500 |
| Anton Regular | 72 | VIDEO | 148×64 | 85×37 | 1.7412 | 1.7297 |
| Anton Regular | 104 | LAGI | 160×91 | 92×53 | 1.7391 | 1.7170 |
| Anton Regular | 104 | MULAI | 236×90 | 136×52 | 1.7353 | 1.7308 |
| Anton Regular | 104 | VIDEO | 213×91 | 123×53 | 1.7317 | 1.7170 |
| Anton Regular | 144 | LAGI | 221×126 | 127×72 | 1.7402 | 1.7500 |
| Anton Regular | 144 | MULAI | 326×125 | 188×72 | 1.7340 | 1.7361 |
| Anton Regular | 144 | VIDEO | 295×126 | 170×72 | 1.7353 | 1.7500 |
| Bangers Regular | 72 | LAGI | 109×56 | 62×31 | 1.7581 | 1.8065 |
| Bangers Regular | 72 | MULAI | 151×54 | 85×31 | 1.7765 | 1.7419 |
| Bangers Regular | 72 | VIDEO | 134×54 | 76×31 | 1.7632 | 1.7419 |
| Bangers Regular | 104 | LAGI | 158×80 | 90×45 | 1.7556 | 1.7778 |
| Bangers Regular | 104 | MULAI | 218×80 | 124×46 | 1.7581 | 1.7391 |
| Bangers Regular | 104 | VIDEO | 194×79 | 110×44 | 1.7636 | 1.7955 |
| Bangers Regular | 144 | LAGI | 218×112 | 124×64 | 1.7581 | 1.7500 |
| Bangers Regular | 144 | MULAI | 302×111 | 171×63 | 1.7661 | 1.7619 |
| Bangers Regular | 144 | VIDEO | 268×108 | 152×61 | 1.7632 | 1.7705 |

## Analisis rasio

Rasio didefinisikan sebagai bbox Preview dibagi bbox libass. Faktor global di bawah adalah mean aritmetika dari 72 rasio W/H individual, dengan bobot sama untuk setiap ukuran, family, teks, dan sumbu.

| Kelompok | Rata-rata rasio W | Rata-rata rasio H |
|---|---:|---:|
| Ukuran 72 px | 1.76087 | 1.75439 |
| Ukuran 104 px | 1.75800 | 1.76953 |
| Ukuran 144 px | 1.75539 | 1.76481 |
| Poppins Bold | 1.76615 | 1.77374 |
| Poppins ExtraBold | 1.76641 | 1.77841 |
| Anton Regular | 1.73732 | 1.73448 |
| Bangers Regular | 1.76247 | 1.76501 |
| Semua sampel | 1.75808 | 1.76291 |

- **Faktor global terbaik dari seluruh sumbu:** **1.76050** (Preview/libass). Untuk mengubahnya menjadi perkiraan skala ukuran ASS terhadap ukuran Preview, nilai font ASS perlu dikalibrasi naik sekitar faktor ini.
- Rata-rata W turun dari 1.76087 pada 72 px ke 1.75539 pada 144 px, perubahan **-0.00548** (sekitar 0.31%). Rata-rata H berubah non-monotonik; rentangnya 1.75439–1.76953, atau **0.01514** (sekitar 0.86%).
- Rasio individual minimum **1.71698**, yaitu 0.04352 di bawah faktor global (sekitar -2.47%). Rasio maksimum **1.80952**, yaitu 0.04902 di atas faktor global (sekitar +2.78%). Deviasi absolut terburuk sekitar **2.78%**.

## Kesimpulan dan status

- **Faktor koreksi global layak digunakan:** Ya untuk fixture ini. Rata-rata antarukuran hampir konstan dan seluruh sampel berada dalam sekitar ±2.8% dari faktor global 1.76050.
- **Faktor berbeda per family diperlukan:** Tidak untuk toleransi sekitar 3%. Namun Anton memiliki rata-rata sekitar 1.735, lebih rendah daripada Poppins sekitar 1.766–1.778; bila target parity memerlukan toleransi di bawah 1%, faktor per-family patut diuji, bukan diasumsikan tidak perlu.
- **Faktor berubah menurut ukuran:** Tidak ada tren bermakna pada ukuran 72–144 px. Pergeseran rata-rata ukuran di bawah 1%; perbedaan lebar/tinggi yang tersisa lebih besar daripada drift terhadap ukuran.
- **Cukup konsisten untuk masuk tahap implementasi:** Ya untuk mencoba faktor global pada satu perubahan ASS yang terisolasi, dengan validasi burn Tauri sebagai gerbang sebelum menyatakan parity terkonfirmasi. Data ini tidak menguji efek, wrapping, font-size di luar rentang 72–144 px, teks lowercase, atau variasi rasterizer di mesin lain.
- **Status akhir: CONFIRMED** untuk linearitas praktis faktor ukuran pada empat font, tiga teks, dan tiga ukuran nominal yang diuji. Faktor global 1.76050 didukung oleh fixture; penyebab internal tepat dan hasil burn Tauri nyata masih perlu diverifikasi.

Tidak ada perubahan pada source produksi, CSS preview, preset, atau renderer. Seluruh fixture sementara sudah dihapus setelah hasil pengukuran dicatat.
