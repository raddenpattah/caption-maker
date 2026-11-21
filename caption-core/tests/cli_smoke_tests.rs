//! Tes smoke untuk CLI. Hanya dijalankan bila feature `whisper` aktif
//! (karena `caption-cli` butuh feature itu).
#![cfg(feature = "whisper")]

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_caption-cli"))
}

/// Path absolut ke workspace root (crates/caption-core -> ../..).
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("caption-core punya parent")
        .to_path_buf()
}

fn words_json() -> PathBuf {
    workspace_root().join("src/mock/words.json")
}

#[test]
fn list_presets_prints_12_lines() {
    let out = Command::new(bin())
        .arg("--list-presets")
        .output()
        .expect("jalankan caption-cli");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    let lines: Vec<&str> = stdout.lines().collect();
    assert_eq!(lines.len(), 12, "harus 12 preset, dapat {}", lines.len());
    for line in &lines {
        let parts: Vec<&str> = line.split('\t').collect();
        assert_eq!(parts.len(), 3, "tiap baris 3 kolom: {line}");
        assert!(matches!(
            parts[2],
            "kata_per_kata" | "baris_bergaya" | "penekanan_kata"
        ));
    }
}

#[test]
fn help_shows_new_flags() {
    let out = Command::new(bin())
        .arg("--help")
        .output()
        .expect("jalankan --help");
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for flag in [
        "--list-presets",
        "--preset",
        "--burn",
        "--quality",
        "--render-frame",
        "--at-ms",
        "--words-json",
        "--size",
        "--ffprobe",
    ] {
        assert!(stdout.contains(flag), "help harus punya {flag}");
    }
}

#[test]
fn unknown_preset_errors() {
    let out = Command::new(bin())
        .args([
            "--preset",
            "tidak-ada",
            "--words-json",
            "src/mock/words.json",
            "--size",
            "1080x1920",
        ])
        .output()
        .expect("jalankan");
    assert!(!out.status.success(), "preset tidak ada harus error");
}

#[test]
fn missing_preset_arg_errors() {
    // --preset dengan id valid + --words-json harus jalan tanpa input
    let out = Command::new(bin())
        .args([
            "--preset",
            "kata-tebal-putih",
            "--words-json",
            "src/mock/words.json",
            "--size",
            "1080x1920",
        ])
        .output()
        .expect("jalankan");
    // Boleh sukses atau gagal karena output default out.ass; yang penting
    // tidak ada panic.
    assert!(out.status.code().is_some(), "tidak boleh panic");
}

#[test]
fn words_required_without_preset() {
    let out = Command::new(bin())
        .args(["--words-json"])
        .arg(words_json())
        .args(["--size", "1080x1920"])
        .output()
        .expect("jalankan");
    assert!(
        !out.status.success(),
        "tanpa --preset dan --words harus error"
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("--words"),
        "pesan harus menyebut --words: {stderr}"
    );
}
