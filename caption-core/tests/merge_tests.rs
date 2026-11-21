use caption_core::{merge_tokens_to_words, RawToken, Word};

fn token(text: &str, t0_ms: i64, t1_ms: i64, is_special: bool) -> RawToken {
    RawToken {
        text: text.to_owned(),
        bytes: Vec::new(),
        t0_ms,
        t1_ms,
        id: 1,
        p: 0.9,
        is_special,
    }
}

#[test]
fn split_utf8_bytes_are_decoded_after_merging() {
    let tokens = [
        RawToken {
            text: String::from_utf8_lossy(&[0xC3]).into_owned(),
            bytes: vec![0xC3],
            t0_ms: 100,
            t1_ms: 120,
            id: 1,
            p: 0.9,
            is_special: false,
        },
        RawToken {
            text: String::from_utf8_lossy(&[0xA9]).into_owned(),
            bytes: vec![0xA9],
            t0_ms: 120,
            t1_ms: 150,
            id: 2,
            p: 0.9,
            is_special: false,
        },
    ];

    let words = merge_tokens_to_words(&tokens);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].text, "é");
    assert!(!words[0].text.contains('\u{FFFD}'));
}

#[test]
fn bpe_fragments_joined() {
    let tokens = [
        token(" Mul", 100, 120, false),
        token("do", 120, 150, false),
        token("on,", 150, 200, false),
    ];

    assert_eq!(
        merge_tokens_to_words(&tokens),
        vec![Word {
            text: "Muldoon,".to_owned(),
            start_ms: 100,
            end_ms: 200
        }]
    );
}

#[test]
fn punctuation_attached_and_spacing_starts_words() {
    let tokens = [
        token("Halo", 0, 100, false),
        token(" dunia,", 100, 200, false),
        token(" ya!", 200, 300, false),
    ];

    assert_eq!(
        merge_tokens_to_words(&tokens)
            .into_iter()
            .map(|word| word.text)
            .collect::<Vec<_>>(),
        ["Halo", "dunia,", "ya!"]
    );
}

#[test]
fn special_tokens_dropped() {
    let tokens = [
        token(" Halo", 0, 100, false),
        token("<|endoftext|>", 100, 100, true),
        token(" dunia", 100, 200, false),
    ];

    assert_eq!(
        merge_tokens_to_words(&tokens)
            .into_iter()
            .map(|word| word.text)
            .collect::<Vec<_>>(),
        ["Halo", "dunia"]
    );
}

#[test]
fn invalid_timestamps_are_repaired() {
    let tokens = [
        token(" pertama", -1, -1, false),
        token(" kedua", -1, 150, false),
        token(" ketiga", 120, 120, false),
    ];

    assert_eq!(
        merge_tokens_to_words(&tokens),
        vec![
            Word {
                text: "pertama".to_owned(),
                start_ms: 0,
                end_ms: 120,
            },
            Word {
                text: "kedua".to_owned(),
                start_ms: 120,
                end_ms: 150,
            },
            Word {
                text: "ketiga".to_owned(),
                start_ms: 150,
                end_ms: 160,
            },
        ]
    );
}

#[test]
fn overlapping_words_are_shifted_forward() {
    let tokens = [
        token(" satu", 0, 120, false),
        token(" dua", 100, 200, false),
    ];

    let words = merge_tokens_to_words(&tokens);
    assert_eq!(words[0].end_ms, 120);
    assert_eq!(words[1].start_ms, 120);
}

#[test]
fn empty_input_returns_no_words() {
    assert!(merge_tokens_to_words(&[]).is_empty());
}

#[test]
fn real_sample_fixture_merges_into_expected_word_count() {
    let tokens: Vec<RawToken> =
        serde_json::from_str(include_str!("fixtures/tokens_id_sample.json"))
            .expect("parse the captured Whisper token fixture");
    assert_eq!(merge_tokens_to_words(&tokens).len(), 48);
}

#[test]
fn merged_words_serialize_as_pretty_word_array() {
    let words = merge_tokens_to_words(&[token(" Halo", 100, 250, false)]);
    let serialized = serde_json::to_string_pretty(&words).expect("serialize merged words");

    assert_eq!(
        serialized,
        "[\n  {\n    \"text\": \"Halo\",\n    \"start_ms\": 100,\n    \"end_ms\": 250\n  }\n]"
    );
}
