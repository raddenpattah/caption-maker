use std::path::{Path, PathBuf};

use caption_core::video::{
    build_burn_args, default_ffprobe_for, parse_ffprobe_json, parse_progress_line, percent,
    Quality, VideoInfo, WorkDirGuard,
};

fn probe_json_1080x1920_aac() -> &'static str {
    r#"{
      "streams": [
        {"codec_type":"video","width":1080,"height":1920,"pix_fmt":"yuv420p","color_transfer":"bt709"},
        {"codec_type":"audio","codec_name":"aac"}
      ],
      "format": {"duration": "23.185000"}
    }"#
}

fn probe_json_rot90_hdr() -> &'static str {
    r#"{
      "streams": [
        {
          "codec_type":"video","width":1920,"height":1080,
          "pix_fmt":"yuv420p10le","color_transfer":"smpte2084",
          "side_data_list":[{"rotation":90}]
        }
      ],
      "format": {"duration": "12.5"}
    }"#
}

fn probe_json_rot270_tags() -> &'static str {
    r#"{
      "streams": [
        {"codec_type":"video","width":1280,"height":720,"pix_fmt":"yuv420p",
         "tags":{"rotate":"270"}}
      ],
      "format": {"duration": "8.0"}
    }"#
}

fn probe_json_audio_opus() -> &'static str {
    r#"{
      "streams": [
        {"codec_type":"video","width":1920,"height":1080,"pix_fmt":"yuv420p"},
        {"codec_type":"audio","codec_name":"opus"}
      ],
      "format": {"duration": "10.0"}
    }"#
}

fn probe_json_no_audio() -> &'static str {
    r#"{
      "streams": [
        {"codec_type":"video","width":720,"height":1280,"pix_fmt":"yuv420p"}
      ],
      "format": {"duration": "5.0"}
    }"#
}

#[test]
fn parse_basic_1080x1920_aac() {
    let info = parse_ffprobe_json(probe_json_1080x1920_aac()).unwrap();
    assert_eq!(info.width, 1080);
    assert_eq!(info.height, 1920);
    assert!((info.duration_s - 23.185).abs() < 1e-6);
    assert_eq!(info.audio_codec.as_deref(), Some("aac"));
    assert_eq!(info.pix_fmt, "yuv420p");
    assert!(!info.is_hdr);
}

#[test]
fn parse_rotation_90_swaps_and_detects_hdr() {
    let info = parse_ffprobe_json(probe_json_rot90_hdr()).unwrap();
    assert_eq!(info.width, 1080, "rot 90 harus tukar lebar-tinggi");
    assert_eq!(info.height, 1920);
    assert!(info.is_hdr, "smpte2084 = HDR");
    assert_eq!(info.audio_codec, None);
}

#[test]
fn parse_rotation_270_from_tags() {
    let info = parse_ffprobe_json(probe_json_rot270_tags()).unwrap();
    assert_eq!(info.width, 720, "rot 270 harus tukar");
    assert_eq!(info.height, 1280);
}

#[test]
fn parse_audio_opus() {
    let info = parse_ffprobe_json(probe_json_audio_opus()).unwrap();
    assert_eq!(info.audio_codec.as_deref(), Some("opus"));
}

#[test]
fn parse_no_audio() {
    let info = parse_ffprobe_json(probe_json_no_audio()).unwrap();
    assert_eq!(info.audio_codec, None);
}

#[test]
fn parse_invalid_json_fails() {
    assert!(parse_ffprobe_json("not json").is_err());
}

#[test]
fn parse_missing_streams_fails() {
    let bad = r#"{"format":{"duration":"1.0"}}"#;
    assert!(parse_ffprobe_json(bad).is_err());
}

#[test]
fn parse_missing_video_stream_fails() {
    let bad =
        r#"{"streams":[{"codec_type":"audio","codec_name":"aac"}],"format":{"duration":"1.0"}}"#;
    assert!(parse_ffprobe_json(bad).is_err());
}

#[test]
fn parse_unknown_duration_as_zero() {
    let json =
        r#"{"streams":[{"codec_type":"video","width":1080,"height":1920,"pix_fmt":"yuv420p"}]}"#;
    let info = parse_ffprobe_json(json).unwrap();
    assert_eq!(info.duration_s, 0.0);
}

fn sample_info(codec: Option<&str>) -> VideoInfo {
    VideoInfo {
        width: 1080,
        height: 1920,
        duration_s: 23.185,
        audio_codec: codec.map(String::from),
        pix_fmt: "yuv420p".into(),
        is_hdr: false,
    }
}

#[test]
fn burn_args_quality_cepat_uses_crf23_veryfast() {
    let args = build_burn_args(
        Path::new("/in.mp4"),
        Path::new("/tmp/work"),
        Path::new("/out.mp4"),
        Quality::Cepat,
        &sample_info(Some("aac")),
    );
    assert!(args.windows(2).any(|w| w[0] == "-crf" && w[1] == "23"));
    assert!(args
        .windows(2)
        .any(|w| w[0] == "-preset" && w[1] == "veryfast"));
    assert!(args.iter().any(|arg| arg == "-y"));
}

#[test]
fn burn_args_quality_bagus_uses_crf18_slow() {
    let args = build_burn_args(
        Path::new("/in.mp4"),
        Path::new("/tmp/work"),
        Path::new("/out.mp4"),
        Quality::Bagus,
        &sample_info(Some("aac")),
    );
    assert!(args.windows(2).any(|w| w[0] == "-crf" && w[1] == "18"));
    assert!(args.windows(2).any(|w| w[0] == "-preset" && w[1] == "slow"));
}

#[test]
fn burn_args_aac_audio_is_copied() {
    let args = build_burn_args(
        Path::new("/in.mp4"),
        Path::new("/tmp/work"),
        Path::new("/out.mp4"),
        Quality::Bagus,
        &sample_info(Some("aac")),
    );
    assert!(args.windows(2).any(|w| w[0] == "-c:a" && w[1] == "copy"));
    assert!(
        !args.iter().any(|a| a == "-b:a"),
        "tidak boleh re-encode aac"
    );
}

#[test]
fn burn_args_non_aac_audio_is_reencoded() {
    let args = build_burn_args(
        Path::new("/in.mp4"),
        Path::new("/tmp/work"),
        Path::new("/out.mp4"),
        Quality::Bagus,
        &sample_info(Some("opus")),
    );
    assert!(args.windows(2).any(|w| w[0] == "-c:a" && w[1] == "aac"));
    assert!(args.windows(2).any(|w| w[0] == "-b:a" && w[1] == "192k"));
}

#[test]
fn burn_args_no_audio_skips_audio_flags() {
    let args = build_burn_args(
        Path::new("/in.mp4"),
        Path::new("/tmp/work"),
        Path::new("/out.mp4"),
        Quality::Bagus,
        &sample_info(None),
    );
    assert!(!args.iter().any(|a| a == "-c:a"));
    assert!(!args.iter().any(|a| a == "-b:a"));
}

#[test]
fn burn_args_contains_relative_ass_filter() {
    let args = build_burn_args(
        Path::new("/in.mp4"),
        Path::new("/tmp/work"),
        Path::new("/out.mp4"),
        Quality::Bagus,
        &sample_info(Some("aac")),
    );
    let vf = args
        .windows(2)
        .find(|w| w[0] == "-vf")
        .map(|w| &w[1])
        .unwrap();
    assert!(
        vf.contains("sub.ass"),
        "filter harus pakai path relatif: {vf}"
    );
    assert!(
        vf.contains("fontsdir=fonts"),
        "filter harus punya fontsdir: {vf}"
    );
    assert!(
        !vf.contains("/tmp/work"),
        "filter tidak boleh pakai path absolut"
    );
}

#[test]
fn burn_args_has_no_size_or_fps_flags() {
    let args = build_burn_args(
        Path::new("/in.mp4"),
        Path::new("/tmp/work"),
        Path::new("/out.mp4"),
        Quality::Bagus,
        &sample_info(Some("aac")),
    );
    assert!(!args.iter().any(|a| a == "-s"), "ukuran tidak boleh diubah");
    assert!(!args.iter().any(|a| a == "-r"), "fps tidak boleh diubah");
}

#[test]
fn parse_progress_out_time_us() {
    let p = parse_progress_line("out_time_us=1500000").unwrap();
    assert_eq!(p.out_time_us, Some(1_500_000));
    assert!(!p.finished);
}

#[test]
fn parse_progress_end() {
    let p = parse_progress_line("progress=end").unwrap();
    assert!(p.finished);
}

#[test]
fn parse_progress_ignores_unrelated_lines() {
    assert!(parse_progress_line("frame=42").is_none());
    assert!(parse_progress_line("").is_none());
}

#[test]
fn percent_normal() {
    assert_eq!(percent(0, 10.0), 0);
    assert_eq!(percent(5_000_000, 10.0), 50);
    assert_eq!(percent(10_000_000, 10.0), 100);
    assert_eq!(percent(11_000_000, 10.0), 100, "clamp ke 100");
}

#[test]
fn percent_zero_duration_is_safe() {
    assert_eq!(percent(1_000_000, 0.0), 0);
    assert_eq!(percent(1_000_000, f64::NAN), 0);
}

#[test]
fn work_dir_guard_removes_directory_on_drop() {
    let work = WorkDirGuard::new("caption-work-dir-test").unwrap();
    let path = work.path().to_path_buf();
    std::fs::write(path.join("temporary.txt"), "temporary").unwrap();

    drop(work);

    assert!(!path.exists());
}

#[test]
fn default_ffprobe_for_unix_style() {
    let p = default_ffprobe_for(Path::new("/usr/bin/ffmpeg"));
    assert_eq!(p, PathBuf::from("/usr/bin/ffprobe"));
}

#[test]
fn default_ffprobe_for_windows_style() {
    let p = default_ffprobe_for(Path::new(r"C:\tools\ffmpeg.exe"));
    assert_eq!(p, PathBuf::from(r"C:\tools\ffprobe.exe"));
}

#[test]
fn default_ffprobe_for_custom_stem() {
    let p = default_ffprobe_for(Path::new("/opt/bin/ffmpeg-custom"));
    assert_eq!(p, PathBuf::from("/opt/bin/ffmpeg-custom_probe"));
}
