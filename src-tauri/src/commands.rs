//! Command Tauri: jembatan UI ke `caption-core`.
//!
//! Semua command yang mengembalikan file path memakai `String` (bukan `PathBuf`)
//! karena Tauri membutuhkan tipe serde-serializable.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Condvar, Mutex};
use std::thread;
#[cfg(feature = "whisper")]
use std::time::Instant;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Manager};

use caption_core::model::Word;
use caption_core::preset::{
    Animation, Emphasis, PresetBox, PresetColors, PresetFont, PresetGlow, StylePreset,
};

// ---------- Tipe data bersama ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub filename: String,
    pub size_bytes: Option<u64>,
    pub estimated_size_bytes: u64,
    pub installed: bool,
}

pub(crate) const MIN_MODEL_SIZE_BYTES: u64 = 20 * 1024 * 1024;
pub(crate) const WHISPER_MODEL_MAGIC: [u8; 4] = *b"lmgg";

const MODEL_CATALOG: [(&str, &str, u64); 4] = [
    ("tiny", "Tiny", 75 * 1024 * 1024),
    ("base", "Base", 142 * 1024 * 1024),
    ("small", "Small", 466 * 1024 * 1024),
    ("medium", "Medium", 1530 * 1024 * 1024),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordDto {
    pub text: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

impl From<WordDto> for Word {
    fn from(dto: WordDto) -> Self {
        Word {
            text: dto.text,
            start_ms: dto.start_ms,
            end_ms: dto.end_ms,
        }
    }
}

impl From<&Word> for WordDto {
    fn from(w: &Word) -> Self {
        Self {
            text: w.text.clone(),
            start_ms: w.start_ms,
            end_ms: w.end_ms,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetSummary {
    pub id: String,
    pub name: String,
    pub category: String,
    pub words_per_line: u8,
    pub font: PresetFont,
    pub colors: PresetColors,
    pub outline: u32,
    pub shadow: u32,
    pub margin_v: u32,
    pub margin_lr: u32,
    pub glow: Option<PresetGlow>,
    pub emphasis: Emphasis,
    pub emphasis_scale: Option<u32>,
    pub animation: Animation,
    #[serde(rename = "box")]
    pub box_: Option<PresetBox>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum WordsPerLineWire {
    Number(u8),
    Sentence(SentenceWire),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SentenceWire {
    Sentence,
}

impl WordsPerLineWire {
    fn into_words_per_line(self) -> Result<caption_core::WordsPerLine, String> {
        match self {
            Self::Number(1) => Ok(caption_core::WordsPerLine::One),
            Self::Number(2) => Ok(caption_core::WordsPerLine::Two),
            Self::Number(3) => Ok(caption_core::WordsPerLine::Three),
            Self::Sentence(SentenceWire::Sentence) => Ok(caption_core::WordsPerLine::Sentence),
            Self::Number(n) => Err(format!("words_per_line tidak valid: {n}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CaptionPositionWire {
    pub x_percent: f64,
    pub y_percent: f64,
}

impl TryFrom<CaptionPositionWire> for caption_core::CaptionPosition {
    type Error = String;

    fn try_from(position: CaptionPositionWire) -> Result<Self, Self::Error> {
        let position = Self {
            x_percent: position.x_percent,
            y_percent: position.y_percent,
        };
        position.validate()?;
        Ok(position)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    pub words: Vec<WordDto>,
    pub preset_id: String,
    pub width: u32,
    pub height: u32,
    pub words_per_line: Option<WordsPerLineWire>,
    #[serde(default)]
    pub caption_position: Option<CaptionPositionWire>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    pub ass: String,
}

// ---------- Command (logika murni, mudah dites) ----------

fn model_filename(id: &str) -> String {
    format!("ggml-{id}.bin")
}

fn valid_model_id(id: &str) -> bool {
    !id.is_empty()
        && id != "."
        && id != ".."
        && id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
}

fn find_model_file(models_dir: &Path, id: &str) -> Result<Option<PathBuf>, String> {
    let expected = model_filename(id);
    let entries = match fs::read_dir(models_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Gagal membaca folder model '{}': {error}",
                models_dir.display()
            ))
        }
    };
    for entry in entries {
        let entry = entry.map_err(|error| format!("Gagal membaca daftar model: {error}"))?;
        let name = entry.file_name();
        if name
            .to_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(&expected))
            && entry
                .file_type()
                .map_err(|error| format!("Gagal memeriksa file model: {error}"))?
                .is_file()
        {
            return Ok(Some(entry.path()));
        }
    }
    Ok(None)
}

fn model_info(
    models_dir: &Path,
    id: &str,
    display_name: &str,
    estimate: u64,
) -> Result<ModelInfo, String> {
    let path = find_model_file(models_dir, id)?;
    let size_bytes = path
        .as_ref()
        .map(|path| {
            fs::metadata(path)
                .map(|metadata| metadata.len())
                .map_err(|error| {
                    format!("Gagal membaca ukuran model '{}': {error}", path.display())
                })
        })
        .transpose()?;
    Ok(ModelInfo {
        id: id.to_owned(),
        name: display_name.to_owned(),
        filename: path
            .as_ref()
            .and_then(|path| path.file_name())
            .and_then(|name| name.to_str())
            .map(str::to_owned)
            .unwrap_or_else(|| model_filename(id)),
        size_bytes,
        estimated_size_bytes: estimate,
        installed: size_bytes.is_some(),
    })
}

/// Membaca katalog dan file model yang benar-benar ada di folder tujuan.
pub fn list_models_impl(models_dir: &Path) -> Result<Vec<ModelInfo>, String> {
    fs::create_dir_all(models_dir).map_err(|error| {
        format!(
            "Gagal membuat folder model '{}': {error}",
            models_dir.display()
        )
    })?;

    let mut models = MODEL_CATALOG
        .iter()
        .map(|(id, name, estimate)| model_info(models_dir, id, name, *estimate))
        .collect::<Result<Vec<_>, _>>()?;
    let mut known_ids = models
        .iter()
        .map(|model| model.id.to_ascii_lowercase())
        .collect::<std::collections::HashSet<_>>();

    let entries = fs::read_dir(models_dir).map_err(|error| {
        format!(
            "Gagal membaca folder model '{}': {error}",
            models_dir.display()
        )
    })?;
    let mut extras = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("Gagal membaca daftar model: {error}"))?;
        if !entry
            .file_type()
            .map_err(|error| format!("Gagal memeriksa file model: {error}"))?
            .is_file()
        {
            continue;
        }
        let Some(filename) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        let Some(id) = model_id_from_filename(&filename) else {
            continue;
        };
        if known_ids.insert(id.to_ascii_lowercase()) {
            extras.push((id, filename, entry.path()));
        }
    }
    extras.sort_by(|left, right| left.0.cmp(&right.0));
    for (id, filename, path) in extras {
        let size_bytes = fs::metadata(&path)
            .map_err(|error| format!("Gagal membaca ukuran model '{}': {error}", path.display()))?
            .len();
        models.push(ModelInfo {
            name: id.clone(),
            id,
            filename,
            size_bytes: Some(size_bytes),
            estimated_size_bytes: 0,
            installed: true,
        });
    }
    Ok(models)
}

fn model_id_from_filename(filename: &str) -> Option<String> {
    let (prefix, extension) = filename.rsplit_once('.')?;
    if !extension.eq_ignore_ascii_case("bin") {
        return None;
    }
    let id = prefix.strip_prefix("ggml-")?;
    if !valid_model_id(id) {
        return None;
    }
    Some(id.to_ascii_lowercase())
}

pub fn resolve_model_path(models_dir: &Path, model_id: &str) -> Result<PathBuf, String> {
    if !valid_model_id(model_id) {
        return Err(format!("ID model tidak valid: '{model_id}'."));
    }
    find_model_file(models_dir, model_id)?.ok_or_else(|| {
        format!(
            "Model '{model_id}' belum terpasang. Impor file '{}' terlebih dahulu.",
            model_filename(model_id)
        )
    })
}

fn validate_model_source(path: &Path) -> Result<(File, String, String, u64), String> {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Nama file model tidak valid.".to_owned())?;
    if !path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("bin"))
    {
        return Err(format!("Ekstensi model harus .bin: '{filename}'."));
    }
    let id = model_id_from_filename(filename)
        .ok_or_else(|| format!("Nama model harus berpola ggml-<nama>.bin: '{filename}'."))?;
    let file = File::open(path).map_err(|error| {
        format!(
            "Tidak dapat membaca file model '{}': {error}",
            path.display()
        )
    })?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("Gagal memeriksa file model '{}': {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "Path model bukan file biasa: '{}'.",
            path.display()
        ));
    }
    if metadata.len() <= MIN_MODEL_SIZE_BYTES {
        return Err(format!(
            "Ukuran model terlalu kecil ({} byte); file harus lebih dari {} MB.",
            metadata.len(),
            MIN_MODEL_SIZE_BYTES / (1024 * 1024)
        ));
    }
    let mut magic = [0; 4];
    let mut reader = &file;
    reader
        .read_exact(&mut magic)
        .map_err(|error| format!("Header model tidak dapat dibaca: {error}"))?;
    if magic != WHISPER_MODEL_MAGIC {
        return Err(format!(
            "Header model tidak valid: magic harus 0x67676d6c (byte 'lmgg'), ditemukan {:02x?}.",
            magic
        ));
    }
    Ok((file, id, filename.to_owned(), metadata.len()))
}

struct PartialModelFile(PathBuf);

impl Drop for PartialModelFile {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub(crate) fn import_model_impl(source: &Path, models_dir: &Path) -> Result<ModelInfo, String> {
    let (mut source_file, id, _source_filename, _) = validate_model_source(source)?;
    fs::create_dir_all(models_dir).map_err(|error| {
        format!(
            "Gagal membuat folder model '{}': {error}",
            models_dir.display()
        )
    })?;

    let destination = models_dir.join(model_filename(&id));
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let partial = models_dir.join(format!(
        "{}-{}-{nonce}.part",
        destination
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("model"),
        std::process::id()
    ));
    let _partial_guard = PartialModelFile(partial.clone());
    let mut partial_file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&partial)
        .map_err(|error| format!("Gagal membuat file sementara model: {error}"))?;
    source_file
        .seek(SeekFrom::Start(0))
        .map_err(|error| format!("Gagal membaca ulang model sumber: {error}"))?;
    std::io::copy(&mut source_file, &mut partial_file)
        .map_err(|error| format!("Gagal menyalin model ke folder aplikasi: {error}"))?;
    partial_file
        .sync_all()
        .map_err(|error| format!("Gagal menyelesaikan salinan model: {error}"))?;
    drop(partial_file);
    fs::rename(&partial, &destination).map_err(|error| {
        format!(
            "Gagal memasang model '{}' secara atomik: {error}",
            destination.display()
        )
    })?;

    let display_name = MODEL_CATALOG
        .iter()
        .find(|(catalog_id, _, _)| *catalog_id == id)
        .map(|(_, name, _)| (*name).to_owned())
        .unwrap_or_else(|| id.clone());
    let estimate = MODEL_CATALOG
        .iter()
        .find(|(catalog_id, _, _)| *catalog_id == id)
        .map(|(_, _, estimate)| *estimate)
        .unwrap_or(0);
    let size_bytes = fs::metadata(&destination)
        .map_err(|error| format!("Model tersalin, tetapi ukurannya tidak dapat dibaca: {error}"))?
        .len();
    let filename = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| "Nama file model hasil impor tidak valid.".to_owned())?
        .to_owned();
    Ok(ModelInfo {
        id,
        name: display_name,
        filename,
        size_bytes: Some(size_bytes),
        estimated_size_bytes: estimate,
        installed: true,
    })
}

fn app_models_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let directory = app
        .path()
        .app_data_dir()
        .map_err(|error| format!("Gagal menentukan folder data aplikasi: {error}"))?
        .join("models");
    fs::create_dir_all(&directory).map_err(|error| {
        format!(
            "Gagal membuat folder model '{}': {error}",
            directory.display()
        )
    })?;
    Ok(directory)
}

/// Validasi string "WxH" (untuk dipakai CLI/UI bersama).
pub fn validate_size_impl(s: &str) -> Result<(u32, u32), String> {
    let (w, h) = s
        .split_once(['x', 'X'])
        .ok_or_else(|| format!("ukuran harus 'WxH', dapat '{s}'"))?;
    let width = w
        .parse::<u32>()
        .map_err(|_| format!("lebar tidak valid: {w}"))?;
    let height = h
        .parse::<u32>()
        .map_err(|_| format!("tinggi tidak valid: {h}"))?;
    if width == 0 || height == 0 {
        return Err("dimensi harus > 0".into());
    }
    Ok((width, height))
}

/// Daftar preset untuk UI.
pub fn list_presets_impl() -> Result<Vec<PresetSummary>, String> {
    let presets = caption_core::builtin_presets()?;
    Ok(presets
        .into_iter()
        .map(|p| PresetSummary {
            id: p.id,
            name: p.name,
            category: p.category.as_str().to_string(),
            words_per_line: p.words_per_line,
            font: p.font,
            colors: p.colors,
            outline: p.outline,
            shadow: p.shadow,
            margin_v: p.margin_v,
            margin_lr: p.margin_lr,
            glow: p.glow,
            emphasis: p.emphasis,
            emphasis_scale: p.emphasis_scale,
            animation: p.animation,
            box_: p.box_,
        })
        .collect())
}

/// Siapkan grup kata dan opsi render yang dipakai semua format ekspor.
fn render_groups_impl(
    req: &RenderRequest,
) -> Result<
    (
        Vec<caption_core::Group>,
        StylePreset,
        caption_core::VideoGeometry,
        caption_core::WordsPerLine,
    ),
    String,
> {
    let preset: StylePreset = caption_core::find_preset(&req.preset_id)?;
    let words: Vec<Word> = req.words.iter().cloned().map(Word::from).collect();
    let per_line = match req.words_per_line.clone() {
        Some(value) => value.into_words_per_line()?,
        None => preset.words_per_line_enum()?,
    };
    let groups = caption_core::group_words(&words, per_line);
    let geometry = caption_core::VideoGeometry {
        width: req.width,
        height: req.height,
    };
    Ok((groups, preset, geometry, per_line))
}

/// Render ASS dari preset + words.
pub fn render_ass_impl(req: &RenderRequest) -> Result<RenderResult, String> {
    let (groups, preset, geometry, per_line) = render_groups_impl(req)?;
    let opts = caption_core::PresetOptions {
        words_per_line: Some(per_line),
        caption_position: req.caption_position.map(TryInto::try_into).transpose()?,
    };
    let ass = caption_core::render_ass_with_preset(&groups, &preset, geometry, &opts);
    Ok(RenderResult { ass })
}

pub fn render_export_impl(req: &RenderRequest, format: &str) -> Result<String, String> {
    let normalized_format = format.to_ascii_lowercase();
    if !matches!(
        normalized_format.as_str(),
        "ass" | "srt" | "vtt" | "txt" | "json"
    ) {
        return Err(format!("Format ekspor tidak didukung: {format}"));
    }
    let caption_position = req.caption_position.map(TryInto::try_into).transpose()?;

    let (groups, preset, geometry, per_line) = render_groups_impl(req)?;
    match normalized_format.as_str() {
        "ass" => {
            let opts = caption_core::PresetOptions {
                words_per_line: Some(per_line),
                caption_position,
            };
            Ok(caption_core::render_ass_with_preset(
                &groups, &preset, geometry, &opts,
            ))
        }
        "srt" => Ok(caption_core::to_srt(&groups)),
        "vtt" => Ok(caption_core::to_vtt(&groups)),
        "txt" => Ok(caption_core::to_txt(&groups)),
        "json" => {
            caption_core::to_json(&groups).map_err(|error| format!("Gagal membuat JSON: {error}"))
        }
        _ => unreachable!("format sudah divalidasi"),
    }
}

// ---------- Command (tauri wrapper) ----------

#[tauri::command]
pub async fn list_models(app: tauri::AppHandle) -> Result<Vec<ModelInfo>, String> {
    let models_dir = app_models_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || list_models_impl(&models_dir))
        .await
        .map_err(|error| format!("Gagal membaca daftar model: {error}"))?
}

#[tauri::command]
pub async fn models_directory(app: tauri::AppHandle) -> Result<String, String> {
    Ok(app_models_dir(&app)?.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn import_model(app: tauri::AppHandle, path: String) -> Result<ModelInfo, String> {
    let models_dir = app_models_dir(&app)?;
    tauri::async_runtime::spawn_blocking(move || import_model_impl(Path::new(&path), &models_dir))
        .await
        .map_err(|error| format!("Gagal mengimpor model: {error}"))?
}

#[tauri::command]
pub async fn pick_input_file(path: String) -> Result<String, String> {
    let ext = std::path::Path::new(&path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase());
    let allowed = ["mp4", "mkv", "mov", "mp3", "wav", "m4a"];
    match ext {
        Some(e) if allowed.contains(&e.as_str()) => Ok(path),
        _ => Err(format!("format tidak didukung: {path}")),
    }
}

#[tauri::command]
pub async fn validate_size_cmd(size: String) -> Result<(u32, u32), String> {
    validate_size_impl(&size)
}

#[tauri::command]
pub async fn list_presets() -> Result<Vec<PresetSummary>, String> {
    list_presets_impl()
}

#[tauri::command]
pub async fn render_ass_from_words(req: RenderRequest) -> Result<RenderResult, String> {
    render_ass_impl(&req)
}

pub fn write_export_file_impl(path: &str, contents: &str) -> Result<String, String> {
    let file_path = std::path::Path::new(path);
    std::fs::write(file_path, contents)
        .map_err(|error| format!("Gagal menyimpan file '{}': {error}", file_path.display()))?;
    Ok(file_path.to_string_lossy().into_owned())
}

pub fn write_export_file_for_format_impl(
    path: &str,
    format: &str,
    req: &RenderRequest,
) -> Result<String, String> {
    let contents = render_export_impl(req, format)?;
    write_export_file_impl(path, &contents)
}

#[tauri::command]
pub async fn write_export_file(
    path: String,
    format: String,
    req: RenderRequest,
) -> Result<String, String> {
    write_export_file_for_format_impl(&path, &format, &req)
}

#[derive(Debug, Clone, Serialize)]
pub struct BurnProgress {
    pub progress: Option<u8>,
    pub indeterminate: bool,
    pub status: &'static str,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BurnQualityWire {
    Cepat,
    Bagus,
}

impl From<BurnQualityWire> for caption_core::Quality {
    fn from(value: BurnQualityWire) -> Self {
        match value {
            BurnQualityWire::Cepat => Self::Cepat,
            BurnQualityWire::Bagus => Self::Bagus,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct BurnRequest {
    pub input: String,
    pub words: Vec<WordDto>,
    pub preset_id: String,
    pub words_per_line: WordsPerLineWire,
    #[serde(default)]
    pub caption_position: Option<CaptionPositionWire>,
    pub output: String,
    pub quality: BurnQualityWire,
}

#[derive(Default, Clone)]
pub struct BurnManager {
    active: Arc<Mutex<Option<Arc<BurnJob>>>>,
}

#[derive(Default, Clone)]
pub struct TranscribeManager {
    active: Arc<Mutex<Option<Arc<TranscribeJob>>>>,
}

#[derive(Default)]
struct TranscribeControl {
    #[cfg(feature = "whisper")]
    child: Option<Child>,
    finished: bool,
}

#[derive(Default)]
pub(crate) struct TranscribeJob {
    cancel_requested: Arc<AtomicBool>,
    control: Mutex<TranscribeControl>,
    finished: Condvar,
}

#[cfg_attr(not(feature = "whisper"), allow(dead_code))]
pub(crate) struct TranscribeLease {
    manager: TranscribeManager,
    pub(crate) job: Arc<TranscribeJob>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscribeProgress {
    pub stage: String,
    pub progress: Option<u8>,
    pub status: &'static str,
    pub elapsed_ms: Option<u64>,
    pub audio_duration_ms: Option<u64>,
    pub threads: Option<i32>,
}

#[cfg(feature = "whisper")]
pub(crate) struct TranscribeResult {
    pub words: Vec<WordDto>,
    pub elapsed_ms: u64,
    pub audio_duration_ms: u64,
    pub threads: i32,
}

#[cfg(feature = "whisper")]
pub(crate) struct TranscribeProcess<'a> {
    pub input: &'a Path,
    pub model_path: &'a Path,
    pub ffmpeg: &'a Path,
    pub ffprobe: &'a Path,
    pub language: &'a str,
    pub threads: Option<i32>,
    pub prompt: Option<String>,
}

#[cfg(feature = "whisper")]
pub(crate) struct TemporaryWav(PathBuf);

#[cfg(feature = "whisper")]
impl TemporaryWav {
    pub(crate) fn new() -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        Self(std::env::temp_dir().join(format!(
            "caption-transcribe-{}-{nonce}.wav",
            std::process::id()
        )))
    }

    pub(crate) fn path(&self) -> &Path {
        &self.0
    }

    #[cfg(feature = "whisper")]
    fn cleanup(&self) -> Result<(), String> {
        match fs::remove_file(&self.0) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(format!(
                "Gagal menghapus WAV sementara '{}': {error}",
                self.0.display()
            )),
        }
    }
}

#[cfg(feature = "whisper")]
impl Drop for TemporaryWav {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.0) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!(
                    "Gagal menghapus WAV sementara '{}': {error}",
                    self.0.display()
                );
            }
        }
    }
}

#[derive(Default)]
struct BurnControl {
    cancel_requested: bool,
    child: Option<Child>,
    finished: bool,
}

#[derive(Default)]
pub(super) struct BurnJob {
    control: Mutex<BurnControl>,
    finished: Condvar,
}

pub(super) struct BurnProcess<'a> {
    pub(super) ffmpeg: &'a Path,
    pub(super) input: &'a Path,
    pub(super) output: &'a Path,
    pub(super) ass: &'a str,
    pub(super) info: &'a caption_core::VideoInfo,
    pub(super) quality: caption_core::Quality,
}

pub(super) struct BurnLease {
    manager: BurnManager,
    job: Arc<BurnJob>,
}

impl BurnManager {
    pub(super) fn reserve(&self) -> Result<BurnLease, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|error| format!("status burn terkunci: {error}"))?;
        if active.is_some() {
            return Err("Burn video lain sedang berjalan.".into());
        }
        let job = Arc::new(BurnJob::default());
        *active = Some(job.clone());
        Ok(BurnLease {
            manager: self.clone(),
            job,
        })
    }

    fn current(&self) -> Result<Arc<BurnJob>, String> {
        self.active
            .lock()
            .map_err(|error| format!("status burn terkunci: {error}"))?
            .clone()
            .ok_or_else(|| "Tidak ada burn video yang sedang berjalan.".into())
    }

    fn release(&self, job: &Arc<BurnJob>) {
        match self.active.lock() {
            Ok(mut active) => {
                if active
                    .as_ref()
                    .is_some_and(|active| Arc::ptr_eq(active, job))
                {
                    *active = None;
                }
            }
            Err(error) => eprintln!("Gagal melepas status burn video: {error}"),
        }
    }
}

#[cfg_attr(not(feature = "whisper"), allow(dead_code))]
impl TranscribeManager {
    pub(crate) fn reserve(&self) -> Result<TranscribeLease, String> {
        let mut active = self
            .active
            .lock()
            .map_err(|error| format!("Status transkripsi terkunci: {error}"))?;
        if active.is_some() {
            return Err("Transkripsi lain sedang berjalan.".into());
        }
        let job = Arc::new(TranscribeJob::default());
        *active = Some(job.clone());
        Ok(TranscribeLease {
            manager: self.clone(),
            job,
        })
    }

    fn current(&self) -> Result<Arc<TranscribeJob>, String> {
        self.active
            .lock()
            .map_err(|error| format!("Status transkripsi terkunci: {error}"))?
            .clone()
            .ok_or_else(|| "Tidak ada transkripsi yang sedang berjalan.".into())
    }

    fn release(&self, job: &Arc<TranscribeJob>) {
        match self.active.lock() {
            Ok(mut active) => {
                if active
                    .as_ref()
                    .is_some_and(|active| Arc::ptr_eq(active, job))
                {
                    *active = None;
                }
            }
            Err(error) => eprintln!("Gagal melepas status transkripsi: {error}"),
        }
    }
}

impl TranscribeJob {
    #[cfg(feature = "whisper")]
    pub(crate) fn is_cancel_requested(&self) -> bool {
        self.cancel_requested.load(Ordering::Acquire)
    }

    #[cfg(feature = "whisper")]
    fn install_child(&self, mut child: Child) -> Result<(), String> {
        let mut control = match self.control.lock() {
            Ok(control) => control,
            Err(error) => {
                let kill = child.kill();
                let wait = child.wait();
                return Err(format!(
                    "Status proses ekstraksi terkunci: {error}; kill={kill:?}; wait={wait:?}"
                ));
            }
        };
        if self.is_cancel_requested() {
            let _ = child.kill();
            let _ = child.wait();
            return Err("Transkripsi dibatalkan.".into());
        }
        control.child = Some(child);
        Ok(())
    }

    #[cfg(feature = "whisper")]
    fn try_wait(&self) -> Result<Option<ExitStatus>, String> {
        let mut control = self
            .control
            .lock()
            .map_err(|error| format!("Status proses ekstraksi terkunci: {error}"))?;
        control
            .child
            .as_mut()
            .ok_or_else(|| "Proses ekstraksi FFmpeg tidak tersedia.".to_string())?
            .try_wait()
            .map_err(|error| format!("Gagal memeriksa proses ekstraksi FFmpeg: {error}"))
    }

    #[cfg(feature = "whisper")]
    fn take_child(&self) -> Result<Option<Child>, String> {
        self.control
            .lock()
            .map(|mut control| control.child.take())
            .map_err(|error| format!("Status proses ekstraksi terkunci: {error}"))
    }

    fn request_cancel(&self) -> Result<(), String> {
        let control = self
            .control
            .lock()
            .map_err(|error| format!("Status proses transkripsi terkunci: {error}"))?;
        if control.finished {
            return Err("Transkripsi sudah selesai.".into());
        }
        self.cancel_requested.store(true, Ordering::Release);
        #[cfg(feature = "whisper")]
        let mut control = control;
        #[cfg(feature = "whisper")]
        if let Some(child) = control.child.as_mut() {
            if let Err(kill_error) = child.kill() {
                if child
                    .try_wait()
                    .map_err(|error| format!("Gagal memeriksa FFmpeg setelah kill: {error}"))?
                    .is_none()
                {
                    return Err(format!("Gagal menghentikan FFmpeg: {kill_error}"));
                }
            }
        }
        Ok(())
    }

    fn wait_until_finished(&self) -> Result<(), String> {
        let mut control = self
            .control
            .lock()
            .map_err(|error| format!("Status proses transkripsi terkunci: {error}"))?;
        while !control.finished {
            control = self
                .finished
                .wait(control)
                .map_err(|error| format!("Status proses transkripsi terkunci: {error}"))?;
        }
        Ok(())
    }

    #[cfg_attr(not(feature = "whisper"), allow(dead_code))]
    fn mark_finished(&self) {
        match self.control.lock() {
            Ok(mut control) => {
                #[cfg(feature = "whisper")]
                if let Some(mut child) = control.child.take() {
                    if child.try_wait().ok().flatten().is_none() {
                        if let Err(error) = child.kill() {
                            eprintln!("Gagal menghentikan FFmpeg saat cleanup: {error}");
                        }
                        if let Err(error) = child.wait() {
                            eprintln!("Gagal menunggu FFmpeg saat cleanup: {error}");
                        }
                    }
                }
                control.finished = true;
                self.finished.notify_all();
            }
            Err(error) => eprintln!("Gagal menandai transkripsi selesai: {error}"),
        }
    }
}

impl Drop for TranscribeLease {
    fn drop(&mut self) {
        self.job.mark_finished();
        self.manager.release(&self.job);
    }
}

impl BurnJob {
    fn is_cancel_requested(&self) -> Result<bool, String> {
        self.control
            .lock()
            .map(|control| control.cancel_requested)
            .map_err(|error| format!("status proses burn terkunci: {error}"))
    }

    fn install_child(&self, mut child: Child) -> Result<(), String> {
        let mut control = match self.control.lock() {
            Ok(control) => control,
            Err(error) => {
                let kill = child.kill();
                let wait = child.wait();
                return Err(format!(
                    "status proses burn terkunci: {error}; kill={kill:?}; wait={wait:?}"
                ));
            }
        };
        if control.cancel_requested {
            let kill = child.kill();
            let wait = child.wait();
            let mut message = "Burn video dibatalkan.".to_string();
            if let Err(error) = kill {
                message.push_str(&format!(" Gagal menghentikan FFmpeg: {error}."));
            }
            if let Err(error) = wait {
                message.push_str(&format!(" Gagal menunggu FFmpeg berhenti: {error}."));
            }
            return Err(message);
        }
        control.child = Some(child);
        Ok(())
    }

    fn try_wait(&self) -> Result<Option<ExitStatus>, String> {
        let mut control = self
            .control
            .lock()
            .map_err(|error| format!("status proses burn terkunci: {error}"))?;
        control
            .child
            .as_mut()
            .ok_or_else(|| "Proses FFmpeg burn tidak tersedia.".to_string())?
            .try_wait()
            .map_err(|error| format!("gagal memeriksa proses FFmpeg: {error}"))
    }

    fn kill(&self) -> Result<(), String> {
        let mut control = self
            .control
            .lock()
            .map_err(|error| format!("status proses burn terkunci: {error}"))?;
        if let Some(child) = control.child.as_mut() {
            if let Err(kill_error) = child.kill() {
                if child
                    .try_wait()
                    .map_err(|error| format!("gagal memeriksa FFmpeg setelah kill: {error}"))?
                    .is_none()
                {
                    return Err(format!("gagal menghentikan FFmpeg: {kill_error}"));
                }
            }
        }
        Ok(())
    }

    fn take_child(&self) -> Result<Option<Child>, String> {
        self.control
            .lock()
            .map(|mut control| control.child.take())
            .map_err(|error| format!("status proses burn terkunci: {error}"))
    }

    fn stop_and_reap(&self) -> Result<(), String> {
        let mut control = self
            .control
            .lock()
            .map_err(|error| format!("status proses burn terkunci: {error}"))?;
        if let Some(mut child) = control.child.take() {
            if child
                .try_wait()
                .map_err(|error| format!("gagal memeriksa FFmpeg saat cleanup: {error}"))?
                .is_none()
            {
                if let Err(kill_error) = child.kill() {
                    if child
                        .try_wait()
                        .map_err(|error| format!("gagal memeriksa FFmpeg setelah kill: {error}"))?
                        .is_none()
                    {
                        return Err(format!(
                            "gagal menghentikan FFmpeg saat cleanup: {kill_error}"
                        ));
                    }
                }
                child
                    .wait()
                    .map_err(|error| format!("gagal menunggu FFmpeg berhenti: {error}"))?;
            }
        }
        Ok(())
    }

    fn request_cancel(&self) -> Result<(), String> {
        let mut control = self
            .control
            .lock()
            .map_err(|error| format!("status proses burn terkunci: {error}"))?;
        if control.finished {
            return Err("Burn video sudah selesai.".into());
        }
        control.cancel_requested = true;
        if let Some(child) = control.child.as_mut() {
            if let Err(kill_error) = child.kill() {
                if child
                    .try_wait()
                    .map_err(|error| format!("gagal memeriksa FFmpeg setelah kill: {error}"))?
                    .is_none()
                {
                    return Err(format!("gagal menghentikan FFmpeg: {kill_error}"));
                }
            }
        }
        Ok(())
    }

    fn wait_until_finished(&self) -> Result<(), String> {
        let mut control = self
            .control
            .lock()
            .map_err(|error| format!("status proses burn terkunci: {error}"))?;
        while !control.finished {
            control = self
                .finished
                .wait(control)
                .map_err(|error| format!("status proses burn terkunci: {error}"))?;
        }
        Ok(())
    }

    fn mark_finished(&self) {
        match self.control.lock() {
            Ok(mut control) => {
                if let Some(child) = control.child.as_mut() {
                    match child.try_wait() {
                        Ok(Some(_)) => {}
                        Ok(None) => {
                            if let Err(error) = child.kill() {
                                eprintln!("Gagal menghentikan FFmpeg saat cleanup: {error}");
                            }
                            if let Err(error) = child.wait() {
                                eprintln!("Gagal menunggu FFmpeg saat cleanup: {error}");
                            }
                        }
                        Err(error) => eprintln!("Gagal memeriksa FFmpeg saat cleanup: {error}"),
                    }
                }
                control.child = None;
                control.finished = true;
                self.finished.notify_all();
            }
            Err(error) => eprintln!("Gagal menandai burn selesai: {error}"),
        }
    }
}

impl Drop for BurnLease {
    fn drop(&mut self) {
        self.job.mark_finished();
        self.manager.release(&self.job);
    }
}

pub fn validate_mp4_output(path: &Path) -> Result<(), String> {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some(extension) if extension.eq_ignore_ascii_case("mp4") => Ok(()),
        _ => Err(format!(
            "File output burn harus berekstensi .mp4: {}",
            path.display()
        )),
    }
}

pub(super) fn configure_no_window(command: &mut Command) {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = command;
}

pub(super) fn probe_video(input: &Path, ffmpeg: &Path) -> Result<caption_core::VideoInfo, String> {
    let ffprobe = caption_core::default_ffprobe_for(ffmpeg);
    probe_video_with_ffprobe(input, &ffprobe)
}

pub(super) fn probe_video_with_ffprobe(
    input: &Path,
    ffprobe: &Path,
) -> Result<caption_core::VideoInfo, String> {
    if !ffprobe.is_file() {
        return Err(format!(
            "Sidecar ffprobe tidak ditemukan: {}",
            ffprobe.display()
        ));
    }
    let mut command = Command::new(ffprobe);
    configure_no_window(&mut command);
    let result = command
        .stdin(Stdio::null())
        .args([
            "-v",
            "error",
            "-print_format",
            "json",
            "-show_streams",
            "-show_format",
        ])
        .arg(input)
        .output()
        .map_err(|error| format!("Gagal menjalankan ffprobe: {error}"))?;
    if !result.status.success() {
        return Err(format!(
            "ffprobe gagal ({}): {}",
            result.status,
            String::from_utf8_lossy(&result.stderr).trim()
        ));
    }
    caption_core::parse_ffprobe_json(&String::from_utf8_lossy(&result.stdout))
}

pub(super) fn burn_progress(
    duration_s: f64,
    update: Option<caption_core::ProgressUpdate>,
    status: &'static str,
) -> BurnProgress {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return BurnProgress {
            progress: None,
            indeterminate: status == "running",
            status,
        };
    }
    let progress = update
        .and_then(|update| update.out_time_us)
        .map(|time| caption_core::percent(time, duration_s))
        .or_else(|| update.filter(|update| update.finished).map(|_| 100));
    BurnProgress {
        progress: progress.or_else(|| (status == "running").then_some(0)),
        indeterminate: false,
        status,
    }
}

fn read_stderr_tail(stderr: impl std::io::Read) -> Result<VecDeque<String>, String> {
    const STDERR_TAIL_LINES: usize = 12;
    let mut lines = VecDeque::with_capacity(STDERR_TAIL_LINES);
    let mut reader = BufReader::new(stderr);
    let mut line = Vec::new();
    loop {
        line.clear();
        let count = reader
            .read_until(b'\n', &mut line)
            .map_err(|error| format!("gagal membaca stderr FFmpeg: {error}"))?;
        if count == 0 {
            break;
        }
        while matches!(line.last(), Some(b'\n' | b'\r')) {
            line.pop();
        }
        if lines.len() == STDERR_TAIL_LINES {
            lines.pop_front();
        }
        lines.push_back(String::from_utf8_lossy(&line).into_owned());
    }
    Ok(lines)
}

fn remove_partial_output(output: &Path) -> Result<(), String> {
    match std::fs::remove_file(output) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Gagal menghapus output parsial '{}': {error}",
            output.display()
        )),
    }
}

pub(super) fn run_ffmpeg_burn<F>(
    process: BurnProcess<'_>,
    job: &Arc<BurnJob>,
    mut emit: F,
) -> Result<String, String>
where
    F: FnMut(BurnProgress) -> Result<(), String>,
{
    let BurnProcess {
        ffmpeg,
        input,
        output,
        ass,
        info,
        quality,
    } = process;
    validate_mp4_output(output)?;
    let mut work = caption_core::WorkDirGuard::new("caption-burn")?;
    let mut output_may_be_partial = false;
    let result = (|| {
        emit(burn_progress(info.duration_s, None, "running"))?;
        caption_core::setup_work_dir(work.path(), ass)?;
        if job.is_cancel_requested()? {
            return Err("Burn video dibatalkan.".into());
        }
        let args = caption_core::build_burn_args(input, work.path(), output, quality, info);
        let mut command = Command::new(ffmpeg);
        configure_no_window(&mut command);
        let mut child = command
            .args(&args)
            .current_dir(work.path())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| format!("Gagal menjalankan FFmpeg: {error}"))?;
        output_may_be_partial = true;
        let stdout = match child.stdout.take() {
            Some(stdout) => stdout,
            None => {
                let kill = child.kill();
                let wait = child.wait();
                return Err(format!(
                    "stdout FFmpeg tidak tersedia; kill={kill:?}; wait={wait:?}"
                ));
            }
        };
        let stderr = match child.stderr.take() {
            Some(stderr) => stderr,
            None => {
                let kill = child.kill();
                let wait = child.wait();
                return Err(format!(
                    "stderr FFmpeg tidak tersedia; kill={kill:?}; wait={wait:?}"
                ));
            }
        };
        job.install_child(child)?;

        let (sender, receiver) = mpsc::channel::<Result<caption_core::ProgressUpdate, String>>();
        let progress_reader = thread::spawn(move || -> Result<(), String> {
            for line in BufReader::new(stdout).lines() {
                let line = match line {
                    Ok(line) => line,
                    Err(error) => {
                        let _ = sender.send(Err(format!("Gagal membaca progress FFmpeg: {error}")));
                        return Ok(());
                    }
                };
                if let Some(update) = caption_core::parse_progress_line(&line) {
                    if sender.send(Ok(update)).is_err() {
                        break;
                    }
                }
            }
            Ok(())
        });
        let stderr_reader = thread::spawn(move || read_stderr_tail(stderr));

        let mut exit_status = None;
        let mut progress_closed = false;
        let mut progress_error = None;
        while exit_status.is_none() || !progress_closed {
            if job.is_cancel_requested()? {
                job.kill()?;
            }
            if !progress_closed {
                match receiver.recv_timeout(Duration::from_millis(100)) {
                    Ok(Ok(update)) => {
                        if let Err(error) =
                            emit(burn_progress(info.duration_s, Some(update), "running"))
                        {
                            progress_error = Some(error);
                            job.kill()?;
                        }
                    }
                    Ok(Err(error)) => {
                        progress_error = Some(error);
                        job.kill()?;
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => progress_closed = true,
                    Err(mpsc::RecvTimeoutError::Timeout) => {}
                }
            } else if exit_status.is_none() {
                thread::sleep(Duration::from_millis(100));
            }
            if exit_status.is_none() {
                exit_status = job.try_wait()?;
            }
        }
        let reader_result = progress_reader
            .join()
            .map_err(|_| "Thread pembaca progress FFmpeg berhenti tidak normal.".to_string())?;
        let stderr_lines = stderr_reader
            .join()
            .map_err(|_| "Thread pembaca stderr FFmpeg berhenti tidak normal.".to_string())??;
        let _child = job.take_child()?;
        reader_result?;
        if let Some(error) = progress_error {
            return Err(format!("Gagal mengirim progress burn: {error}"));
        }
        if job.is_cancel_requested()? {
            return Err("Burn video dibatalkan.".into());
        }
        let status =
            exit_status.ok_or_else(|| "Status akhir FFmpeg tidak tersedia.".to_string())?;
        if !status.success() {
            let tail = stderr_lines.into_iter().collect::<Vec<_>>().join("\n");
            let detail = if tail.is_empty() {
                String::new()
            } else {
                format!("\nLog terakhir FFmpeg:\n{tail}")
            };
            return Err(format!("FFmpeg burn-in gagal ({status}).{detail}"));
        }
        Ok(())
    })();

    let mut errors = Vec::new();
    if let Err(error) = result {
        errors.push(error);
        if let Err(error) = job.stop_and_reap() {
            errors.push(error);
        }
        if output_may_be_partial {
            if let Err(error) = remove_partial_output(output) {
                errors.push(error);
            }
        }
    }
    if let Err(error) = work.cleanup() {
        errors.push(error);
    }
    if !errors.is_empty() {
        let status = match job.is_cancel_requested() {
            Ok(true) => "cancelled",
            Ok(false) => "failed",
            Err(error) => {
                errors.push(error);
                "failed"
            }
        };
        if let Err(error) = emit(BurnProgress {
            progress: None,
            indeterminate: false,
            status,
        }) {
            errors.push(format!("Gagal mengirim status akhir burn: {error}"));
        }
        return Err(errors.join("\n"));
    }
    emit(BurnProgress {
        progress: Some(100),
        indeterminate: false,
        status: "completed",
    })
    .map_err(|error| {
        format!(
            "Burn selesai dan file tersimpan di '{}', tetapi event progress gagal: {error}",
            output.display()
        )
    })?;
    Ok(output.to_string_lossy().into_owned())
}

fn burn_video_impl(
    app: tauri::AppHandle,
    request: BurnRequest,
    job: Arc<BurnJob>,
) -> Result<String, String> {
    let input = PathBuf::from(&request.input);
    let output = PathBuf::from(&request.output);
    validate_mp4_output(&output)?;
    let ffmpeg = sidecar_path("ffmpeg");
    if !ffmpeg.is_absolute() || !ffmpeg.is_file() {
        return Err(format!(
            "Sidecar FFmpeg absolut tidak ditemukan: {}",
            ffmpeg.display()
        ));
    }
    let info = probe_video(&input, &ffmpeg)?;
    if job.is_cancel_requested()? {
        return Err("Burn video dibatalkan.".into());
    }
    let words_per_line = request.words_per_line.into_words_per_line()?;
    let render_request = RenderRequest {
        words: request.words,
        preset_id: request.preset_id,
        width: info.width,
        height: info.height,
        words_per_line: Some(WordsPerLineWire::from(words_per_line)),
        caption_position: request.caption_position,
    };
    let ass = render_ass_impl(&render_request)?.ass;
    run_ffmpeg_burn(
        BurnProcess {
            ffmpeg: &ffmpeg,
            input: &input,
            output: &output,
            ass: &ass,
            info: &info,
            quality: request.quality.into(),
        },
        &job,
        move |progress| {
            app.emit("burn-progress", progress)
                .map_err(|error| format!("Gagal mengirim event burn-progress: {error}"))
        },
    )
}

impl From<caption_core::WordsPerLine> for WordsPerLineWire {
    fn from(value: caption_core::WordsPerLine) -> Self {
        match value {
            caption_core::WordsPerLine::One => Self::Number(1),
            caption_core::WordsPerLine::Two => Self::Number(2),
            caption_core::WordsPerLine::Three => Self::Number(3),
            caption_core::WordsPerLine::Sentence => Self::Sentence(SentenceWire::Sentence),
        }
    }
}

#[tauri::command]
pub async fn burn_video(
    app: tauri::AppHandle,
    manager: tauri::State<'_, BurnManager>,
    req: BurnRequest,
) -> Result<String, String> {
    validate_mp4_output(Path::new(&req.output))?;
    let lease = manager.reserve()?;
    let job = lease.job.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let _lease = lease;
        burn_video_impl(app, req, job)
    })
    .await
    .map_err(|error| format!("Task burn video gagal: {error}"))?
}

#[tauri::command]
pub async fn cancel_burn(manager: tauri::State<'_, BurnManager>) -> Result<(), String> {
    let job = manager.current()?;
    job.request_cancel()?;
    tauri::async_runtime::spawn_blocking(move || job.wait_until_finished())
        .await
        .map_err(|error| format!("Task pembatalan burn gagal: {error}"))?
}

// ---------- Command transcribe (butuh feature `whisper`) ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscribeRequest {
    pub input: Option<String>,
    pub model: Option<String>,
    pub language: Option<String>,
    pub threads: Option<i32>,
    pub prompt: Option<String>,
    pub words_per_line: Option<WordsPerLineWire>,
}

#[cfg(feature = "whisper")]
fn transcribe_progress(
    stage: &str,
    progress: Option<u8>,
    status: &'static str,
) -> TranscribeProgress {
    TranscribeProgress {
        stage: stage.to_owned(),
        progress,
        status,
        elapsed_ms: None,
        audio_duration_ms: None,
        threads: None,
    }
}

#[cfg(feature = "whisper")]
fn emit_transcribe_progress(
    app: &tauri::AppHandle,
    progress: TranscribeProgress,
) -> Result<(), String> {
    app.emit("transcribe_progress", progress)
        .map_err(|error| format!("Gagal mengirim event transcribe_progress: {error}"))
}

#[cfg_attr(not(feature = "whisper"), allow(dead_code))]
pub(crate) fn parse_audio_probe_info(json: &str) -> Result<(bool, Option<f64>), String> {
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|error| format!("JSON ffprobe tidak valid: {error}"))?;
    let has_audio = value
        .get("streams")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|streams| {
            streams.iter().any(|stream| {
                stream.get("codec_type").and_then(serde_json::Value::as_str) == Some("audio")
            })
        });
    let duration = value
        .get("format")
        .and_then(|format| format.get("duration"))
        .and_then(|duration| duration.as_str())
        .and_then(|duration| duration.parse::<f64>().ok())
        .filter(|duration| duration.is_finite() && *duration > 0.0);
    Ok((has_audio, duration))
}

#[cfg(feature = "whisper")]
fn probe_audio_stream(input: &Path, ffprobe: &Path) -> Result<f64, String> {
    if !ffprobe.is_file() {
        return Err(format!(
            "Sidecar ffprobe tidak ditemukan: {}",
            ffprobe.display()
        ));
    }
    let mut command = Command::new(ffprobe);
    configure_no_window(&mut command);
    let output = command
        .stdin(Stdio::null())
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type:format=duration",
            "-of",
            "json",
        ])
        .arg(input)
        .output()
        .map_err(|error| format!("Gagal menjalankan ffprobe: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "ffprobe gagal ({}): {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let (has_audio, duration) = parse_audio_probe_info(&String::from_utf8_lossy(&output.stdout))?;
    if !has_audio {
        return Err("Video tidak memiliki audio.".into());
    }
    Ok(duration.unwrap_or(0.0))
}

#[cfg_attr(not(feature = "whisper"), allow(dead_code))]
pub(crate) fn extraction_progress(
    duration_s: f64,
    update: Option<caption_core::ProgressUpdate>,
) -> Option<u8> {
    if !duration_s.is_finite() || duration_s <= 0.0 {
        return None;
    }
    let time = update.and_then(|update| update.out_time_us)?;
    let percent = caption_core::percent(time, duration_s).min(100);
    Some(((u16::from(percent) * 10) / 100) as u8)
}

#[cfg(feature = "whisper")]
fn run_audio_extraction<F>(
    ffmpeg: &Path,
    input: &Path,
    output: &Path,
    duration_s: f64,
    job: &Arc<TranscribeJob>,
    mut emit: F,
) -> Result<(), String>
where
    F: FnMut(TranscribeProgress) -> Result<(), String>,
{
    if job.is_cancel_requested() {
        return Err("Transkripsi dibatalkan.".into());
    }
    let mut command = Command::new(ffmpeg);
    configure_no_window(&mut command);
    let mut child = command
        .args(["-y", "-i"])
        .arg(input)
        .args([
            "-vn",
            "-ac",
            "1",
            "-ar",
            "16000",
            "-c:a",
            "pcm_s16le",
            "-progress",
            "pipe:1",
            "-nostats",
        ])
        .arg(output)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Gagal menjalankan FFmpeg untuk ekstraksi audio: {error}"))?;
    let stdout = match child.stdout.take() {
        Some(stdout) => stdout,
        None => {
            let kill = child.kill();
            let wait = child.wait();
            return Err(format!(
                "stdout FFmpeg ekstraksi tidak tersedia; kill={kill:?}; wait={wait:?}"
            ));
        }
    };
    let stderr = match child.stderr.take() {
        Some(stderr) => stderr,
        None => {
            let kill = child.kill();
            let wait = child.wait();
            return Err(format!(
                "stderr FFmpeg ekstraksi tidak tersedia; kill={kill:?}; wait={wait:?}"
            ));
        }
    };

    let (sender, receiver) = mpsc::channel::<Result<String, String>>();
    let progress_reader = thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            match line {
                Ok(line) => {
                    if sender.send(Ok(line)).is_err() {
                        break;
                    }
                }
                Err(error) => {
                    let _ = sender.send(Err(format!("Gagal membaca progress ekstraksi: {error}")));
                    break;
                }
            }
        }
    });
    let stderr_reader = thread::spawn(move || read_stderr_tail(stderr));
    if let Err(error) = job.install_child(child) {
        let _ = progress_reader.join();
        let _ = stderr_reader.join();
        return Err(error);
    }

    let mut exit_status = None;
    let mut progress_closed = false;
    let mut progress_error = None;
    while exit_status.is_none() || !progress_closed {
        if job.is_cancel_requested() {
            job.request_cancel()?;
        }
        if !progress_closed {
            match receiver.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(line)) => {
                    if let Some(update) = caption_core::parse_progress_line(&line) {
                        if let Err(error) = emit(transcribe_progress(
                            "Mengekstrak audio",
                            extraction_progress(duration_s, Some(update)),
                            "running",
                        )) {
                            progress_error = Some(error);
                            job.request_cancel()?;
                        }
                    }
                }
                Ok(Err(error)) => {
                    progress_error = Some(error);
                    job.request_cancel()?;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => progress_closed = true,
                Err(mpsc::RecvTimeoutError::Timeout) => {}
            }
        } else if exit_status.is_none() {
            thread::sleep(Duration::from_millis(100));
        }
        if exit_status.is_none() {
            exit_status = job.try_wait()?;
        }
    }
    progress_reader
        .join()
        .map_err(|_| "Thread pembaca progress ekstraksi berhenti tidak normal.".to_string())?;
    let stderr_lines = stderr_reader
        .join()
        .map_err(|_| "Thread pembaca stderr ekstraksi berhenti tidak normal.".to_string())??;
    let _child = job.take_child()?;
    if let Some(error) = progress_error {
        return Err(error);
    }
    if job.is_cancel_requested() {
        return Err("Transkripsi dibatalkan.".into());
    }
    let status =
        exit_status.ok_or_else(|| "Status akhir FFmpeg ekstraksi tidak tersedia.".to_string())?;
    if !status.success() {
        let tail = stderr_lines.into_iter().collect::<Vec<_>>().join("\n");
        return Err(format!("Ekstraksi audio FFmpeg gagal ({status}): {tail}"));
    }
    emit(transcribe_progress(
        "Mengekstrak audio",
        Some(10),
        "running",
    ))
}

#[cfg(feature = "whisper")]
pub(crate) fn run_transcription<F>(
    process: TranscribeProcess<'_>,
    job: Arc<TranscribeJob>,
    mut emit: F,
) -> Result<TranscribeResult, String>
where
    F: FnMut(TranscribeProgress) -> Result<(), String> + Send + 'static,
{
    let TranscribeProcess {
        input,
        model_path,
        ffmpeg,
        ffprobe,
        language,
        threads,
        prompt,
    } = process;
    let started = Instant::now();
    let audio_duration_s = probe_audio_stream(input, ffprobe)?;
    if job.is_cancel_requested() {
        return Err("Transkripsi dibatalkan.".into());
    }
    let wav = TemporaryWav::new();
    emit(transcribe_progress("Mengekstrak audio", Some(0), "running"))?;
    run_audio_extraction(ffmpeg, input, &wav.0, audio_duration_s, &job, &mut emit)?;

    let samples = match caption_core::load_wav_16k_mono(wav.path()) {
        Ok(samples) => samples,
        Err(error) => {
            let cleanup_error = wav.cleanup().err();
            return Err(match cleanup_error {
                Some(cleanup_error) => {
                    format!("Gagal memuat WAV: {error}; {cleanup_error}")
                }
                None => format!("Gagal memuat WAV: {error}"),
            });
        }
    };
    wav.cleanup()?;
    let audio_duration_ms = (samples.len() as f64 / 16.0).round() as u64;
    if job.is_cancel_requested() {
        return Err("Transkripsi dibatalkan.".into());
    }
    emit(transcribe_progress("Memuat model", Some(10), "running"))?;

    let mut options = caption_core::TranscribeOptions::new(model_path.to_owned());
    options.language = language.to_owned();
    if let Some(threads) = threads {
        options.threads = threads;
    }
    options.initial_prompt = prompt;
    let thread_count = options.threads;
    let progress_emit = Arc::new(Mutex::new(emit));
    let progress_error = Arc::new(Mutex::new(None));
    let progress_emit_callback = progress_emit.clone();
    let callback_error = progress_error.clone();
    options.on_progress = Some(Arc::new(move |progress| {
        let percent = progress.clamp(0, 100) as u8;
        let overall = 10 + ((u16::from(percent) * 90) / 100) as u8;
        if let Ok(mut emit) = progress_emit_callback.lock() {
            if let Err(error) = emit(transcribe_progress(
                "Mentranskripsi",
                Some(overall),
                "running",
            )) {
                eprintln!("{error}");
                if let Ok(mut callback_error) = callback_error.lock() {
                    *callback_error = Some(error);
                }
            }
        }
    }));
    options.on_abort = Some(job.cancel_requested.clone());

    let tokens = match caption_core::transcribe(&samples, &options) {
        Ok(tokens) => tokens,
        Err(_) if job.is_cancel_requested() => return Err("Transkripsi dibatalkan.".into()),
        Err(error) => {
            return Err(format!(
                "Model gagal dibuka. Mungkin rusak, impor ulang model. Detail: {error}"
            ))
        }
    };
    if job.is_cancel_requested() {
        return Err("Transkripsi dibatalkan.".into());
    }
    if let Some(error) = progress_error
        .lock()
        .map_err(|error| format!("Status error progress terkunci: {error}"))?
        .take()
    {
        return Err(error);
    }
    let words = caption_core::merge_tokens_to_words(&tokens)
        .iter()
        .map(WordDto::from)
        .collect();
    let elapsed_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
    let rtf = elapsed_ms as f64 / (audio_duration_ms.max(1) as f64);
    eprintln!(
        "audio={:.3}s transcribe_elapsed={:.3}s rtf={rtf:.3} threads={thread_count}",
        audio_duration_ms as f64 / 1000.0,
        elapsed_ms as f64 / 1000.0
    );
    let mut emit = progress_emit
        .lock()
        .map_err(|error| format!("Status progress transkripsi terkunci: {error}"))?;
    emit(transcribe_progress("Mentranskripsi", Some(100), "running"))?;
    Ok(TranscribeResult {
        words,
        elapsed_ms,
        audio_duration_ms,
        threads: thread_count,
    })
}

#[cfg(feature = "whisper")]
#[tauri::command]
pub async fn transcribe(
    app: tauri::AppHandle,
    manager: tauri::State<'_, TranscribeManager>,
    request: TranscribeRequest,
) -> Result<Vec<WordDto>, String> {
    let input = request.input.ok_or("input wajib")?;
    let model_id = request.model.ok_or("model wajib")?;
    let models_dir = app_models_dir(&app)?;
    let model_path = resolve_model_path(&models_dir, &model_id)?;
    let lease = manager.reserve()?;
    let job = lease.job.clone();
    let started = Instant::now();
    tauri::async_runtime::spawn_blocking(move || {
        let _lease = lease;
        let ffmpeg = sidecar_path("ffmpeg");
        let ffprobe = sidecar_path("ffprobe");
        let emit_app = app.clone();
        let emit = move |progress| emit_transcribe_progress(&emit_app, progress);
        let result = run_transcription(
            TranscribeProcess {
                input: Path::new(&input),
                model_path: &model_path,
                ffmpeg: &ffmpeg,
                ffprobe: &ffprobe,
                language: request.language.as_deref().unwrap_or("id"),
                threads: request.threads,
                prompt: request.prompt,
            },
            job.clone(),
            emit,
        );
        match result {
            Ok(result) => {
                let mut completed = transcribe_progress("Selesai", Some(100), "completed");
                completed.elapsed_ms = Some(result.elapsed_ms);
                completed.audio_duration_ms = Some(result.audio_duration_ms);
                completed.threads = Some(result.threads);
                emit_transcribe_progress(&app, completed)?;
                Ok(result.words)
            }
            Err(error) => {
                let status = if job.is_cancel_requested() || error.contains("dibatalkan") {
                    "cancelled"
                } else {
                    "failed"
                };
                let stage = if status == "cancelled" {
                    "Dibatalkan"
                } else {
                    "Gagal"
                };
                let mut terminal = transcribe_progress(stage, None, status);
                terminal.elapsed_ms =
                    Some(started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64);
                emit_transcribe_progress(&app, terminal)?;
                Err(error)
            }
        }
    })
    .await
    .map_err(|error| format!("Task transkripsi gagal: {error}"))?
}

#[cfg(not(feature = "whisper"))]
#[tauri::command]
pub async fn transcribe(
    _app: tauri::AppHandle,
    _manager: tauri::State<'_, TranscribeManager>,
    _request: TranscribeRequest,
) -> Result<Vec<WordDto>, String> {
    Err("Fitur Whisper tidak aktif pada build ini.".into())
}

#[tauri::command]
pub async fn cancel_transcribe(manager: tauri::State<'_, TranscribeManager>) -> Result<(), String> {
    let job = manager.current()?;
    job.request_cancel()?;
    tauri::async_runtime::spawn_blocking(move || job.wait_until_finished())
        .await
        .map_err(|error| format!("Task pembatalan transkripsi gagal: {error}"))?
}

// ---------- Lokasi sidecar FFmpeg/ffprobe ----------

/// Tauri menaruh sidecar (`ffmpeg.exe`, `ffprobe.exe`) di folder yang sama dengan
/// exe aplikasi. Fungsi ini memakai berkas itu bila ada, dan jatuh ke nama polos
/// (dicari lewat PATH) bila tidak ada.
pub fn sidecar_path(name: &str) -> std::path::PathBuf {
    let exe = std::env::current_exe().ok();
    sidecar_path_in(exe.as_deref().and_then(|p| p.parent()), name)
}

/// Versi yang bisa dites: folder exe diberikan dari luar.
pub fn sidecar_path_in(exe_dir: Option<&std::path::Path>, name: &str) -> std::path::PathBuf {
    let file = if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    };
    if let Some(dir) = exe_dir {
        let candidate = dir.join(&file);
        if candidate.is_file() {
            return candidate;
        }
    }
    std::path::PathBuf::from(name)
}

#[cfg(test)]
mod sidecar_tests {
    use super::{sidecar_path, sidecar_path_in};
    use std::path::{Path, PathBuf};

    fn unique_dir(tag: &str) -> PathBuf {
        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "caption-sidecar-{tag}-{}-{nonce}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn exe_name(name: &str) -> String {
        if cfg!(windows) {
            format!("{name}.exe")
        } else {
            name.to_string()
        }
    }

    #[test]
    fn prefers_file_next_to_exe() {
        let dir = unique_dir("ada");
        let file = dir.join(exe_name("ffmpeg"));
        std::fs::write(&file, b"x").unwrap();
        assert_eq!(sidecar_path_in(Some(dir.as_path()), "ffmpeg"), file);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn falls_back_to_plain_name_when_missing() {
        let dir = unique_dir("kosong");
        assert_eq!(
            sidecar_path_in(Some(dir.as_path()), "ffmpeg"),
            PathBuf::from("ffmpeg")
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn falls_back_when_exe_dir_unknown() {
        assert_eq!(sidecar_path_in(None, "ffprobe"), PathBuf::from("ffprobe"));
    }

    #[test]
    fn unknown_sidecar_falls_back_to_plain_name() {
        let p = sidecar_path("sidecar-yang-tidak-ada");
        assert_eq!(p, Path::new("sidecar-yang-tidak-ada"));
    }
}
