mod ass;
mod audio;
mod error;
pub mod exporter;
mod group;
mod merge;
pub mod model;
#[cfg(feature = "whisper")]
mod whisper;

pub use ass::{render_ass, validate_ass, write_ass, AssValidation};
pub use audio::{extract_audio, load_wav_16k_mono};
pub use error::CaptionError;
pub use exporter::{to_json, to_srt, to_txt, to_vtt, Format};
pub use group::group_words;
pub use merge::merge_tokens_to_words;
pub use model::{
    AssOptions, AssStyle, Group, RawToken, Rgb, TranscribeOptions, Word, WordsPerLine,
};
#[cfg(feature = "whisper")]
pub use whisper::transcribe;
