use caption_core::exporter::{to_json, to_srt, to_txt, to_vtt};
use caption_core::model::{Group, Word};

fn sample_groups() -> Vec<Group> {
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
                start_ms: 3661000,
                end_ms: 3662500,
            }],
        },
    ]
}

#[test]
fn srt_has_index_timestamp_and_text() {
    let srt = to_srt(&sample_groups());
    assert!(srt.starts_with("1\n00:00:00,120 --> 00:00:00,660\nhalo dunia\n"));
    assert!(srt.contains("2\n01:01:01,000 --> 01:01:02,500\ntes\n"));
}

#[test]
fn vtt_has_header_and_dot_separator() {
    let vtt = to_vtt(&sample_groups());
    assert!(vtt.starts_with("WEBVTT\n\n"));
    assert!(vtt.contains("00:00:00.120 --> 00:00:00.660\nhalo dunia"));
    assert!(!vtt.contains(','), "VTT tidak boleh pakai koma timestamp");
}

#[test]
fn txt_is_one_line_per_group() {
    let txt = to_txt(&sample_groups());
    assert_eq!(txt, "halo dunia\ntes\n");
}

#[test]
fn json_has_expected_shape() {
    let json = to_json(&sample_groups()).unwrap();
    assert!(json.contains("\"start_ms\": 120"));
    assert!(json.contains("\"end_ms\": 660"));
    assert!(json.contains("\"text\": \"halo dunia\""));
    assert!(json.contains("\"words\""));
}

#[test]
fn empty_groups_produce_no_events() {
    assert_eq!(to_srt(&[]), "");
    assert_eq!(to_vtt(&[]), "WEBVTT\n\n");
    assert_eq!(to_txt(&[]), "");
    assert_eq!(to_json(&[]).unwrap(), "[]");
}

#[test]
fn srt_rounds_not_truncates() {
    let g = vec![Group {
        words: vec![Word {
            text: "x".into(),
            start_ms: 1999,
            end_ms: 2500,
        }],
    }];
    let srt = to_srt(&g);
    assert!(srt.contains("00:00:01,999 --> 00:00:02,500"));
}
