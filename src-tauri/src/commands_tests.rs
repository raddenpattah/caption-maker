//! Tes unit untuk `commands.rs` — logika murni tanpa runtime Tauri.

use crate::commands::*;

#[test]
fn list_models_has_3_models() {
    let models = list_models_impl();
    assert_eq!(models.len(), 3);
    assert!(models.iter().any(|m| m.id == "small" && m.installed));
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
        words_per_line: Some(1),
    };
    let res = render_ass_impl(&req).unwrap();
    assert!(res.ass.contains("[Script Info]"));
    assert!(res.ass.contains("PlayResX: 1080"));
    assert!(res.ass.contains("PlayResY: 1920"));
    assert!(res.ass.contains("HALO"), "harus uppercase");
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
        words_per_line: Some(5),
    };
    assert!(render_ass_impl(&req).is_err());
}
