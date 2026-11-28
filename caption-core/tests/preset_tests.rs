use caption_core::preset::{
    builtin_presets, find_preset, validate_preset, Animation, Emphasis, PresetCategory, StylePreset,
};

#[test]
fn all_builtin_presets_valid() {
    let presets = builtin_presets().expect("preset bawaan harus valid");
    assert_eq!(presets.len(), 12, "harus ada 12 preset");
    for p in &presets {
        validate_preset(p).unwrap_or_else(|e| panic!("preset {}: {e}", p.id));
    }
}

#[test]
fn preset_ids_unique() {
    let presets = builtin_presets().unwrap();
    let mut ids: Vec<&str> = presets.iter().map(|p| p.id.as_str()).collect();
    ids.sort();
    let mut dedup = ids.clone();
    dedup.dedup();
    assert_eq!(ids, dedup, "ada id duplikat");
}

#[test]
fn categories_are_balanced() {
    let presets = builtin_presets().unwrap();
    let kata = presets
        .iter()
        .filter(|p| p.category == PresetCategory::KataPerKata)
        .count();
    let baris = presets
        .iter()
        .filter(|p| p.category == PresetCategory::BarisBergaya)
        .count();
    let tekan = presets
        .iter()
        .filter(|p| p.category == PresetCategory::PenekananKata)
        .count();
    assert_eq!((kata, baris, tekan), (4, 4, 4));
}

#[test]
fn find_preset_works() {
    let p = find_preset("tekan-warna-kuning").unwrap();
    assert_eq!(p.emphasis, Emphasis::ColorActive);
    assert!(find_preset("tidak-ada").is_err());
}

#[test]
fn rejects_bad_words_per_line() {
    let mut p = find_preset("kata-tebal-putih").unwrap();
    p.words_per_line = 4;
    let err = validate_preset(&p).unwrap_err();
    assert!(err.contains("words_per_line"), "pesan: {err}");
}

#[test]
fn rejects_bad_color() {
    let mut p = find_preset("kata-tebal-putih").unwrap();
    p.colors.text = "red".into();
    let err = validate_preset(&p).unwrap_err();
    assert!(err.contains("colors.text"), "pesan: {err}");
}

#[test]
fn rejects_bad_emphasis_scale() {
    let mut p = find_preset("tekan-besar").unwrap();
    p.emphasis_scale = None;
    let err = validate_preset(&p).unwrap_err();
    assert!(err.contains("emphasis_scale"), "pesan: {err}");
}

#[test]
fn rejects_bad_font_size() {
    let mut p = find_preset("kata-tebal-putih").unwrap();
    p.font.size = 0;
    let err = validate_preset(&p).unwrap_err();
    assert!(err.contains("font.size"), "pesan: {err}");
}

#[test]
fn serde_roundtrip_single_preset() {
    let p = find_preset("baris-neon-mint").unwrap();
    let json = serde_json::to_string(&p).unwrap();
    let back: StylePreset = serde_json::from_str(&json).unwrap();
    assert_eq!(p, back);
}

#[test]
fn json_uses_box_key_not_box_underscore() {
    let p = find_preset("kata-kotak-merah").unwrap();
    let json = serde_json::to_string(&p).unwrap();
    assert!(
        json.contains("\"box\":"),
        "JSON harus pakai key box, dapat: {json}"
    );
    assert!(!json.contains("\"box_\":"), "JSON tidak boleh pakai box_");
}

#[test]
fn animation_defaults_to_none_and_rejects_unknown_values() {
    let preset = find_preset("kata-bangers").unwrap();
    let mut value = serde_json::to_value(&preset).unwrap();
    value.as_object_mut().unwrap().remove("animation");
    let legacy: StylePreset = serde_json::from_value(value).unwrap();
    assert_eq!(legacy.animation, Animation::None);

    let mut value = serde_json::to_value(&preset).unwrap();
    value["animation"] = serde_json::json!("bounce");
    assert!(serde_json::from_value::<StylePreset>(value).is_err());
}

#[test]
fn only_kata_tebal_putih_has_pop_animation() {
    for preset in builtin_presets().unwrap() {
        assert_eq!(
            preset.animation,
            if preset.id == "kata-tebal-putih" {
                Animation::Pop
            } else {
                Animation::None
            },
            "{}",
            preset.id
        );
    }
}
