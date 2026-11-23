use caption_core::{group_words, Group, Word, WordsPerLine};

fn words(count: usize) -> Vec<Word> {
    (0..count)
        .map(|index| Word {
            text: format!("kata{index}"),
            start_ms: (index * 100) as u32,
            end_ms: (index * 100 + 80) as u32,
        })
        .collect()
}

#[test]
fn seven_words_are_grouped_with_remainders_for_each_size() {
    let words = words(7);
    for (per_line, sizes) in [
        (WordsPerLine::One, vec![1, 1, 1, 1, 1, 1, 1]),
        (WordsPerLine::Two, vec![2, 2, 2, 1]),
        (WordsPerLine::Three, vec![3, 3, 1]),
    ] {
        let groups = group_words(&words, per_line);
        assert_eq!(
            groups
                .iter()
                .map(|group| group.words.len())
                .collect::<Vec<_>>(),
            sizes
        );
        assert_eq!(
            groups
                .into_iter()
                .flat_map(|group| group.words)
                .collect::<Vec<_>>(),
            words
        );
    }
}

#[test]
fn empty_input_returns_no_groups() {
    assert!(group_words(&[], WordsPerLine::Two).is_empty());
}

#[test]
fn fewer_words_than_group_size_stays_in_one_group() {
    assert_eq!(
        group_words(&words(2), WordsPerLine::Three),
        vec![Group { words: words(2) }]
    );
}

fn sentence_words(texts: &[&str], starts: &[u32]) -> Vec<Word> {
    texts
        .iter()
        .enumerate()
        .map(|(index, text)| Word {
            text: (*text).to_string(),
            start_ms: starts[index],
            end_ms: starts[index].saturating_add(100),
        })
        .collect()
}

#[test]
fn sentence_group_ends_at_punctuation() {
    let words = sentence_words(
        &["Halo.", "Apa", "kabarmu?", "Baik", "!"],
        &[0, 100, 200, 300, 400],
    );
    let groups = group_words(&words, WordsPerLine::Sentence);
    assert_eq!(
        groups
            .iter()
            .map(|group| group.words.len())
            .collect::<Vec<_>>(),
        vec![1, 2, 2]
    );
}

#[test]
fn sentence_group_ends_at_700_ms_gap() {
    let words = sentence_words(&["satu", "dua", "tiga"], &[0, 100, 900]);
    let groups = group_words(&words, WordsPerLine::Sentence);
    assert_eq!(
        groups
            .iter()
            .map(|group| group.words.len())
            .collect::<Vec<_>>(),
        vec![2, 1]
    );
}

#[test]
fn sentence_group_has_eight_word_limit() {
    let words = words(10);
    let groups = group_words(&words, WordsPerLine::Sentence);
    assert_eq!(
        groups
            .iter()
            .map(|group| group.words.len())
            .collect::<Vec<_>>(),
        vec![8, 2]
    );
}

#[test]
fn sentence_group_keeps_trailing_words() {
    let words = sentence_words(&["satu", "dua", "tiga"], &[0, 100, 200]);
    let groups = group_words(&words, WordsPerLine::Sentence);
    assert_eq!(groups, vec![Group { words }]);
}

#[test]
fn sentence_empty_input_returns_no_groups() {
    assert!(group_words(&[], WordsPerLine::Sentence).is_empty());
}
