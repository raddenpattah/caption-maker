# Laporan Fase "Mesin Gaya + Burn-in"

## 1. Ringkasan

- Skema preset gaya (12 preset JSON) + generator ASS berbasis preset dengan 5 mode emphasis selesai.
- Probe video (ffprobe) + pembangunan argumen burn-in FFmpeg selesai. Font Poppins, Anton, Bangers (OFL) disertakan.
- CLI: --list-presets, --preset, --burn, --quality, --render-frame, --at-ms, --words-json, --size, --ffprobe. Jalur lama byte-identik.
- Script menghasilkan 24 PNG 1080x1920 + burn_test.mp4 1080x1920 audio AAC.
- Semua fmt, Clippy, tes, diff baseline hijau.

**Semua klaim visual: NOT RUN.**

## 2. Checklist tugas

| # | Tugas | Status | Bukti |
|---|---|---|---|
| 1 | Skema preset + 12 JSON + validasi | PASS | preset_tests 10/10 |
| 2 | Generator ASS + 5 emphasis mode | PASS | ass_preset_tests 19/19 |
| 3 | Probe + burn-in FFmpeg | PASS | video_tests 23/23 |
| 4 | CLI flag baru, lama utuh | PASS | cli_smoke_tests 5/5, ASS IDENTIK |
| 5 | Script render sheet + video uji | PASS (eksekusi); NOT RUN (visual) | 24 PNG, burn 1080x1920 aac |
| 6 | README + roadmap | PASS | README 151 baris |
| 7 | Verifikasi | PASS | semua exit 0, ASS IDENTIK |
| 8 | Laporan ini | PASS | - |

## 3. Output mentah

- `cargo fmt --all -- --check`: exit 0, kosong.
- `cargo clippy --workspace --all-targets -- -D warnings`: exit 0.
- `cargo clippy --workspace --all-targets --no-default-features -- -D warnings`: exit 0.
- `cargo test --workspace --no-default-features`: 93 lulus, 0 gagal.
- `cargo test --workspace`: 98 lulus, 0 gagal (termasuk 5 CLI smoke).
- Baseline: `tokens=92 words=48 groups=24`, `diff` = `ASS IDENTIK`.
- Script: burn 1080x1920, durasi 23.185011, audio `aac`.
- PNG sample: `1080,1920`.

## 4. Font + lisensi + sumber

| Font | File | Lisensi | Sumber |
|---|---|---|---|
| Poppins Bold | Poppins-Bold.ttf | OFL | google/fonts/ofl/poppins |
| Poppins ExtraBold | Poppins-ExtraBold.ttf | OFL | google/fonts/ofl/poppins |
| Anton | Anton-Regular.ttf | OFL | google/fonts/ofl/anton |
| Bangers | Bangers-Regular.ttf | OFL | google/fonts/ofl/bangers |

3 keluarga font unik. Tiap file disertai OFL.txt.

## 5. Variable vs static font

Semua TTF statis (bukan variable). Pengujian libass dengan variable font: NOT RUN (tidak ada variable font).

## 6. Deviasi

Tidak ada deviasi dari instruksi.

## 7. Asumsi

1. Zona aman platform (alignment=2, margin_v=450, margin_lr=80) belum diuji.
2. Latar frame default #202020.
3. --words-json format flat sama dengan --dump-words.
4. --preset + --highlight/--words: preset menang, cetak peringatan.
5. Font fallback Arial hanya runtime, bukan distribusi.
6. -ss pada --render-frame tidak berpengaruh pada lavfi color statis.
7. tokens=92 dipertahankan di jalur lama; --words-json laporkan tokens=0.

## 8. NOT RUN (tidak bisa dinilai tanpa visual)

- Apakah preset terlihat bagus.
- Apakah margin_v=450 benar-benar di zona aman TikTok/Reels/Shorts.
- Apakah glow cukup halus.
- Apakah scale_active menggelembungkan kata tanpa menabrak margin.
- Apakah kotak proporsional.
- Sinkronisasi caption vs audio di video burn-in.
- Kualitas warna HDR (tidak ada tone mapping).

## 9. Saran langkah berikutnya

1. Verifikasi visual oleh manusia dari out/presets/*.png dan out/burn_test.mp4.
2. Setelah lolos, tandai [x] roadmap untuk burn-in dan template gaya.
3. Uji burn-in pada input HDR, audio non-AAC, dan rotasi 90/270.
4. Uji --render-frame dengan input video asli.
5. Lanjut Fase 3B (Tauri), Fase 4 (packaging), Fase 5 (rilis v0.1.0).
