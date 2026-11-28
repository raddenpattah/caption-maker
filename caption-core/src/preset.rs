//! Skema preset gaya caption dan pemuatan 12 preset bawaan.
//!
//! Semua ukuran dalam piksel pada acuan sisi pendek 1080; renderer yang
//! bertanggung jawab menskalakan sesuai geometri video aktual.

use serde::{Deserialize, Serialize};

use crate::model::WordsPerLine;

/// Keluarga font yang dipakai preset.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetFont {
    pub family: String,
    pub file: String,
    pub size: u32,
    pub bold: bool,
    pub italic: bool,
    pub uppercase: bool,
    pub letter_spacing: i32,
}

/// Warna dalam format `#RRGGBB` atau `#RRGGBBAA`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetColors {
    pub text: String,
    pub highlight: String,
    pub outline: String,
    pub shadow: String,
}

/// Kotak latar opsional (BorderStyle=3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetBox {
    pub color: String,
    pub padding: u32,
}

/// Glow opsional (event layer tambahan dengan outline berwarna).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PresetGlow {
    pub color: String,
    pub radius: u32,
}

/// Animasi saat kata caption mulai tampil.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Animation {
    #[default]
    None,
    Pop,
}

/// Mode penekanan kata.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Emphasis {
    None,
    Fill,
    ColorActive,
    UnderlineActive,
    ScaleActive,
}

/// Kategori preset (untuk pengelompokan di `--list-presets`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PresetCategory {
    KataPerKata,
    BarisBergaya,
    PenekananKata,
}

impl PresetCategory {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::KataPerKata => "kata_per_kata",
            Self::BarisBergaya => "baris_bergaya",
            Self::PenekananKata => "penekanan_kata",
        }
    }
}

/// Satu preset gaya caption.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StylePreset {
    pub id: String,
    pub name: String,
    pub category: PresetCategory,
    pub font: PresetFont,
    pub colors: PresetColors,
    pub outline: u32,
    pub shadow: u32,
    pub blur: u32,
    #[serde(rename = "box")]
    pub box_: Option<PresetBox>,
    pub glow: Option<PresetGlow>,
    pub emphasis: Emphasis,
    pub emphasis_scale: Option<u32>,
    #[serde(default)]
    pub animation: Animation,
    pub words_per_line: u8,
    pub alignment: u8,
    pub margin_v: u32,
    pub margin_lr: u32,
}

// Serde butuh nama JSON "box" tetapi "box" adalah keyword di Rust.
// Solusi: pakai field `box_` di Rust + rename ke "box" di JSON.
impl StylePreset {
    /// Helper untuk tes/CLI: apakah preset valid (dipanggil lewat `validate`).
    pub fn words_per_line_enum(&self) -> Result<WordsPerLine, String> {
        match self.words_per_line {
            1 => Ok(WordsPerLine::One),
            2 => Ok(WordsPerLine::Two),
            3 => Ok(WordsPerLine::Three),
            other => Err(format!("words_per_line harus 1..=3, dapat {other}")),
        }
    }
}

/// 12 preset bawaan, ditanam ke binary lewat `include_str!`.
const PRESETS_JSON: &[&str] = &[
    include_str!("../presets/kata-tebal-putih.json"),
    include_str!("../presets/kata-kotak-merah.json"),
    include_str!("../presets/kata-anton-kuning.json"),
    include_str!("../presets/kata-bangers.json"),
    include_str!("../presets/baris-kotak-hitam.json"),
    include_str!("../presets/baris-kotak-kuning.json"),
    include_str!("../presets/baris-neon-mint.json"),
    include_str!("../presets/baris-isi-kuning.json"),
    include_str!("../presets/tekan-warna-kuning.json"),
    include_str!("../presets/tekan-garis-bawah.json"),
    include_str!("../presets/tekan-besar.json"),
    include_str!("../presets/tekan-ungu-kotak-gelap.json"),
];

/// Muat semua preset bawaan. Gagal hanya jika JSON tertanam rusak (bug build).
pub fn builtin_presets() -> Result<Vec<StylePreset>, String> {
    let mut out = Vec::with_capacity(PRESETS_JSON.len());
    for (i, raw) in PRESETS_JSON.iter().enumerate() {
        let p: StylePreset = serde_json::from_str(raw)
            .map_err(|e| format!("preset ke-{i} JSON tidak valid: {e}"))?;
        validate_preset(&p)?;
        out.push(p);
    }
    // Cek id unik.
    for i in 0..out.len() {
        for j in (i + 1)..out.len() {
            if out[i].id == out[j].id {
                return Err(format!("id preset duplikat: {}", out[i].id));
            }
        }
    }
    Ok(out)
}

/// Cari preset berdasarkan id.
pub fn find_preset(id: &str) -> Result<StylePreset, String> {
    builtin_presets()?
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("preset '{id}' tidak ditemukan"))
}

/// Validasi semua field dengan pesan error yang menyebut nama field.
pub fn validate_preset(p: &StylePreset) -> Result<(), String> {
    if p.id.is_empty() {
        return Err("field 'id' tidak boleh kosong".into());
    }
    if p.name.is_empty() {
        return Err(format!("field 'name' pada preset '{}' kosong", p.id));
    }
    if p.font.family.is_empty() {
        return Err(format!("field 'font.family' pada preset '{}' kosong", p.id));
    }
    if p.font.file.is_empty() {
        return Err(format!("field 'font.file' pada preset '{}' kosong", p.id));
    }
    if p.font.size == 0 || p.font.size > 400 {
        return Err(format!(
            "field 'font.size' pada preset '{}' harus 1..=400, dapat {}",
            p.id, p.font.size
        ));
    }
    if !(-50..=50).contains(&p.font.letter_spacing) {
        return Err(format!(
            "field 'font.letter_spacing' pada preset '{}' di luar -50..=50",
            p.id
        ));
    }
    for (field, value) in [
        ("colors.text", &p.colors.text),
        ("colors.highlight", &p.colors.highlight),
        ("colors.outline", &p.colors.outline),
        ("colors.shadow", &p.colors.shadow),
    ] {
        validate_color(field, value, &p.id)?;
    }
    if let Some(b) = &p.box_ {
        validate_color("box.color", &b.color, &p.id)?;
        if b.padding > 200 {
            return Err(format!("field 'box.padding' pada preset '{}' > 200", p.id));
        }
    }
    if let Some(g) = &p.glow {
        validate_color("glow.color", &g.color, &p.id)?;
        if g.radius == 0 || g.radius > 100 {
            return Err(format!(
                "field 'glow.radius' pada preset '{}' harus 1..=100",
                p.id
            ));
        }
    }
    if p.outline > 100 {
        return Err(format!("field 'outline' pada preset '{}' > 100", p.id));
    }
    if p.shadow > 100 {
        return Err(format!("field 'shadow' pada preset '{}' > 100", p.id));
    }
    if p.blur > 100 {
        return Err(format!("field 'blur' pada preset '{}' > 100", p.id));
    }
    if !(1..=3).contains(&p.words_per_line) {
        return Err(format!(
            "field 'words_per_line' pada preset '{}' harus 1..=3",
            p.id
        ));
    }
    if !(1..=9).contains(&p.alignment) {
        return Err(format!(
            "field 'alignment' pada preset '{}' harus 1..=9 (numpad)",
            p.id
        ));
    }
    if p.margin_v > 2000 {
        return Err(format!("field 'margin_v' pada preset '{}' > 2000", p.id));
    }
    if p.margin_lr > 2000 {
        return Err(format!("field 'margin_lr' pada preset '{}' > 2000", p.id));
    }
    if p.emphasis == Emphasis::ScaleActive {
        let s = p.emphasis_scale.ok_or_else(|| {
            format!(
                "field 'emphasis_scale' wajib diisi saat emphasis=scale_active (preset '{}')",
                p.id
            )
        })?;
        if !(100..=200).contains(&s) {
            return Err(format!(
                "field 'emphasis_scale' pada preset '{}' harus 100..=200",
                p.id
            ));
        }
    }
    Ok(())
}

fn validate_color(field: &str, value: &str, preset_id: &str) -> Result<(), String> {
    let ok = (value.len() == 7 || value.len() == 9)
        && value.starts_with('#')
        && value[1..].chars().all(|c| c.is_ascii_hexdigit());
    if ok {
        Ok(())
    } else {
        Err(format!(
            "field '{field}' pada preset '{preset_id}' bukan '#RRGGBB' atau '#RRGGBBAA': {value}"
        ))
    }
}
