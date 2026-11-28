//! Renderer ASS berbasis preset gaya.
//!
//! Tidak mengubah `ass.rs` lama: modul ini menyediakan jalur baru
//! `render_ass_with_preset` untuk mode emphasis dan fitur preset.
//! Semua ukuran preset dalam piksel acuan sisi pendek 1080.

use crate::model::{Group, WordsPerLine};
use crate::preset::{Animation, Emphasis, StylePreset};

const POP_START_SCALE: u32 = 85;
const POP_OVERSHOOT_SCALE: u32 = 108;
const POP_UP_MS: u32 = 90;
const POP_TOTAL_MS: u32 = 160;
const POP_MIN_DURATION_MS: u32 = 40;

/// Posisi anchor tengah caption sebagai persentase dari PlayRes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaptionPosition {
    pub x_percent: f64,
    pub y_percent: f64,
}

impl CaptionPosition {
    pub fn validate(self) -> Result<(), String> {
        if !self.x_percent.is_finite()
            || !self.y_percent.is_finite()
            || !(0.0..=100.0).contains(&self.x_percent)
            || !(0.0..=100.0).contains(&self.y_percent)
        {
            return Err(
                "posisi caption harus berupa persentase finite dalam rentang 0..=100".into(),
            );
        }
        Ok(())
    }

    fn ass_override(self, geometry: &VideoGeometry) -> String {
        let x = (self.x_percent * f64::from(geometry.width) / 100.0).round() as u32;
        let y = (self.y_percent * f64::from(geometry.height) / 100.0).round() as u32;
        format!("{{\\an5\\pos({x},{y})}}")
    }
}

/// Geometri video sebenarnya (setelah rotasi).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VideoGeometry {
    pub width: u32,
    pub height: u32,
}

impl VideoGeometry {
    /// Skala terhadap acuan sisi pendek 1080.
    pub fn scale(&self) -> f64 {
        f64::from(self.width.min(self.height)) / 1080.0
    }
    pub fn is_portrait(&self) -> bool {
        self.height > self.width
    }
}

/// Opsi tambahan untuk render preset.
#[derive(Debug, Clone, Copy, Default)]
pub struct PresetOptions {
    /// Override `words_per_line` dari preset (None = pakai preset).
    pub words_per_line: Option<WordsPerLine>,
    /// Posisi anchor tengah caption; None mempertahankan alignment dan margin preset.
    pub caption_position: Option<CaptionPosition>,
}

/// Karakter teks yang harus di-escape di ASS.
fn escape_text(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('{', "\\{")
        .replace('}', "\\}")
        .replace('\n', "\\N")
}

/// Konversi "#RRGGBB" / "#RRGGBBAA" ke `&HAABBGGRR&`.
/// Alpha ASS terbalik: input AA (0=transparan..FF=opak) → ASS AA (00=opak..FF=transparan).
pub fn ass_color_from_hex(hex: &str) -> Result<String, String> {
    let h = hex.strip_prefix('#').unwrap_or(hex);
    let (rgb, aa) = match h.len() {
        6 => (h, "FF"),
        8 => (&h[..6], &h[6..]),
        _ => return Err(format!("warna tidak valid: {hex}")),
    };
    let r = u8::from_str_radix(&rgb[0..2], 16).map_err(|_| format!("warna tidak valid: {hex}"))?;
    let g = u8::from_str_radix(&rgb[2..4], 16).map_err(|_| format!("warna tidak valid: {hex}"))?;
    let b = u8::from_str_radix(&rgb[4..6], 16).map_err(|_| format!("warna tidak valid: {hex}"))?;
    let a_in = u8::from_str_radix(aa, 16).map_err(|_| format!("warna tidak valid: {hex}"))?;
    let a_ass = 255 - a_in;
    Ok(format!("&H{a_ass:02X}{b:02X}{g:02X}{r:02X}&"))
}

/// Format timestamp ASS `H:MM:SS.cc`.
fn timestamp(ms: u32) -> String {
    let cs = ms / 10;
    let h = cs / 360_000;
    let m = (cs % 360_000) / 6_000;
    let s = (cs % 6_000) / 100;
    let centi = cs % 100;
    format!("{h}:{m:02}:{s:02}.{centi:02}")
}

fn pop_tags(preset: &StylePreset, group: &Group, start_ms: u32, end_ms: u32) -> String {
    let duration_ms = end_ms.saturating_sub(start_ms);
    if preset.animation != Animation::Pop
        || group.words.len() != 1
        || preset.emphasis == Emphasis::ScaleActive
        || duration_ms < POP_MIN_DURATION_MS
    {
        return String::new();
    }

    let total_ms = duration_ms.min(POP_TOTAL_MS);
    let up_ms =
        (f64::from(total_ms) * f64::from(POP_UP_MS) / f64::from(POP_TOTAL_MS)).round() as u32;
    format!(
        "{{\\fscx{POP_START_SCALE}\\fscy{POP_START_SCALE}\\t(0,{up_ms},\\fscx{POP_OVERSHOOT_SCALE}\\fscy{POP_OVERSHOOT_SCALE})\\t({up_ms},{total_ms},\\fscx100\\fscy100)}}"
    )
}

/// Bangun teks satu grup dengan seluruh kata, kata ke-`active` diberi override.
fn build_group_text(preset: &StylePreset, group: &Group, active: Option<usize>) -> String {
    let mut out = String::new();
    for (i, w) in group.words.iter().enumerate() {
        if i > 0 {
            out.push(' ');
        }
        let text = if preset.font.uppercase {
            w.text.to_uppercase()
        } else {
            w.text.clone()
        };
        let text = escape_text(&text);
        match active {
            Some(a) if a == i => {
                let mut tags = String::new();
                match preset.emphasis {
                    Emphasis::ColorActive => {
                        let c = ass_color_from_hex(&preset.colors.highlight).unwrap_or_default();
                        tags.push_str(&format!("\\c{c}"));
                    }
                    Emphasis::UnderlineActive => {
                        let c = ass_color_from_hex(&preset.colors.highlight).unwrap_or_default();
                        tags.push_str(&format!("\\c{c}\\u1"));
                    }
                    Emphasis::ScaleActive => {
                        let c = ass_color_from_hex(&preset.colors.highlight).unwrap_or_default();
                        let s = preset.emphasis_scale.unwrap_or(100);
                        tags.push_str(&format!("\\c{c}\\fscx{s}\\fscy{s}"));
                    }
                    _ => {}
                }
                out.push_str(&format!("{{{tags}}}{text}"));
            }
            _ => out.push_str(&text),
        }
    }
    out
}

/// Gaya global dari preset (dipakai di baris Style).
fn build_style_line(preset: &StylePreset, geom: &VideoGeometry) -> String {
    let scale = geom.scale();
    let font_size = (f64::from(preset.font.size) * scale).round() as u32;
    let outline = (f64::from(preset.outline) * scale).round() as u32;
    let shadow = (f64::from(preset.shadow) * scale).round() as u32;
    let margin_v = if geom.is_portrait() {
        (f64::from(preset.margin_v) * scale).round() as u32
    } else {
        (f64::from(geom.height) * 0.08).round() as u32
    };
    let margin_lr = (f64::from(preset.margin_lr) * scale).round() as u32;
    let primary = ass_color_from_hex(&preset.colors.text).unwrap_or("&H00FFFFFF&".into());
    let secondary = primary.clone();
    let (border_style, outline_color, outline_thickness) = if let Some(b) = &preset.box_ {
        let c = ass_color_from_hex(&b.color).unwrap_or("&H00000000&".into());
        (3u32, c, (f64::from(b.padding) * scale).round() as u32)
    } else {
        let c = ass_color_from_hex(&preset.colors.outline).unwrap_or("&H00000000&".into());
        (1u32, c, outline)
    };
    let back = ass_color_from_hex(&preset.colors.shadow).unwrap_or("&H80000000&".into());
    let bold = if preset.font.bold { -1 } else { 0 };
    let italic = if preset.font.italic { -1 } else { 0 };
    let spacing = (f64::from(preset.font.letter_spacing) * scale).round() as i32;
    format!(
        "Style: Default,{},{},{},{},{},{},{bold},{italic},0,0,100,100,{spacing},0,{border_style},{outline_thickness},{shadow},{},{},{},{},1",
        preset.font.family,
        font_size,
        primary,
        secondary,
        outline_color,
        back,
        preset.alignment,
        margin_lr,
        margin_lr,
        margin_v
    )
}

/// Render ASS lengkap berdasarkan preset.
pub fn render_ass_with_preset(
    groups: &[Group],
    preset: &StylePreset,
    geometry: VideoGeometry,
    options: &PresetOptions,
) -> String {
    let per_line = options.words_per_line.or(preset.words_per_line_enum().ok());

    let regrouped: Vec<Group> = match per_line {
        Some(WordsPerLine::Sentence) => groups.to_vec(),
        Some(per_line) => groups
            .iter()
            .flat_map(|g| g.words.iter().cloned())
            .collect::<Vec<_>>()
            .chunks(per_line.get())
            .map(|chunk| Group {
                words: chunk.to_vec(),
            })
            .collect(),
        None => Vec::new(),
    };

    let mut out = String::new();
    out.push_str("[Script Info]\n");
    out.push_str("; Script generated by Caption Maker (preset)\n");
    out.push_str("ScriptType: v4.00+\n");
    out.push_str(&format!("PlayResX: {}\n", geometry.width));
    out.push_str(&format!("PlayResY: {}\n", geometry.height));
    out.push_str("WrapStyle: 0\n");
    out.push_str("ScaledBorderAndShadow: yes\n\n");
    out.push_str("[V4+ Styles]\n");
    out.push_str("Format: Name, Fontname, Fontsize, PrimaryColour, SecondaryColour, OutlineColour, BackColour, Bold, Italic, Underline, StrikeOut, ScaleX, ScaleY, Spacing, Angle, BorderStyle, Outline, Shadow, Alignment, MarginL, MarginR, MarginV, Encoding\n");
    out.push_str(&build_style_line(preset, &geometry));
    out.push('\n');
    out.push_str("\n[Events]\n");
    out.push_str(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text\n",
    );

    let glow_layer = preset.glow.is_some();
    let position_override = options
        .caption_position
        .map(|position| position.ass_override(&geometry))
        .unwrap_or_default();

    for group in &regrouped {
        if group.words.is_empty() {
            continue;
        }
        match preset.emphasis {
            Emphasis::None | Emphasis::Fill => {
                if glow_layer {
                    render_glow_events(&mut out, preset, group, &geometry, &position_override);
                }
                let text = match preset.emphasis {
                    Emphasis::Fill => build_fill_text(preset, group, &geometry),
                    _ => build_group_text(preset, group, None),
                };
                let pop = pop_tags(preset, group, group.start_ms(), group.end_ms());
                out.push_str(&format!(
                    "Dialogue: 1,{},{},Default,,0,0,0,,{}{}{}\n",
                    timestamp(group.start_ms()),
                    timestamp(group.end_ms()),
                    pop,
                    position_override,
                    text
                ));
            }
            Emphasis::ColorActive | Emphasis::UnderlineActive | Emphasis::ScaleActive => {
                for (i, _) in group.words.iter().enumerate() {
                    let start = group.words[i].start_ms;
                    let end = if i + 1 < group.words.len() {
                        group.words[i + 1].start_ms
                    } else {
                        group.words.last().unwrap().end_ms
                    };
                    if glow_layer {
                        render_glow_single(
                            &mut out,
                            preset,
                            group,
                            start,
                            end,
                            &geometry,
                            &position_override,
                        );
                    }
                    let pop = pop_tags(preset, group, start, end);
                    let text = build_group_text(preset, group, Some(i));
                    out.push_str(&format!(
                        "Dialogue: 1,{},{},Default,,0,0,0,,{}{}{}\n",
                        timestamp(start),
                        timestamp(end),
                        pop,
                        position_override,
                        text
                    ));
                }
            }
        }
    }
    out
}

fn build_fill_text(preset: &StylePreset, group: &Group, _geom: &VideoGeometry) -> String {
    let mut out = String::new();
    let mut cumulative_cs = 0_i64;
    let group_start = i64::from(group.start_ms());
    for w in &group.words {
        let start = i64::from(w.start_ms);
        let end = i64::from(w.end_ms);
        let gap_cs = (start - group_start).max(0) / 10 - cumulative_cs;
        if gap_cs > 0 {
            out.push_str(&format!("{{\\k{gap_cs}}}"));
            cumulative_cs += gap_cs;
        }
        let end_cs = (end - group_start).max(0) / 10;
        let word_cs = (end_cs - cumulative_cs).max(0);
        out.push_str(&format!("{{\\kf{word_cs}}}"));
        let text = if preset.font.uppercase {
            w.text.to_uppercase()
        } else {
            w.text.clone()
        };
        out.push_str(&escape_text(&text));
        out.push(' ');
        cumulative_cs += word_cs;
    }
    out.trim_end().to_owned()
}

fn render_glow_events(
    out: &mut String,
    preset: &StylePreset,
    group: &Group,
    geometry: &VideoGeometry,
    position_override: &str,
) {
    let g = preset.glow.as_ref().unwrap();
    let c = ass_color_from_hex(&g.color).unwrap_or("&H00000000&".into());
    let r = (f64::from(g.radius) * geometry.scale()).round() as u32;
    let text = build_group_text(preset, group, None);
    let pop = pop_tags(preset, group, group.start_ms(), group.end_ms());
    out.push_str(&format!(
        "Dialogue: 0,{},{},Default,,0,0,0,,{}{position_override}{{\\bord{r}\\blur{r}\\c{c}\\3c{c}}}{text}\n",
        timestamp(group.start_ms()),
        timestamp(group.end_ms()),
        pop,
    ));
}

fn render_glow_single(
    out: &mut String,
    preset: &StylePreset,
    group: &Group,
    start: u32,
    end: u32,
    geometry: &VideoGeometry,
    position_override: &str,
) {
    let g = preset.glow.as_ref().unwrap();
    let c = ass_color_from_hex(&g.color).unwrap_or("&H00000000&".into());
    let r = (f64::from(g.radius) * geometry.scale()).round() as u32;
    let text = build_group_text(preset, group, None);
    let pop = pop_tags(preset, group, start, end);
    out.push_str(&format!(
        "Dialogue: 0,{},{},Default,,0,0,0,,{}{position_override}{{\\bord{r}\\blur{r}\\c{c}\\3c{c}}}{text}\n",
        timestamp(start),
        timestamp(end),
        pop,
    ));
}
