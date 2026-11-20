use crate::model::{RawToken, Word};

#[derive(Default)]
struct PendingWord {
    bytes: Vec<u8>,
    start_ms: Option<i64>,
    end_ms: Option<i64>,
}

pub fn merge_tokens_to_words(tokens: &[RawToken]) -> Vec<Word> {
    let mut pending = Vec::new();
    let mut current: Option<PendingWord> = None;

    for token in tokens.iter().filter(|token| !token.is_special) {
        let starts_word = token.text.chars().next().is_some_and(char::is_whitespace);
        let bytes = if token.bytes.is_empty() {
            token.text.as_bytes()
        } else {
            &token.bytes
        };
        let fragment = trim_ascii_whitespace(bytes);
        if fragment.is_empty() {
            continue;
        }

        if starts_word {
            if let Some(word) = current.take() {
                pending.push(word);
            }
            current = Some(PendingWord::default());
        } else if current.is_none() {
            current = Some(PendingWord::default());
        }

        let word = current.as_mut().expect("current word was initialized");
        word.bytes.extend_from_slice(fragment);
        if word.start_ms.is_none() && token.t0_ms >= 0 {
            word.start_ms = Some(token.t0_ms);
        }
        if token.t1_ms >= 0 {
            word.end_ms = Some(token.t1_ms);
        }
    }
    if let Some(word) = current {
        pending.push(word);
    }

    repair_timings(pending)
}

fn repair_timings(pending: Vec<PendingWord>) -> Vec<Word> {
    let mut words = Vec::with_capacity(pending.len());
    let mut previous_end = 0_i64;

    for (index, word) in pending.iter().enumerate() {
        let next_start = pending[index + 1..]
            .iter()
            .find_map(|next| next.start_ms.filter(|&start| start >= 0));
        let start = word
            .start_ms
            .filter(|&start| start >= 0)
            .unwrap_or(previous_end)
            .max(previous_end);
        let end = word
            .end_ms
            .filter(|&end| end > start)
            .or_else(|| next_start.filter(|&next| next > start))
            .unwrap_or(start.saturating_add(10));
        let end = end.max(start.saturating_add(10));

        words.push(Word {
            text: String::from_utf8_lossy(&word.bytes).into_owned(),
            start_ms: to_u32(start),
            end_ms: to_u32(end),
        });
        previous_end = end;
    }

    words
}

fn trim_ascii_whitespace(bytes: &[u8]) -> &[u8] {
    let start = bytes
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
        .unwrap_or(bytes.len());
    let end = bytes
        .iter()
        .rposition(|byte| !byte.is_ascii_whitespace())
        .map_or(start, |index| index + 1);
    &bytes[start..end]
}

fn to_u32(value: i64) -> u32 {
    value.clamp(0, i64::from(u32::MAX)) as u32
}
