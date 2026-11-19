use std::path::Path;
use std::process::Command;

use crate::CaptionError;

pub fn extract_audio(ffmpeg: &Path, input: &Path, out_wav: &Path) -> Result<(), CaptionError> {
    let output = Command::new(ffmpeg)
        .arg("-y")
        .arg("-i")
        .arg(input)
        .args(["-vn", "-ac", "1", "-ar", "16000", "-c:a", "pcm_s16le"])
        .arg(out_wav)
        .output()
        .map_err(|source| CaptionError::FfmpegLaunch {
            path: ffmpeg.to_owned(),
            source,
        })?;

    if !output.status.success() {
        return Err(CaptionError::FfmpegFailed {
            status: output.status.to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }

    Ok(())
}

pub fn load_wav_16k_mono(path: &Path) -> Result<Vec<f32>, CaptionError> {
    let mut reader = hound::WavReader::open(path)?;
    let spec = reader.spec();
    if spec.channels != 1
        || spec.sample_rate != 16_000
        || spec.bits_per_sample != 16
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err(CaptionError::InvalidWav {
            channels: spec.channels,
            sample_rate: spec.sample_rate,
            bits_per_sample: spec.bits_per_sample,
            sample_format: spec.sample_format,
        });
    }

    reader
        .samples::<i16>()
        .map(|sample| {
            sample
                .map(|value| f32::from(value) / 32_768.0)
                .map_err(CaptionError::from)
        })
        .collect()
}
