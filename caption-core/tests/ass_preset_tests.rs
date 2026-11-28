use caption_core::model::{Group, Word, WordsPerLine};
use caption_core::preset::{find_preset, Animation, Emphasis};
use caption_core::preset_render::{
    ass_color_from_hex, render_ass_with_preset, CaptionPosition, PresetOptions, VideoGeometry,
};
use caption_core::validate_ass;

fn words_3() -> Vec<Group> {
    vec![Group {
        words: vec![
            Word {
                text: "satu".into(),
                start_ms: 1000,
                end_ms: 1500,
            },
            Word {
                text: "dua".into(),
                start_ms: 1500,
                end_ms: 2100,
            },
            Word {
                text: "tiga".into(),
                start_ms: 2100,
                end_ms: 2800,
            },
        ],
    }]
}

fn words_5() -> Vec<Group> {
    vec![Group {
        words: vec![
            Word {
                text: "a".into(),
                start_ms: 0,
                end_ms: 500,
            },
            Word {
                text: "b".into(),
                start_ms: 500,
                end_ms: 1000,
            },
            Word {
                text: "c".into(),
                start_ms: 1000,
                end_ms: 1500,
            },
            Word {
                text: "d".into(),
                start_ms: 1500,
                end_ms: 2000,
            },
            Word {
                text: "e".into(),
                start_ms: 2000,
                end_ms: 2500,
            },
        ],
    }]
}

fn portrait() -> VideoGeometry {
    VideoGeometry {
        width: 1080,
        height: 1920,
    }
}
fn landscape() -> VideoGeometry {
    VideoGeometry {
        width: 1920,
        height: 1080,
    }
}
fn small_portrait() -> VideoGeometry {
    VideoGeometry {
        width: 720,
        height: 1280,
    }
}

fn dialogue_lines(ass: &str) -> Vec<&str> {
    ass.lines().filter(|l| l.starts_with("Dialogue:")).collect()
}

#[test]
fn all_emphasis_renders_valid_ass() {
    for id in [
        "kata-tebal-putih",
        "baris-isi-kuning",
        "tekan-warna-kuning",
        "tekan-garis-bawah",
        "tekan-besar",
        "baris-neon-mint",
    ] {
        let p = find_preset(id).unwrap();
        let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
        let v = validate_ass(&ass);
        assert!(
            v.is_valid(),
            "preset {id} menghasilkan ASS tidak valid: {:?}",
            v.errors
        );
    }
}

#[test]
fn color_active_produces_one_event_per_word_contiguous() {
    let p = find_preset("tekan-warna-kuning").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    let events = dialogue_lines(&ass);
    assert_eq!(events.len(), 3, "harus 3 event (1 per kata)");
    // Kontigu: event[i].end == event[i+1].start, tanpa overlap
    let parse = |line: &str| {
        let parts: Vec<&str> = line.splitn(10, ',').collect();
        (parts[1].trim().to_string(), parts[2].trim().to_string())
    };
    let times: Vec<_> = events.iter().map(|e| parse(e)).collect();
    for w in times.windows(2) {
        assert_eq!(w[0].1, w[1].0, "event harus kontigu");
    }
    assert_eq!(times[0].0, "0:00:01.00");
    assert_eq!(times[2].1, "0:00:02.80");
}

#[test]
fn color_active_highlights_only_active_word() {
    let p = find_preset("tekan-warna-kuning").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    let events = dialogue_lines(&ass);
    // Setiap event menampilkan SELURUH teks grup dengan kata aktif di-override.
    assert!(
        events[0].contains("{\\c&H004DE1FF&}SATU"),
        "dapat: {}",
        events[0]
    );
    assert_eq!(
        events[0].matches("\\c").count(),
        1,
        "hanya kata aktif yang diwarnai: {}",
        events[0]
    );
    assert!(
        events[1].contains("{\\c&H004DE1FF&}DUA"),
        "dapat: {}",
        events[1]
    );
}

#[test]
fn underline_active_has_u1_tag() {
    let p = find_preset("tekan-garis-bawah").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    for line in dialogue_lines(&ass) {
        assert!(line.contains("\\u1"), "event harus punya \\u1: {line}");
    }
}

#[test]
fn scale_active_has_fscx_fscy() {
    let p = find_preset("tekan-besar").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    for line in dialogue_lines(&ass) {
        assert!(
            line.contains("\\fscx118"),
            "event harus punya \\fscx118: {line}"
        );
        assert!(
            line.contains("\\fscy118"),
            "event harus punya \\fscy118: {line}"
        );
    }
}

#[test]
fn fill_mode_uses_kf_tags() {
    let p = find_preset("baris-isi-kuning").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    let events = dialogue_lines(&ass);
    assert_eq!(events.len(), 1, "fill = 1 event per grup");
    assert!(events[0].contains("\\kf"), "harus ada \\kf: {}", events[0]);
    assert!(
        !events[0].contains("\\c&H"),
        "fill tidak override warna per kata"
    );
}

#[test]
fn none_mode_has_no_override_tags() {
    let p = find_preset("kata-tebal-putih").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    let events = dialogue_lines(&ass);
    assert_eq!(events.len(), 3, "words_per_line=1 → 3 event");
    for e in &events {
        assert!(!e.contains("\\kf"), "none tidak pakai \\kf: {e}");
        assert!(!e.contains("\\c&H"), "none tidak override warna: {e}");
    }
}

#[test]
fn glow_adds_lower_layer_events() {
    let p = find_preset("baris-neon-mint").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    let events = dialogue_lines(&ass);
    let glow_event = events
        .iter()
        .find(|e| e.starts_with("Dialogue: 0,"))
        .unwrap();
    let layer0 = events
        .iter()
        .filter(|e| e.starts_with("Dialogue: 0,"))
        .count();
    let layer1 = events
        .iter()
        .filter(|e| e.starts_with("Dialogue: 1,"))
        .count();
    assert_eq!(layer0, 1, "glow = 1 event layer 0");
    assert_eq!(layer1, 1, "teks utama = 1 event layer 1");
    assert!(glow_event.contains("\\bord12\\blur12"));
    assert!(glow_event.contains("\\c&H00C0FF5C&\\3c&H00C0FF5C&"));
}

#[test]
fn glow_radius_scales_for_720x1280_and_sets_secondary_color() {
    let p = find_preset("baris-neon-mint").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, small_portrait(), &PresetOptions::default());
    let glow_event = dialogue_lines(&ass)
        .into_iter()
        .find(|line| line.starts_with("Dialogue: 0,"))
        .unwrap();

    assert!(glow_event.contains("\\bord8\\blur8"), "{glow_event}");
    assert!(
        glow_event.contains("\\c&H00C0FF5C&\\3c&H00C0FF5C&"),
        "{glow_event}"
    );
}

#[test]
fn box_preset_sets_border_style_3() {
    let p = find_preset("kata-kotak-merah").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    let style = ass
        .lines()
        .find(|l| l.starts_with("Style: Default,"))
        .unwrap();
    let parts: Vec<&str> = style.splitn(23, ',').collect();
    assert_eq!(parts[15], "3", "BorderStyle harus 3 untuk kotak");
}

#[test]
fn kata_kotak_merah_uses_independent_vertical_box_padding() {
    let preset = find_preset("kata-kotak-merah").unwrap();
    let ass = render_ass_with_preset(&words_3(), &preset, portrait(), &PresetOptions::default());
    let style = ass
        .lines()
        .find(|line| line.starts_with("Style: Default,"))
        .unwrap();
    let fields: Vec<_> = style.splitn(23, ',').collect();

    assert_eq!(fields[16], "22", "padding horizontal dipertahankan");
    for event in dialogue_lines(&ass) {
        assert!(
            event.contains(r"{\ybord8}"),
            "setiap event harus memakai padding vertikal 8: {event}"
        );
    }
}

#[test]
fn no_box_preset_sets_border_style_1() {
    let p = find_preset("kata-tebal-putih").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    let style = ass
        .lines()
        .find(|l| l.starts_with("Style: Default,"))
        .unwrap();
    let parts: Vec<&str> = style.splitn(23, ',').collect();
    assert_eq!(parts[15], "1", "BorderStyle harus 1 tanpa kotak");
}

#[test]
fn uppercase_is_applied() {
    let p = find_preset("kata-tebal-putih").unwrap(); // uppercase=true
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    assert!(ass.contains("SATU"), "harus uppercase: {ass}");
    assert!(!ass.contains("satu"), "tidak boleh lowercase");
}

#[test]
fn scaling_1080x1920_vs_720x1280() {
    let p = find_preset("kata-tebal-putih").unwrap();
    let big = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    let small = render_ass_with_preset(&words_3(), &p, small_portrait(), &PresetOptions::default());
    let parse_font = |ass: &str| -> u32 {
        let style = ass
            .lines()
            .find(|l| l.starts_with("Style: Default,"))
            .unwrap();
        let parts: Vec<&str> = style.splitn(23, ',').collect();
        parts[2].parse().unwrap()
    };
    let f_big = parse_font(&big);
    let f_small = parse_font(&small);
    // 720/1080 = 0.666..., font 96*0.666 ≈ 64
    assert_eq!(f_big, 96, "font di 1080 harus 96");
    assert_eq!(f_small, 64, "font di 720 harus 64 (dibulatkan)");
}

#[test]
fn landscape_uses_8_percent_margin_v() {
    let p = find_preset("kata-tebal-putih").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, landscape(), &PresetOptions::default());
    let style = ass
        .lines()
        .find(|l| l.starts_with("Style: Default,"))
        .unwrap();
    let parts: Vec<&str> = style.splitn(23, ',').collect();
    let margin_v: u32 = parts[21].parse().unwrap();
    assert_eq!(margin_v, 86, "8% dari 1080 ≈ 86");
}

#[test]
fn ass_color_rrggbb() {
    assert_eq!(ass_color_from_hex("#FFD400").unwrap(), "&H0000D4FF&");
}

#[test]
fn ass_color_rrggbbaa_alpha_inverted() {
    // Input #RRGGBBAA: AA=FF (opak) → ASS 00 (opak)
    assert_eq!(ass_color_from_hex("#FF0000FF").unwrap(), "&H000000FF&");
    // AA=80 (semi) → ASS 7F
    assert_eq!(ass_color_from_hex("#FF000080").unwrap(), "&H7F0000FF&");
    // AA=00 (transparan) → ASS FF
    assert_eq!(ass_color_from_hex("#FF000000").unwrap(), "&HFF0000FF&");
}

#[test]
fn words_per_line_override_works() {
    let p = find_preset("kata-tebal-putih").unwrap(); // default 1
    let opts = PresetOptions {
        words_per_line: Some(WordsPerLine::Three),
        ..PresetOptions::default()
    };
    let ass = render_ass_with_preset(&words_5(), &p, portrait(), &opts);
    let events = dialogue_lines(&ass);
    // 5 kata → grup 3 + 2 = 2 event
    assert_eq!(events.len(), 2, "override words_per_line=3 → 2 event");
}

#[test]
fn custom_position_applies_to_glow_and_primary_events_at_playres_resolution() {
    let preset = find_preset("baris-neon-mint").unwrap();
    let geometry = small_portrait();
    let options = PresetOptions {
        caption_position: Some(CaptionPosition {
            x_percent: 25.0,
            y_percent: 75.0,
        }),
        ..PresetOptions::default()
    };

    let ass = render_ass_with_preset(&words_3(), &preset, geometry, &options);
    let dialogues = dialogue_lines(&ass);
    assert!(dialogues
        .iter()
        .any(|line| line.starts_with("Dialogue: 0,") && line.contains("\\pos(180,960)")));
    assert!(dialogues
        .iter()
        .any(|line| line.starts_with("Dialogue: 1,") && line.contains("\\pos(180,960)")));
    assert!(dialogues
        .iter()
        .all(|line| line.contains("{\\an5\\pos(180,960)}")));
}

#[test]
fn playres_matches_geometry() {
    let p = find_preset("kata-tebal-putih").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    assert!(ass.contains("PlayResX: 1080"));
    assert!(ass.contains("PlayResY: 1920"));
}

#[test]
fn fontname_matches_preset() {
    let p = find_preset("kata-anton-kuning").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    let style = ass
        .lines()
        .find(|l| l.starts_with("Style: Default,"))
        .unwrap();
    assert!(style.contains(",Anton,"), "Fontname harus Anton: {style}");
}

#[test]
fn pop_events_start_with_exact_tags_and_validate() {
    let preset = find_preset("kata-tebal-putih").unwrap();
    let ass = render_ass_with_preset(&words_3(), &preset, portrait(), &PresetOptions::default());
    let expected = r"{\fscx85\fscy85\t(0,90,\fscx108\fscy108)\t(90,160,\fscx100\fscy100)}";
    let events = dialogue_lines(&ass);
    assert_eq!(events.len(), 3);
    for event in events {
        let text = event.splitn(10, ',').nth(9).unwrap();
        assert!(
            text.starts_with(expected),
            "pop harus menjadi blok override pertama: {event}"
        );
    }
    assert!(validate_ass(&ass).is_valid());
}

#[test]
fn kata_kotak_merah_pop_and_box_shadow_are_rendered() {
    let preset = find_preset("kata-kotak-merah").unwrap();
    let ass = render_ass_with_preset(&words_3(), &preset, portrait(), &PresetOptions::default());
    let style = ass
        .lines()
        .find(|line| line.starts_with("Style: Default,"))
        .unwrap();
    let fields: Vec<_> = style.splitn(23, ',').collect();
    assert_eq!(fields[15], "3", "kotak harus memakai BorderStyle 3");
    assert_eq!(fields[17], "6", "Shadow harus 6");
    assert_eq!(
        fields[6],
        ass_color_from_hex("#000000A0").unwrap(),
        "BackColour harus berasal dari warna shadow preset"
    );

    let expected_pop = r"{\fscx85\fscy85\t(0,90,\fscx108\fscy108)\t(90,160,\fscx100\fscy100)}";
    let events = dialogue_lines(&ass);
    assert_eq!(events.len(), 3);
    for event in events {
        let text = event.splitn(10, ',').nth(9).unwrap();
        assert!(
            text.starts_with(expected_pop),
            "setiap event kotak merah harus dimulai dengan tag pop: {event}"
        );
    }
}

#[test]
fn pop_tag_uses_event_duration_and_skips_short_events() {
    let preset = find_preset("kata-tebal-putih").unwrap();
    let render_duration = |duration_ms: u32| {
        let group = Group {
            words: vec![Word {
                text: "pop".into(),
                start_ms: 1000,
                end_ms: 1000 + duration_ms,
            }],
        };
        let ass = render_ass_with_preset(&[group], &preset, portrait(), &PresetOptions::default());
        dialogue_lines(&ass)[0].to_owned()
    };

    let hundred_ms = render_duration(100);
    assert!(hundred_ms
        .contains(r"{\fscx85\fscy85\t(0,56,\fscx108\fscy108)\t(56,100,\fscx100\fscy100)}"));
    let thirty_ms = render_duration(30);
    assert!(!thirty_ms.contains(r"\fscx85"));
}

#[test]
fn pop_is_disabled_for_non_pop_mult_word_and_scale_active_cases() {
    let words_two = vec![Group {
        words: vec![
            Word {
                text: "satu".into(),
                start_ms: 1000,
                end_ms: 1500,
            },
            Word {
                text: "dua".into(),
                start_ms: 1500,
                end_ms: 2100,
            },
        ],
    }];

    let mut no_animation = find_preset("kata-bangers").unwrap();
    no_animation.words_per_line = 1;
    let ass = render_ass_with_preset(
        &words_two,
        &no_animation,
        portrait(),
        &PresetOptions::default(),
    );
    assert!(dialogue_lines(&ass)
        .iter()
        .all(|event| !event.contains(r"\fscx85")));

    let pop = find_preset("kata-tebal-putih").unwrap();
    let override_two = PresetOptions {
        words_per_line: Some(WordsPerLine::Two),
        ..PresetOptions::default()
    };
    let ass = render_ass_with_preset(&words_two, &pop, portrait(), &override_two);
    assert!(dialogue_lines(&ass)
        .iter()
        .all(|event| !event.contains(r"\fscx85")));

    let mut scale_active = find_preset("tekan-besar").unwrap();
    scale_active.animation = Animation::Pop;
    let ass = render_ass_with_preset(
        &words_two,
        &scale_active,
        portrait(),
        &PresetOptions {
            words_per_line: Some(WordsPerLine::One),
            ..PresetOptions::default()
        },
    );
    assert!(dialogue_lines(&ass)
        .iter()
        .all(|event| !event.contains(r"\fscx85")));
    assert_eq!(scale_active.emphasis, Emphasis::ScaleActive);
}

#[test]
fn glow_and_primary_events_both_receive_pop_tags() {
    let mut preset = find_preset("baris-neon-mint").unwrap();
    preset.animation = Animation::Pop;
    let ass = render_ass_with_preset(
        &words_3(),
        &preset,
        portrait(),
        &PresetOptions {
            words_per_line: Some(WordsPerLine::One),
            ..PresetOptions::default()
        },
    );
    let expected_start = r"{\fscx85\fscy85\t(0,90,\fscx108\fscy108)\t(90,160,\fscx100\fscy100)}";
    let events = dialogue_lines(&ass);
    let layer0: Vec<_> = events
        .iter()
        .filter(|event| event.starts_with("Dialogue: 0,"))
        .collect();
    let layer1: Vec<_> = events
        .iter()
        .filter(|event| event.starts_with("Dialogue: 1,"))
        .collect();
    assert_eq!(layer0.len(), 3);
    assert_eq!(layer1.len(), 3);
    for event in layer0.into_iter().chain(layer1) {
        assert!(
            event
                .splitn(10, ',')
                .nth(9)
                .unwrap()
                .starts_with(expected_start),
            "tag pop harus mendahului tag layer lain: {event}"
        );
    }
}

#[test]
fn golden_preset_tekan_warna_kuning() {
    let p = find_preset("tekan-warna-kuning").unwrap();
    let ass = render_ass_with_preset(&words_3(), &p, portrait(), &PresetOptions::default());
    assert!(ass.starts_with("[Script Info]\n"));
    assert!(ass.contains("[V4+ Styles]"));
    assert!(ass.contains("[Events]"));
    let events = dialogue_lines(&ass);
    assert_eq!(events.len(), 3);
    assert_eq!(
        events[0],
        "Dialogue: 1,0:00:01.00,0:00:01.50,Default,,0,0,0,,{\\c&H004DE1FF&}SATU DUA"
    );
}
