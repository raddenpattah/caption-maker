//! Tes unit untuk `commands.rs` — logika murni tanpa runtime Tauri.

use crate::commands::*;
use caption_core::preset::Emphasis;

#[test]
fn list_models_reports_catalog_and_installed_quantized_files() {
    let folder = test_folder("list-models");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("ggml-small.bin"), b"small").unwrap();
    std::fs::write(folder.join("ggml-small-q5_1.bin"), b"quantized").unwrap();
    std::fs::write(folder.join("not-a-model.bin"), b"ignored").unwrap();

    let models = list_models_impl(&folder).unwrap();
    assert_eq!(models.len(), 5);
    assert!(models.iter().any(|model| {
        model.id == "small"
            && model.name == "Small"
            && model.installed
            && model.size_bytes == Some(5)
    }));
    assert!(models.iter().any(|model| {
        model.id == "small-q5_1" && model.filename == "ggml-small-q5_1.bin" && model.installed
    }));
    assert!(models
        .iter()
        .any(|model| model.id == "tiny" && !model.installed && model.size_bytes.is_none()));
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn import_model_rejects_wrong_extension_and_name() {
    let folder = test_folder("import-invalid-name");
    std::fs::create_dir_all(&folder).unwrap();
    let source = folder.join("ggml-small.txt");
    std::fs::write(&source, b"not a model").unwrap();
    let error = import_model_impl(&source, &folder.join("models")).unwrap_err();
    assert!(error.contains("Ekstensi model harus .bin"));

    let bad_name = folder.join("small.bin");
    std::fs::write(&bad_name, b"not a model").unwrap();
    let error = import_model_impl(&bad_name, &folder.join("models")).unwrap_err();
    assert!(error.contains("ggml-<nama>.bin"));
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn import_model_rejects_too_small_file() {
    let folder = test_folder("import-small-file");
    std::fs::create_dir_all(&folder).unwrap();
    let source = folder.join("ggml-small.bin");
    std::fs::write(&source, WHISPER_MODEL_MAGIC).unwrap();
    let error = import_model_impl(&source, &folder.join("models")).unwrap_err();
    assert!(error.contains("terlalu kecil"));
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn import_model_rejects_invalid_magic() {
    let folder = test_folder("import-bad-magic");
    std::fs::create_dir_all(&folder).unwrap();
    let source = create_sparse_model(&folder.join("ggml-bad.bin"), *b"nope");
    let error = import_model_impl(&source, &folder.join("models")).unwrap_err();
    assert!(error.contains("magic harus 0x67676d6c"));
    assert!(!folder.join("models").exists());
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn import_model_copies_valid_model_and_leaves_no_partial_file() {
    let folder = test_folder("import-valid-model");
    std::fs::create_dir_all(&folder).unwrap();
    let source = create_sparse_model(&folder.join("ggml-custom-q5_1.BIN"), WHISPER_MODEL_MAGIC);
    let models_dir = folder.join("models");

    let imported = import_model_impl(&source, &models_dir).unwrap();

    assert_eq!(imported.id, "custom-q5_1");
    assert_eq!(imported.filename, "ggml-custom-q5_1.bin");
    assert!(imported.installed);
    assert_eq!(
        std::fs::metadata(models_dir.join(&imported.filename))
            .unwrap()
            .len(),
        MIN_MODEL_SIZE_BYTES + 1
    );
    assert!(std::fs::read_dir(&models_dir).unwrap().all(|entry| !entry
        .unwrap()
        .file_name()
        .to_string_lossy()
        .ends_with(".part")));
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn import_model_atomically_replaces_same_filename() {
    let folder = test_folder("import-replace-model");
    std::fs::create_dir_all(&folder).unwrap();
    let models_dir = folder.join("models");
    std::fs::create_dir_all(&models_dir).unwrap();
    std::fs::write(models_dir.join("ggml-small.bin"), b"old model").unwrap();
    let source = create_sparse_model(&folder.join("ggml-small.bin"), WHISPER_MODEL_MAGIC);

    let imported = import_model_impl(&source, &models_dir).unwrap();

    assert!(imported.installed);
    assert_eq!(
        std::fs::metadata(models_dir.join("ggml-small.bin"))
            .unwrap()
            .len(),
        MIN_MODEL_SIZE_BYTES + 1
    );
    assert_eq!(
        std::fs::read_dir(&models_dir).unwrap().count(),
        1,
        "file sementara tidak boleh tertinggal"
    );
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn resolve_model_path_finds_installed_model_and_reports_missing_model() {
    let folder = test_folder("resolve-model");
    std::fs::create_dir_all(&folder).unwrap();
    let installed = folder.join("ggml-small.BIN");
    std::fs::write(&installed, b"model").unwrap();

    assert_eq!(resolve_model_path(&folder, "small").unwrap(), installed);
    let missing = resolve_model_path(&folder.join("missing-folder"), "base").unwrap_err();
    assert!(missing.contains("belum terpasang"));
    assert!(resolve_model_path(&folder, "../outside").is_err());
    std::fs::remove_dir_all(folder).unwrap();
}

#[test]
fn transcribe_manager_rejects_parallel_transcriptions() {
    let manager = TranscribeManager::default();
    let first = manager.reserve().unwrap();
    assert!(matches!(
        manager.reserve(),
        Err(error) if error.contains("Transkripsi lain sedang berjalan")
    ));
    drop(first);
    assert!(manager.reserve().is_ok());
}

#[test]
fn burn_manager_releases_lease_after_error() {
    let manager = BurnManager::default();
    let result: Result<(), &str> = {
        let _lease = manager.reserve().unwrap();
        Err("simulated burn failure")
    };

    assert_eq!(result, Err("simulated burn failure"));
    assert!(manager.reserve().is_ok());
}

#[test]
fn burn_manager_releases_lease_after_panic() {
    let manager = BurnManager::default();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _lease = manager.reserve().unwrap();
        panic!("simulated blocking-task panic");
    }));

    assert!(result.is_err());
    assert!(manager.reserve().is_ok());
}

#[test]
#[cfg(feature = "whisper")]
fn temporary_wav_guard_removes_file_on_drop() {
    let wav = TemporaryWav::new();
    std::fs::write(wav.path(), b"temporary audio").unwrap();
    let path = wav.path().to_owned();
    assert!(path.is_file());
    drop(wav);
    assert!(!path.exists());
}

#[test]
fn audio_probe_detects_missing_audio_and_duration() {
    let (has_audio, duration) = parse_audio_probe_info(
        r#"{"streams":[{"codec_type":"video"}],"format":{"duration":"12.5"}}"#,
    )
    .unwrap();
    assert!(!has_audio);
    assert_eq!(duration, Some(12.5));

    let (has_audio, _) =
        parse_audio_probe_info(r#"{"streams":[{"codec_type":"audio"}],"format":{}}"#).unwrap();
    assert!(has_audio);
}

#[test]
fn extraction_progress_maps_media_time_to_first_ten_percent() {
    let update = caption_core::parse_progress_line("out_time_us=5000000");
    assert_eq!(extraction_progress(10.0, update), Some(5));
    assert_eq!(extraction_progress(0.0, update), None);
}

fn test_folder(label: &str) -> std::path::PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "caption-maker-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn create_sparse_model(path: &std::path::Path, magic: [u8; 4]) -> std::path::PathBuf {
    use std::io::{Seek, SeekFrom, Write};

    let mut file = std::fs::File::create(path).unwrap();
    file.write_all(&magic).unwrap();
    file.seek(SeekFrom::Start(MIN_MODEL_SIZE_BYTES)).unwrap();
    file.write_all(&[0]).unwrap();
    path.to_owned()
}

#[test]
fn validate_size_ok() {
    assert_eq!(validate_size_impl("1080x1920").unwrap(), (1080, 1920));
    assert_eq!(validate_size_impl("720X1280").unwrap(), (720, 1280));
}

#[test]
fn validate_size_rejects_bad() {
    assert!(validate_size_impl("1080").is_err());
    assert!(validate_size_impl("0x1920").is_err());
    assert!(validate_size_impl("abcx1920").is_err());
}

#[test]
fn list_presets_returns_12() {
    let presets = list_presets_impl().unwrap();
    assert_eq!(presets.len(), 12);
    assert!(presets.iter().any(|p| p.id == "tekan-warna-kuning"));
    for (id, words_per_line) in [
        ("kata-tebal-putih", 1),
        ("kata-kotak-merah", 1),
        ("kata-anton-kuning", 1),
        ("kata-bangers", 1),
        ("baris-kotak-hitam", 3),
        ("baris-kotak-kuning", 3),
        ("baris-neon-mint", 3),
        ("baris-isi-kuning", 3),
        ("tekan-warna-kuning", 2),
        ("tekan-garis-bawah", 2),
        ("tekan-besar", 2),
        ("tekan-ungu-kotak-gelap", 3),
    ] {
        let preset = presets.iter().find(|preset| preset.id == id).unwrap();
        assert_eq!(preset.words_per_line, words_per_line, "{id}");
    }

    let anton = presets
        .iter()
        .find(|preset| preset.id == "kata-anton-kuning")
        .unwrap();
    assert_eq!(anton.font.family, "Anton");
    assert!(anton.font.uppercase);
    assert_eq!(anton.colors.text, "#FFD400");

    let boxed = presets
        .iter()
        .find(|preset| preset.id == "baris-kotak-kuning")
        .unwrap();
    assert_eq!(
        boxed
            .box_
            .as_ref()
            .map(|preset_box| preset_box.color.as_str()),
        Some("#FFE14D")
    );
    let boxed_json = serde_json::to_value(boxed).unwrap();
    assert_eq!(boxed_json["box"]["color"].as_str(), Some("#FFE14D"));
}

#[test]
fn list_presets_carries_glow_and_emphasis() {
    let presets = list_presets_impl().unwrap();
    let neon = presets
        .iter()
        .find(|preset| preset.id == "baris-neon-mint")
        .unwrap();
    let glow = neon.glow.as_ref().unwrap();
    assert_eq!(glow.color, "#5CFFC0");
    assert_eq!(glow.radius, 12);

    let tekan = presets
        .iter()
        .find(|preset| preset.id == "tekan-besar")
        .unwrap();
    assert_eq!(tekan.emphasis, Emphasis::ScaleActive);
    assert_eq!(tekan.emphasis_scale, Some(118));
}

#[test]
fn preset_summary_serializes_frontend_keys() {
    let presets = list_presets_impl().unwrap();
    let neon = presets
        .iter()
        .find(|preset| preset.id == "baris-neon-mint")
        .unwrap();
    let neon_json = serde_json::to_value(neon).unwrap();
    assert_eq!(neon_json["emphasis"], "none");
    assert_eq!(neon_json["glow"]["color"], "#5CFFC0");
    assert_eq!(neon_json["glow"]["radius"], 12);
    assert!(neon_json.get("box").is_some());
    assert!(neon_json.get("box_").is_none());

    let tekan = presets
        .iter()
        .find(|preset| preset.id == "tekan-besar")
        .unwrap();
    let tekan_json = serde_json::to_value(tekan).unwrap();
    assert_eq!(tekan_json["emphasis"], "scale_active");
    assert_eq!(tekan_json["emphasis_scale"], 118);
    assert!(tekan_json.get("glow").is_some());
    assert!(tekan_json.get("box").is_some());
    assert!(tekan_json.get("box_").is_none());
}

#[test]
fn render_ass_impl_ok() {
    let req = RenderRequest {
        words: vec![
            WordDto {
                text: "halo".into(),
                start_ms: 100,
                end_ms: 500,
            },
            WordDto {
                text: "dunia".into(),
                start_ms: 500,
                end_ms: 1000,
            },
        ],
        preset_id: "kata-tebal-putih".into(),
        width: 1080,
        height: 1920,
        words_per_line: Some(WordsPerLineWire::Number(1)),
        caption_position: None,
    };
    let res = render_ass_impl(&req).unwrap();
    assert!(res.ass.contains("[Script Info]"));
    assert!(res.ass.contains("PlayResX: 1080"));
    assert!(res.ass.contains("PlayResY: 1920"));
    assert!(res.ass.contains("HALO"), "harus uppercase");
    assert!(!res.ass.contains("\\pos("));
}

#[test]
fn render_ass_position_is_applied_to_primary_and_glow_events() {
    let req = RenderRequest {
        words: vec![
            WordDto {
                text: "Halo".into(),
                start_ms: 100,
                end_ms: 500,
            },
            WordDto {
                text: "dunia.".into(),
                start_ms: 500,
                end_ms: 1000,
            },
        ],
        preset_id: "baris-neon-mint".into(),
        width: 1080,
        height: 1920,
        words_per_line: Some(WordsPerLineWire::Number(2)),
        caption_position: Some(CaptionPositionWire {
            x_percent: 25.0,
            y_percent: 75.0,
        }),
    };

    let result = render_ass_impl(&req).unwrap();
    let dialogues: Vec<_> = result
        .ass
        .lines()
        .filter(|line| line.starts_with("Dialogue:"))
        .collect();
    assert!(dialogues
        .iter()
        .any(|line| line.starts_with("Dialogue: 0,")));
    assert!(dialogues
        .iter()
        .any(|line| line.starts_with("Dialogue: 1,")));
    assert!(dialogues
        .iter()
        .all(|line| line.contains("{\\an5\\pos(270,1440)}")));
}

#[test]
fn render_ass_rejects_invalid_caption_position() {
    let req = RenderRequest {
        words: vec![WordDto {
            text: "Caption".into(),
            start_ms: 100,
            end_ms: 500,
        }],
        preset_id: "kata-tebal-putih".into(),
        width: 1080,
        height: 1920,
        words_per_line: None,
        caption_position: Some(CaptionPositionWire {
            x_percent: 101.0,
            y_percent: 50.0,
        }),
    };
    assert!(render_ass_impl(&req).is_err());
}

#[test]
fn export_ass_forwards_caption_position() {
    let mut req = sample_export_request();
    req.caption_position = Some(CaptionPositionWire {
        x_percent: 50.0,
        y_percent: 50.0,
    });

    let ass = render_export_impl(&req, "ass").unwrap();
    assert!(ass.contains("{\\an5\\pos(540,960)}"));
}

#[test]
fn render_request_accepts_legacy_payload_without_caption_position() {
    let req: RenderRequest = serde_json::from_str(
        r#"{"words":[],"preset_id":"kata-tebal-putih","width":1080,"height":1920}"#,
    )
    .unwrap();
    assert!(req.caption_position.is_none());
}

#[test]
fn render_ass_impl_bad_preset() {
    let req = RenderRequest {
        words: vec![WordDto {
            text: "x".into(),
            start_ms: 0,
            end_ms: 100,
        }],
        preset_id: "tidak-ada".into(),
        width: 1080,
        height: 1920,
        words_per_line: None,
        caption_position: None,
    };
    assert!(render_ass_impl(&req).is_err());
}

#[test]
fn render_ass_impl_bad_words_per_line() {
    let req = RenderRequest {
        words: vec![WordDto {
            text: "x".into(),
            start_ms: 0,
            end_ms: 100,
        }],
        preset_id: "kata-tebal-putih".into(),
        width: 1080,
        height: 1920,
        words_per_line: Some(WordsPerLineWire::Number(5)),
        caption_position: None,
    };
    assert!(render_ass_impl(&req).is_err());
}

#[test]
fn render_ass_impl_accepts_sentence_wire_value() {
    let req = RenderRequest {
        words: vec![WordDto {
            text: "Halo.".into(),
            start_ms: 0,
            end_ms: 100,
        }],
        preset_id: "kata-tebal-putih".into(),
        width: 1080,
        height: 1920,
        words_per_line: Some(WordsPerLineWire::Sentence(SentenceWire::Sentence)),
        caption_position: None,
    };
    let result = render_ass_impl(&req).unwrap();
    assert!(result.ass.contains("HALO."));
}

#[test]
fn write_export_file_impl_writes_contents_and_returns_path() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "caption-maker-export-test-{}-{nonce}.ass",
        std::process::id()
    ));
    let path = path.to_string_lossy().into_owned();

    assert_eq!(
        write_export_file_impl(&path, "[Script Info]").unwrap(),
        path
    );
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "[Script Info]");
    std::fs::remove_file(path).unwrap();
}

fn sample_export_request() -> RenderRequest {
    RenderRequest {
        words: vec![
            WordDto {
                text: "Halo".into(),
                start_ms: 120,
                end_ms: 360,
            },
            WordDto {
                text: "dunia".into(),
                start_ms: 390,
                end_ms: 660,
            },
        ],
        preset_id: "kata-tebal-putih".into(),
        width: 1080,
        height: 1920,
        words_per_line: Some(WordsPerLineWire::Number(2)),
        caption_position: None,
    }
}

#[test]
fn write_export_file_for_format_renders_supported_formats() {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    for (format, expected) in [
        ("ass", "[Script Info]"),
        ("srt", "00:00:00,120 --> 00:00:00,660\nHalo dunia"),
        ("vtt", "WEBVTT\n\n00:00:00.120 --> 00:00:00.660\nHalo dunia"),
        ("txt", "Halo dunia\n"),
        ("json", "\"text\": \"Halo dunia\""),
    ] {
        let path = std::env::temp_dir().join(format!(
            "caption-maker-export-{format}-{}-{nonce}.{format}",
            std::process::id()
        ));
        let path = path.to_string_lossy().into_owned();
        assert_eq!(
            write_export_file_for_format_impl(&path, format, &sample_export_request()).unwrap(),
            path
        );
        let contents = std::fs::read_to_string(&path).unwrap();
        assert!(contents.contains(expected), "format: {format}");
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn write_export_file_for_format_rejects_unknown_format_without_writing() {
    let path = std::env::temp_dir().join(format!(
        "caption-maker-export-unknown-{}-{}.txt",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let path = path.to_string_lossy().into_owned();

    let result = write_export_file_for_format_impl(&path, "xml", &sample_export_request());
    assert!(matches!(result, Err(message) if message.contains("Format ekspor tidak didukung")));
    assert!(!std::path::Path::new(&path).exists());
}

#[test]
fn burn_output_requires_mp4_extension() {
    assert!(validate_mp4_output(std::path::Path::new("caption.mp4")).is_ok());
    assert!(validate_mp4_output(std::path::Path::new("caption.MP4")).is_ok());
    assert!(validate_mp4_output(std::path::Path::new("caption.ass")).is_err());
    assert!(validate_mp4_output(std::path::Path::new("caption")).is_err());
}

#[test]
fn burn_manager_rejects_a_second_active_burn() {
    let manager = BurnManager::default();
    let first = manager.reserve().unwrap();
    let second = manager.reserve();
    assert!(matches!(second, Err(error) if error.contains("sedang berjalan")));

    drop(first);

    assert!(manager.reserve().is_ok());
}

#[test]
fn burn_progress_is_indeterminate_without_duration() {
    let update = caption_core::parse_progress_line("out_time_us=1500000");
    let progress = burn_progress(0.0, update, "running");
    assert_eq!(progress.progress, None);
    assert!(progress.indeterminate);
}

#[test]
fn burn_progress_reports_percentage_with_duration() {
    let update = caption_core::parse_progress_line("out_time_us=5000000");
    let progress = burn_progress(10.0, update, "running");
    assert_eq!(progress.progress, Some(50));
    assert!(!progress.indeterminate);
}

#[cfg(feature = "whisper")]
#[test]
#[ignore = "requires CAPTION_TEST_MODEL and the bundled FFmpeg sidecars"]
fn transcribe_whisper_integration() {
    let model = std::env::var_os("CAPTION_TEST_MODEL")
        .expect("atur CAPTION_TEST_MODEL ke path ggml-small.bin");
    let model = std::path::PathBuf::from(model);
    assert!(
        model.is_file(),
        "model tidak ditemukan: {}",
        model.display()
    );
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace root")
        .to_owned();
    let input = root.join("samples").join("test.wav");
    let sidecars = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
    let ffmpeg = sidecars.join("ffmpeg-x86_64-pc-windows-msvc.exe");
    let ffprobe = sidecars.join("ffprobe-x86_64-pc-windows-msvc.exe");
    assert!(
        input.is_file(),
        "audio tidak ditemukan: {}",
        input.display()
    );
    assert!(
        ffmpeg.is_file(),
        "FFmpeg tidak ditemukan: {}",
        ffmpeg.display()
    );
    assert!(
        ffprobe.is_file(),
        "ffprobe tidak ditemukan: {}",
        ffprobe.display()
    );

    let lease = TranscribeManager::default().reserve().unwrap();
    let result = run_transcription(
        TranscribeProcess {
            input: &input,
            model_path: &model,
            ffmpeg: &ffmpeg,
            ffprobe: &ffprobe,
            language: "id",
            threads: None,
            prompt: None,
        },
        lease.job.clone(),
        |_| Ok(()),
    )
    .unwrap();

    assert!(
        !result.words.is_empty(),
        "transkripsi tidak menghasilkan kata"
    );
    assert!(result
        .words
        .windows(2)
        .all(|pair| pair[0].start_ms <= pair[1].start_ms));
    println!(
        "elapsed_ms={} audio_duration_ms={} rtf={:.3} threads={} words={}",
        result.elapsed_ms,
        result.audio_duration_ms,
        result.elapsed_ms as f64 / result.audio_duration_ms.max(1) as f64,
        result.threads,
        result.words.len()
    );
}

#[test]
#[ignore = "requires the bundled FFmpeg sidecar with libass and libx264"]
fn burn_video_sidecar_integration() {
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::sync::Arc;

    let sidecars = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("binaries");
    let ffmpeg = std::env::var_os("CAPTION_TEST_FFMPEG")
        .map(PathBuf::from)
        .unwrap_or_else(|| sidecars.join("ffmpeg-x86_64-pc-windows-msvc.exe"));
    let ffprobe = std::env::var_os("CAPTION_TEST_FFPROBE")
        .map(PathBuf::from)
        .unwrap_or_else(|| sidecars.join("ffprobe-x86_64-pc-windows-msvc.exe"));
    assert!(
        ffmpeg.is_file(),
        "sidecar tidak ditemukan: {}",
        ffmpeg.display()
    );
    assert!(
        ffprobe.is_file(),
        "sidecar tidak ditemukan: {}",
        ffprobe.display()
    );
    let folder = caption_core::WorkDirGuard::new("caption-burn-integration").unwrap();
    let input = folder.path().join("input.mp4");
    let output = folder.path().join("burned.mp4");

    let mut command = Command::new(&ffmpeg);
    configure_no_window(&mut command);
    let generated = command
        .args([
            "-y",
            "-f",
            "lavfi",
            "-i",
            "testsrc=size=1080x1920:rate=30",
            "-t",
            "2",
            "-c:v",
            "libx264",
            "-preset",
            "ultrafast",
            "-pix_fmt",
            "yuv420p",
            "-threads",
            "1",
        ])
        .arg(&input)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        generated.status.success(),
        "pembuatan video sintetis gagal: {}",
        String::from_utf8_lossy(&generated.stderr)
    );

    let info = probe_video_with_ffprobe(&input, &ffprobe).unwrap();
    let ass = render_ass_impl(&RenderRequest {
        words: vec![
            WordDto {
                text: "caption".into(),
                start_ms: 100,
                end_ms: 800,
            },
            WordDto {
                text: "test".into(),
                start_ms: 800,
                end_ms: 1500,
            },
        ],
        preset_id: "kata-anton-kuning".into(),
        width: info.width,
        height: info.height,
        words_per_line: Some(WordsPerLineWire::Number(1)),
        caption_position: Some(CaptionPositionWire {
            x_percent: 50.0,
            y_percent: 50.0,
        }),
    })
    .unwrap()
    .ass;
    assert!(ass.contains("{\\an5\\pos(540,960)}"));
    let job = Arc::new(BurnJob::default());
    run_ffmpeg_burn(
        BurnProcess {
            ffmpeg: &ffmpeg,
            input: &input,
            output: &output,
            ass: &ass,
            info: &info,
            quality: caption_core::Quality::Cepat,
        },
        &job,
        |_| Ok(()),
    )
    .unwrap();

    assert!(output.is_file(), "output burn tidak terbentuk");
    let output_info = probe_video_with_ffprobe(&output, &ffprobe).unwrap();
    assert_eq!((output_info.width, output_info.height), (1080, 1920));
    assert!(output_info.duration_s >= 1.9);
}
