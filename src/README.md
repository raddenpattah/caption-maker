# Antarmuka Caption Maker

Frontend ini memakai HTML, CSS, dan JavaScript ES modules tanpa framework atau build step. Semua aset dan data contoh berada di lokal. Jalankan server statis dari direktori proyek:

```sh
python3 -m http.server 8080 --directory src
```

Buka `http://localhost:8080` di browser.

Backend saat ini adalah mock: daftar model, progress transkripsi, dan kata hasil transkripsi bukan panggilan ke Tauri. Data preview berasal dari `mock/words.json`. Ekspor `.ass` nyata belum tersedia dan baru dihubungkan pada Fase 3B.

Pratinjau karaoke dibuat dengan elemen HTML/CSS dan pengisian warna progresif sebagai simulasi tag karaoke ASS (`\kf`); ini bukan renderer ASS dan tampilannya dapat berbeda dari pemutar subtitle.
