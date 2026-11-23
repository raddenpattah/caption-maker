use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Word {
    pub text: String,
    pub start_ms: u32,
    pub end_ms: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Group {
    pub words: Vec<Word>,
}

impl Group {
    pub fn start_ms(&self) -> u32 {
        self.words
            .first()
            .expect("groups must contain words")
            .start_ms
    }

    pub fn end_ms(&self) -> u32 {
        self.words.last().expect("groups must contain words").end_ms
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RawToken {
    pub text: String,
    #[serde(default)]
    pub bytes: Vec<u8>,
    pub t0_ms: i64,
    pub t1_ms: i64,
    pub id: i32,
    pub p: f32,
    pub is_special: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum WordsPerLine {
    One = 1,
    Two = 2,
    Three = 3,
    Sentence = 0,
}

impl WordsPerLine {
    pub fn get(self) -> usize {
        match self {
            WordsPerLine::One => 1,
            WordsPerLine::Two => 2,
            WordsPerLine::Three => 3,
            WordsPerLine::Sentence => 0,
        }
    }
}

pub struct TranscribeOptions {
    pub model_path: PathBuf,
    pub language: String,
    pub threads: i32,
    pub initial_prompt: Option<String>,
    pub on_progress: Option<Arc<dyn Fn(i32) + Send + Sync>>,
    pub on_abort: Option<Arc<AtomicBool>>,
}

impl TranscribeOptions {
    pub fn new(model_path: PathBuf) -> Self {
        let threads = std::thread::available_parallelism()
            .map(usize::from)
            .unwrap_or(1)
            .min(8) as i32;

        Self {
            model_path,
            language: "id".to_owned(),
            threads,
            initial_prompt: None,
            on_progress: None,
            on_abort: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssStyle {
    pub font_name: String,
    pub font_size: u32,
    pub primary: Rgb,
    pub secondary: Rgb,
    pub outline_color: Rgb,
    pub outline: f32,
    pub shadow: f32,
    pub alignment: u8,
    pub margin_v: u32,
}

impl Default for AssStyle {
    fn default() -> Self {
        Self {
            font_name: "Arial".to_owned(),
            font_size: 64,
            primary: Rgb {
                r: 255,
                g: 255,
                b: 0,
            },
            secondary: Rgb {
                r: 255,
                g: 255,
                b: 255,
            },
            outline_color: Rgb { r: 0, g: 0, b: 0 },
            outline: 3.0,
            shadow: 1.0,
            alignment: 2,
            margin_v: 60,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct AssOptions {
    pub style: AssStyle,
    pub highlight: bool,
    pub play_res_x: u32,
    pub play_res_y: u32,
}

impl Default for AssOptions {
    fn default() -> Self {
        Self {
            style: AssStyle::default(),
            highlight: false,
            play_res_x: 1920,
            play_res_y: 1080,
        }
    }
}
