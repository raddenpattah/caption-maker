//! Command Tauri: jembatan UI ke `caption-core`.
//!
//! Semua command yang mengembalikan file path memakai `String` (bukan `PathBuf`)
//! karena Tauri membutuhkan tipe serde-serializable.

use serde::{Deserialize, Serialize};

use caption_core::model::Word;
use caption_core::preset::StylePreset;

// ---------- Tipe data bersama ----------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub installed: bool,
}

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    pub words: Vec<WordDto>,
    pub preset_id: String,
    pub width: u32,
    pub height: u32,
    pub words_per_line: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResult {
    pub ass: String,
}

// ---------- Command (logika murni, mudah dites) ----------

/// Daftar model yang diketahui (mock — model nyata diunduh manual oleh user).
pub fn list_models_impl() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "base".into(),
            name: "Base".into(),
            installed: false,
        },
        ModelInfo {
            id: "small".into(),
            name: "Small".into(),
            installed: true,
        },
        ModelInfo {
            id: "medium".into(),
            name: "Medium".into(),
            installed: false,
        },
    ]
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
        })
        .collect())
}

/// Render ASS dari preset + words.
pub fn render_ass_impl(req: &RenderRequest) -> Result<RenderResult, String> {
    let preset: StylePreset = caption_core::find_preset(&req.preset_id)?;
    let words: Vec<Word> = req.words.iter().cloned().map(Word::from).collect();
    let per_line = match req.words_per_line {
        Some(1) => caption_core::WordsPerLine::One,
        Some(2) => caption_core::WordsPerLine::Two,
        Some(3) => caption_core::WordsPerLine::Three,
        Some(n) => return Err(format!("words_per_line tidak valid: {n}")),
        None => preset.words_per_line_enum()?,
    };
    let groups = caption_core::group_words(&words, per_line);
    let geometry = caption_core::VideoGeometry {
        width: req.width,
        height: req.height,
    };
    let opts = caption_core::PresetOptions {
        words_per_line: req.words_per_line,
    };
    let ass = caption_core::render_ass_with_preset(&groups, &preset, geometry, &opts);
    Ok(RenderResult { ass })
}

// ---------- Command (tauri wrapper) ----------

#[tauri::command]
pub async fn list_models() -> Result<Vec<ModelInfo>, String> {
    Ok(list_models_impl())
}

#[tauri::command]
pub async fn import_model(path: String) -> Result<String, String> {
    if !path.ends_with(".bin") {
        return Err(format!("model harus .bin: {path}"));
    }
    Ok(path)
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
