# FFmpeg Sidecar

Letakkan file binary FFmpeg untuk Windows di direktori ini sebelum build:

    ffmpeg-x86_64-pc-windows-msvc.exe

Nama file harus sesuai dengan target triple Tauri (untuk Windows x64: `x86_64-pc-windows-msvc`).

Contoh unduhan resmi (GPL build, karena butuh x264):
- https://www.gyan.dev/ffmpeg/builds/ (pilih `ffmpeg-release-essentials.zip`)

File binary ini **tidak dilacak Git** (lihat `.gitignore`).
