//! Exporter untuk format subtitle selain ASS: SRT, VTT, TXT, JSON.

use crate::model::Group;
use serde::Serialize;

/// Format ekspor yang didukung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    Ass,
    Srt,
    Vtt,
    Txt,
    Json,
}

fn timestamp_srt(ms: u32) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1_000;
    let millis = ms % 1_000;
    format!("{h:02}:{m:02}:{s:02},{millis:03}")
}

fn timestamp_vtt(ms: u32) -> String {
    let h = ms / 3_600_000;
    let m = (ms % 3_600_000) / 60_000;
    let s = (ms % 60_000) / 1_000;
    let millis = ms % 1_000;
    format!("{h:02}:{m:02}:{s:02}.{millis:03}")
}

fn group_text(group: &Group) -> String {
    group
        .words
        .iter()
        .map(|w| w.text.trim())
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn to_srt(groups: &[Group]) -> String {
    let mut out = String::new();
    for (i, group) in groups.iter().enumerate() {
        if group.words.is_empty() {
            continue;
        }
        out.push_str(&format!("{}\n", i + 1));
        out.push_str(&format!(
            "{} --> {}\n",
            timestamp_srt(group.start_ms()),
            timestamp_srt(group.end_ms())
        ));
        out.push_str(&group_text(group));
        out.push_str("\n\n");
    }
    out
}

pub fn to_vtt(groups: &[Group]) -> String {
    let mut out = String::from("WEBVTT\n\n");
    for group in groups.iter() {
        if group.words.is_empty() {
            continue;
        }
        out.push_str(&format!(
            "{} --> {}\n",
            timestamp_vtt(group.start_ms()),
            timestamp_vtt(group.end_ms())
        ));
        out.push_str(&group_text(group));
        out.push_str("\n\n");
    }
    out
}

pub fn to_txt(groups: &[Group]) -> String {
    let mut out = String::new();
    for group in groups.iter() {
        if group.words.is_empty() {
            continue;
        }
        out.push_str(&group_text(group));
        out.push('\n');
    }
    out
}

#[derive(Serialize)]
struct JsonWord<'a> {
    text: &'a str,
    start_ms: u32,
    end_ms: u32,
}

#[derive(Serialize)]
struct JsonGroup<'a> {
    start_ms: u32,
    end_ms: u32,
    text: String,
    words: Vec<JsonWord<'a>>,
}

pub fn to_json(groups: &[Group]) -> Result<String, serde_json::Error> {
    let payload: Vec<JsonGroup> = groups
        .iter()
        .filter(|g| !g.words.is_empty())
        .map(|g| JsonGroup {
            start_ms: g.start_ms(),
            end_ms: g.end_ms(),
            text: group_text(g),
            words: g
                .words
                .iter()
                .map(|w| JsonWord {
                    text: &w.text,
                    start_ms: w.start_ms,
                    end_ms: w.end_ms,
                })
                .collect(),
        })
        .collect();
    serde_json::to_string_pretty(&payload)
}
