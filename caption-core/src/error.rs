use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum CaptionError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("WAV error: {0}")]
    Wav(#[from] hound::Error),
    #[error("input WAV must be 16 kHz, mono, signed 16-bit PCM (got {channels} channel(s), {sample_rate} Hz, {bits_per_sample} bits, {sample_format:?})")]
    InvalidWav {
        channels: u16,
        sample_rate: u32,
        bits_per_sample: u16,
        sample_format: hound::SampleFormat,
    },
    #[error("failed to run FFmpeg at {path}: {source}")]
    FfmpegLaunch {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("FFmpeg exited with {status}: {stderr}")]
    FfmpegFailed { status: String, stderr: String },
    #[cfg(feature = "whisper")]
    #[error("Whisper error: {0}")]
    Whisper(#[from] whisper_rs::WhisperError),
    #[error("model path is not valid UTF-8: {0}")]
    InvalidModelPath(PathBuf),
    #[error("thread count must be positive")]
    InvalidThreadCount,
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}
