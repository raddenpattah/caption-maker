mod ass;
mod audio;
mod error;
mod group;
mod merge;
mod model;
#[cfg(feature = "whisper")]
mod whisper;

pub use ass::{render_ass, write_ass};
pub use audio::{extract_audio, load_wav_16k_mono};
pub use error::CaptionError;
pub use group::group_words;
pub use merge::merge_tokens_to_words;
pub use model::{
    AssOptions, AssStyle, Group, RawToken, Rgb, TranscribeOptions, Word, WordsPerLine,
};
#[cfg(feature = "whisper")]
pub use whisper::transcribe;
