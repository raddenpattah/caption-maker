use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use caption_core::{load_wav_16k_mono, CaptionError};

struct TestWav(PathBuf);

impl TestWav {
    fn new(channels: u16, sample_rate: u32) -> Self {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "caption-core-test-{}-{nonce}.wav",
            std::process::id()
        ));
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };
        let mut writer = hound::WavWriter::create(&path, spec).expect("create test WAV");
        for sample in [-32_768_i16, 0, 16_384] {
            for _ in 0..channels {
                writer.write_sample(sample).expect("write test sample");
            }
        }
        writer.finalize().expect("finalize test WAV");
        Self(path)
    }
}

impl Drop for TestWav {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[test]
fn loads_16khz_mono_i16_as_normalized_f32() {
    let wav = TestWav::new(1, 16_000);
    let samples = load_wav_16k_mono(&wav.0).expect("load valid WAV");
    assert_eq!(samples, [-1.0, 0.0, 0.5]);
}

#[test]
fn rejects_wav_with_wrong_channel_count_or_sample_rate() {
    let wav = TestWav::new(2, 44_100);
    let error = load_wav_16k_mono(&wav.0).expect_err("reject incompatible WAV");
    assert!(matches!(error, CaptionError::InvalidWav { .. }));
}
