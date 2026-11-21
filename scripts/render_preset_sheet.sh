#!/usr/bin/env bash
# Render satu PNG per preset (untuk penilaian visual manusia).
# Semua klaim visual = NOT RUN; script ini hanya menghasilkan file.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BIN="$ROOT/target/release/caption-cli"
if [[ ! -x "$BIN" ]]; then
  echo "Binary $BIN belum ada. Jalankan dulu: cargo build --release --bin caption-cli"
  exit 1
fi

OUT_DIR="$ROOT/out/presets"
mkdir -p "$OUT_DIR"

WORDS_JSON="$ROOT/src/mock/words.json"
SIZE="1080x1920"
FRAMES=(4200 1500)

# Ambil daftar id dari --list-presets
mapfile -t IDS < <("$BIN" --list-presets | awk -F'\t' '{print $1}')
if [[ ${#IDS[@]} -eq 0 ]]; then
  echo "Tidak ada preset dari --list-presets"
  exit 1
fi

echo "== Render ${#IDS[@]} preset x ${#FRAMES[@]} frame =="
for id in "${IDS[@]}"; do
  for ms in "${FRAMES[@]}"; do
    out="$OUT_DIR/${id}__${ms}.png"
    echo "-- $id @ ${ms}ms -> $out"
    "$BIN" \
      --preset "$id" \
      --words-json "$WORDS_JSON" \
      --size "$SIZE" \
      --render-frame "$out" \
      --at-ms "$ms"
  done
done

echo "== Verifikasi PNG =="
for f in "$OUT_DIR"/*.png; do
  [[ -f "$f" ]] || continue
  # ffprobe untuk memastikan ukuran
  w=$(ffprobe -v error -select_streams v:0 -show_entries stream=width -of csv=p=0 "$f")
  h=$(ffprobe -v error -select_streams v:0 -show_entries stream=height -of csv=p=0 "$f")
  echo "$f ${w}x${h}"
done

echo "== Buat video uji vertikal sintetis =="
TEST_MP4="$ROOT/out/vertical_test.mp4"
ffmpeg -y -v error \
  -f lavfi -i "color=c=0x202020:s=1080x1920:r=30" \
  -i "$ROOT/samples/test.wav" \
  -c:v libx264 -preset veryfast -pix_fmt yuv420p \
  -c:a aac -b:a 192k -shortest \
  "$TEST_MP4"
echo "OK $TEST_MP4"

echo "== Burn-in satu preset (tekan-warna-kuning) =="
BURN_MP4="$ROOT/out/burn_test.mp4"
"$BIN" \
  --preset tekan-warna-kuning \
  --words-json "$WORDS_JSON" \
  "$TEST_MP4" \
  --burn "$BURN_MP4" \
  --quality bagus

echo "== Verifikasi burn =="
ffprobe -v error -select_streams v:0 -show_entries stream=width,height -of csv=p=0 "$BURN_MP4"
ffprobe -v error -show_entries format=duration -of csv=p=0 "$BURN_MP4"
ffprobe -v error -select_streams a -show_entries stream=codec_name -of csv=p=0 "$BURN_MP4"

echo "== Selesai. File di out/. Penilaian visual = NOT RUN =="
