use crate::model::{Group, Word, WordsPerLine};

pub fn group_words(words: &[Word], per_line: WordsPerLine) -> Vec<Group> {
    words
        .chunks(per_line.get())
        .map(|words| Group {
            words: words.to_vec(),
        })
        .collect()
}
