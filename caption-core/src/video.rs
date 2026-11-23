//! Probe video (ffprobe) dan pembangunan argumen burn-in (ffmpeg + libass).
//!
//! Modul ini bebas `whisper` dan `clap`, sehingga bisa dites dengan
//! `--no-default-features`. Pemanggilan FFmpeg/ffprobe sungguhan
//! dilakukan pemanggil (CLI), bukan modul ini — agar mudah dites tanpa
//! FFmpeg terpasang.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static WORK_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Font yang ditanam ke binary lewat `include_bytes!`.
pub struct BundledFont {
    pub file: &'static str,
    pub bytes: &'static [u8],
}

/// Font yang disertakan untuk burn-in.
pub const BUNDLED_FONTS: &[BundledFont] = &[
    BundledFont {
        file: "Poppins-Bold.ttf",
        bytes: include_bytes!("../assets/fonts/Poppins-Bold.ttf"),
    },
    BundledFont {
        file: "Poppins-ExtraBold.ttf",
        bytes: include_bytes!("../assets/fonts/Poppins-ExtraBold.ttf"),
    },
    BundledFont {
        file: "Anton-Regular.ttf",
        bytes: include_bytes!("../assets/fonts/Anton-Regular.ttf"),
    },
    BundledFont {
        file: "Bangers-Regular.ttf",
        bytes: include_bytes!("../assets/fonts/Bangers-Regular.ttf"),
    },
];

/// Ringkasan video dari ffprobe.
#[derive(Debug, Clone, PartialEq)]
pub struct VideoInfo {
    pub width: u32,
    pub height: u32,
    /// Durasi dalam detik; `0.0` berarti ffprobe tidak memberikannya.
    pub duration_s: f64,
    pub audio_codec: Option<String>,
    pub pix_fmt: String,
    pub is_hdr: bool,
}

/// Kualitas burn-in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Quality {
    Cepat,
    Bagus,
}

impl Quality {
    pub fn crf(self) -> &'static str {
        match self {
            Quality::Cepat => "23",
            Quality::Bagus => "18",
        }
    }
    pub fn preset(self) -> &'static str {
        match self {
            Quality::Cepat => "veryfast",
            Quality::Bagus => "slow",
        }
    }
}

/// Parse JSON keluaran `ffprobe -print_format json -show_streams -show_format`.
/// Fungsi ini tidak memanggil ffprobe; pemanggil yang melakukannya.
pub fn parse_ffprobe_json(json: &str) -> Result<VideoInfo, String> {
    let v: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("ffprobe JSON tidak valid: {e}"))?;

    let streams = v
        .get("streams")
        .and_then(|s| s.as_array())
        .ok_or("ffprobe JSON: 'streams' tidak ada")?;

    let mut video_stream: Option<&serde_json::Value> = None;
    let mut audio_codec: Option<String> = None;
    for s in streams {
        let codec_type = s.get("codec_type").and_then(|x| x.as_str()).unwrap_or("");
        if codec_type == "video" && video_stream.is_none() {
            video_stream = Some(s);
        } else if codec_type == "audio" && audio_codec.is_none() {
            audio_codec = s
                .get("codec_name")
                .and_then(|x| x.as_str())
                .map(|x| x.to_string());
        }
    }
    let vs = video_stream.ok_or("ffprobe JSON: stream video tidak ada")?;

    let mut width = vs
        .get("width")
        .and_then(|x| x.as_u64())
        .ok_or("ffprobe JSON: lebar video tidak ada")? as u32;
    let mut height = vs
        .get("height")
        .and_then(|x| x.as_u64())
        .ok_or("ffprobe JSON: tinggi video tidak ada")? as u32;

    // Rotasi: side_data displaymatrix rotation, atau tags.rotate.
    let rotation = detect_rotation(vs);
    if rotation == 90 || rotation == 270 {
        std::mem::swap(&mut width, &mut height);
    }

    let duration_s = v
        .get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .and_then(|d| d.parse::<f64>().ok())
        .or_else(|| {
            vs.get("duration")
                .and_then(|d| d.as_str())
                .and_then(|d| d.parse::<f64>().ok())
        })
        .unwrap_or(0.0);

    let pix_fmt = vs
        .get("pix_fmt")
        .and_then(|x| x.as_str())
        .unwrap_or("")
        .to_string();

    let is_hdr = detect_hdr(vs, &pix_fmt);

    Ok(VideoInfo {
        width,
        height,
        duration_s,
        audio_codec,
        pix_fmt,
        is_hdr,
    })
}

fn detect_rotation(vs: &serde_json::Value) -> i64 {
    // 1) side_data_list[].rotation
    if let Some(sd) = vs.get("side_data_list").and_then(|x| x.as_array()) {
        for entry in sd {
            if let Some(r) = entry.get("rotation").and_then(|x| x.as_i64()) {
                return r.rem_euclid(360);
            }
        }
    }
    // 2) tags.rotate (string)
    if let Some(r) = vs
        .get("tags")
        .and_then(|t| t.get("rotate"))
        .and_then(|x| x.as_str())
        .and_then(|s| s.parse::<i64>().ok())
    {
        return r.rem_euclid(360);
    }
    0
}

fn detect_hdr(vs: &serde_json::Value, pix_fmt: &str) -> bool {
    let transfer = vs
        .get("color_transfer")
        .and_then(|x| x.as_str())
        .unwrap_or("");
    if transfer == "smpte2084" || transfer == "arib-std-b67" {
        return true;
    }
    // 10-bit pix_fmt biasanya HDR-capable (yuv420p10le, yuv422p10le, dll).
    pix_fmt.contains("10le") || pix_fmt.contains("10be")
}

/// Bangun argumen ffmpeg untuk burn-in. Pemanggil yang menjalankan proses.
///
/// `work_dir` diasumsikan berisi `sub.ass` dan `fonts/`. Filter memakai
/// path RELATIF (`sub.ass`) agar aman dari escape path Windows.
pub fn build_burn_args(
    input: &Path,
    work_dir: &Path,
    output: &Path,
    quality: Quality,
    info: &VideoInfo,
) -> Vec<String> {
    let mut args: Vec<String> = Vec::new();
    args.push("-y".into());
    args.push("-i".into());
    args.push(input.to_string_lossy().into_owned());

    let filter = "ass=sub.ass:fontsdir=fonts".to_string();
    args.push("-vf".into());
    args.push(filter);

    args.push("-c:v".into());
    args.push("libx264".into());
    args.push("-crf".into());
    args.push(quality.crf().into());
    args.push("-preset".into());
    args.push(quality.preset().into());
    args.push("-pix_fmt".into());
    args.push("yuv420p".into());
    args.push("-movflags".into());
    args.push("+faststart".into());

    match info.audio_codec.as_deref() {
        Some("aac") => {
            args.push("-c:a".into());
            args.push("copy".into());
        }
        Some(_) => {
            args.push("-c:a".into());
            args.push("aac".into());
            args.push("-b:a".into());
            args.push("192k".into());
        }
        None => {}
    }

    args.push("-progress".into());
    args.push("pipe:1".into());
    args.push("-nostats".into());

    args.push(output.to_string_lossy().into_owned());

    // work_dir dipakai pemanggil sebagai current_dir; dikembalikan lewat
    // parameter agar pemanggil tahu harus chdir ke mana.
    let _ = work_dir;

    args
}

/// Tulis `sub.ass` + ekstrak font ke `work_dir/fonts/`.
pub fn setup_work_dir(work_dir: &Path, ass_content: &str) -> Result<(), String> {
    std::fs::create_dir_all(work_dir).map_err(|e| format!("buat work_dir: {e}"))?;
    std::fs::write(work_dir.join("sub.ass"), ass_content)
        .map_err(|e| format!("tulis sub.ass: {e}"))?;
    let fonts_dir = work_dir.join("fonts");
    std::fs::create_dir_all(&fonts_dir).map_err(|e| format!("buat fonts dir: {e}"))?;
    for font in BUNDLED_FONTS {
        std::fs::write(fonts_dir.join(font.file), font.bytes)
            .map_err(|e| format!("tulis font {}: {e}", font.file))?;
    }
    Ok(())
}

/// Direktori kerja unik yang selalu dibersihkan ketika guard di-drop.
pub struct WorkDirGuard {
    path: PathBuf,
}

impl WorkDirGuard {
    pub fn new(prefix: &str) -> Result<Self, String> {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| format!("waktu sistem tidak valid: {error}"))?
            .as_nanos();
        let counter = WORK_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("{prefix}-{}-{nonce}-{counter}", std::process::id()));
        std::fs::create_dir(&path)
            .map_err(|error| format!("buat direktori sementara '{}': {error}", path.display()))?;
        Ok(Self { path })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn cleanup(&mut self) -> Result<(), String> {
        if self.path.as_os_str().is_empty() {
            return Ok(());
        }
        match std::fs::remove_dir_all(&self.path) {
            Ok(()) => {
                self.path.clear();
                Ok(())
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.path.clear();
                Ok(())
            }
            Err(error) => Err(format!(
                "hapus direktori sementara '{}': {error}",
                self.path.display()
            )),
        }
    }
}

impl Drop for WorkDirGuard {
    fn drop(&mut self) {
        if !self.path.as_os_str().is_empty() {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

/// Satu baris progress ffmpeg (`out_time_us=...` atau `progress=end`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgressUpdate {
    pub out_time_us: Option<u64>,
    pub finished: bool,
}

pub fn parse_progress_line(line: &str) -> Option<ProgressUpdate> {
    let line = line.trim();
    if line.is_empty() {
        return None;
    }
    if line == "progress=end" {
        return Some(ProgressUpdate {
            out_time_us: None,
            finished: true,
        });
    }
    if let Some(v) = line.strip_prefix("out_time_us=") {
        let us = v.parse::<u64>().ok()?;
        return Some(ProgressUpdate {
            out_time_us: Some(us),
            finished: false,
        });
    }
    if let Some(v) = line.strip_prefix("out_time_ms=") {
        // ffmpeg -progress melaporkan out_time_ms dalam mikrodetik (bug historis).
        let us = v.parse::<u64>().ok()?;
        return Some(ProgressUpdate {
            out_time_us: Some(us),
            finished: false,
        });
    }
    None
}

/// Persen 0..=100 dari `out_time_us` terhadap durasi detik.
pub fn percent(done_us: u64, duration_s: f64) -> u8 {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return 0;
    }
    let total_us = duration_s * 1_000_000.0;
    let p = (done_us as f64 / total_us) * 100.0;
    p.clamp(0.0, 100.0).round() as u8
}

/// Path ffprobe default: di samping ffmpeg, dengan ekstensi disesuaikan.
/// Tahan terhadap path bergaya Windows meski dijalankan di Linux.
pub fn default_ffprobe_for(ffmpeg: &Path) -> PathBuf {
    let s = ffmpeg.to_string_lossy();
    // Deteksi pemisah path: pakai '\' jika ada, jika tidak '/'.
    let (dir_part, file_part) = if let Some(idx) = s.rfind(['/', '\\']) {
        (&s[..=idx], &s[idx + 1..])
    } else {
        ("", s.as_ref())
    };
    // Pisahkan ekstensi .exe (atau apa pun setelah titik terakhir) secara manual.
    let (file_stem, ext) = match file_part.rfind('.') {
        Some(dot) => (&file_part[..dot], Some(&file_part[dot..])),
        None => (file_part, None),
    };
    let new_stem = if file_stem.eq_ignore_ascii_case("ffmpeg") {
        "ffprobe".to_string()
    } else {
        format!("{file_stem}_probe")
    };
    let mut out = String::with_capacity(dir_part.len() + new_stem.len() + 8);
    out.push_str(dir_part);
    out.push_str(&new_stem);
    if let Some(ext) = ext {
        out.push_str(ext);
    }
    PathBuf::from(out)
}

/// Bangun path relatif `sub.ass` + `fonts` di bawah `work_dir`.
/// Dipakai filter FFmpeg agar tidak perlu escape path absolut (aman di Windows).
pub fn filter_relative_paths() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("sub", "sub.ass");
    m.insert("fonts", "fonts");
    m
}
