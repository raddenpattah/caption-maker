use crate::model::{Group, Word, WordsPerLine};

const SENTENCE_MAX_WORDS: usize = 8;
const SENTENCE_MIN_GAP_MS: u32 = 700;

pub fn group_words(words: &[Word], per_line: WordsPerLine) -> Vec<Group> {
    match per_line {
        WordsPerLine::One | WordsPerLine::Two | WordsPerLine::Three => words
            .chunks(per_line.get())
            .map(|words| Group {
                words: words.to_vec(),
            })
            .collect(),
        WordsPerLine::Sentence => {
            let mut groups: Vec<Group> = Vec::new();
            let mut current: Vec<Word> = Vec::new();

            for (index, word) in words.iter().enumerate() {
                current.push(word.clone());

                let ends_with_punctuation = word.text.trim_end().ends_with(['.', '?', '!']);
                let has_large_gap = words.get(index + 1).is_some_and(|next| {
                    next.start_ms.saturating_sub(word.end_ms) >= SENTENCE_MIN_GAP_MS
                });
                let reaches_max_words = current.len() >= SENTENCE_MAX_WORDS;

                if ends_with_punctuation || has_large_gap || reaches_max_words {
                    groups.push(Group {
                        words: std::mem::take(&mut current),
                    });
                }
            }

            if !current.is_empty() {
                groups.push(Group { words: current });
            }

            groups
        }
    }
}
