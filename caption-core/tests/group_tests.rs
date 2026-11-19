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
