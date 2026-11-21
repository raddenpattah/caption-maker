use caption_core::model::{AssOptions, Group, Word};
use caption_core::{render_ass, validate_ass};

fn sample() -> Vec<Group> {
    vec![
        Group {
            words: vec![
                Word {
                    text: "halo".into(),
                    start_ms: 120,
                    end_ms: 360,
                },
                Word {
                    text: "dunia".into(),
                    start_ms: 390,
                    end_ms: 660,
                },
            ],
        },
        Group {
            words: vec![Word {
                text: "tes".into(),
                start_ms: 770,
                end_ms: 1940,
            }],
        },
    ]
}

#[test]
fn rendered_ass_is_valid() {
    let ass = render_ass(&sample(), &AssOptions::default());
    let v = validate_ass(&ass);
    assert!(v.is_valid(), "errors: {:?}", v.errors);
}

#[test]
fn rendered_ass_with_highlight_is_valid() {
    let opts = AssOptions {
        highlight: true,
        ..AssOptions::default()
    };
    let ass = render_ass(&sample(), &opts);
    assert!(validate_ass(&ass).is_valid());
}

#[test]
fn missing_script_info_is_detected() {
    let bad = "[V4+ Styles]\nStyle: Default,Arial,64,&H00FFFFFF,&H00FFFFFF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3,1,2,40,40,60,1\n[Events]\nFormat: Layer,Start,End,Style,Name,MarginL,MarginR,MarginV,Effect,Text\n";
    let v = validate_ass(bad);
    assert!(!v.is_valid());
    assert!(v.errors.iter().any(|e| e.contains("[Script Info]")));
}

#[test]
fn malformed_dialogue_field_count_is_detected() {
    let bad = "[Script Info]\nPlayResX: 1920\nPlayResY: 1080\n\n[V4+ Styles]\nStyle: Default,Arial,64,&H00FFFFFF,&H00FFFFFF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3,1,2,40,40,60,1\n\n[Events]\nDialogue: 0,0:00:00.12,0:00:00.74\n";
    let v = validate_ass(bad);
    assert!(!v.is_valid());
    assert!(v.errors.iter().any(|e| e.contains("field")));
}

#[test]
fn start_after_end_is_detected() {
    let bad = "[Script Info]\nPlayResX: 1920\nPlayResY: 1080\n\n[V4+ Styles]\nStyle: Default,Arial,64,&H00FFFFFF,&H00FFFFFF,&H00000000,&H80000000,-1,0,0,0,100,100,0,0,1,3,1,2,40,40,60,1\n\n[Events]\nDialogue: 0,0:00:02.00,0:00:01.00,Default,,0,0,0,,x\n";
    let v = validate_ass(bad);
    assert!(!v.is_valid());
    assert!(v.errors.iter().any(|e| e.contains("Start >= End")));
}

#[test]
fn empty_input_reports_missing_sections() {
    let v = validate_ass("");
    assert!(!v.is_valid());
    assert!(v.errors.iter().any(|e| e.contains("[Script Info]")));
    assert!(v.errors.iter().any(|e| e.contains("[V4+ Styles]")));
    assert!(v.errors.iter().any(|e| e.contains("[Events]")));
}

#[test]
fn text_with_commas_is_accepted() {
    // Koma di teks boleh; splitn(10) melindungi field terakhir.
    let mut groups = sample();
    groups[0].words[0].text = "halo, dunia".into();
    let ass = render_ass(&groups, &AssOptions::default());
    // Validator tidak boleh salah hitung field gara-gara koma di teks.
    assert!(validate_ass(&ass).is_valid());
}
