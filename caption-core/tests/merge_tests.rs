use caption_core::{merge_tokens_to_words, RawToken, Word};

fn token(text: &str, t0_ms: i64, t1_ms: i64, is_special: bool) -> RawToken {
    RawToken {
        text: text.to_owned(),
        t0_ms,
        t1_ms,
        id: 1,
        p: 0.9,
        is_special,
    }
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
