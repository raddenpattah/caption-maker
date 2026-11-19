use caption_core::{group_words, render_ass, AssOptions, AssStyle, Group, Rgb, Word, WordsPerLine};

fn words_with_times(times: &[(u32, u32, &str)]) -> Vec<Word> {
    times
        .iter()
        .map(|&(start_ms, end_ms, text)| Word {
            text: text.to_owned(),
            start_ms,
            end_ms,
        })
        .collect()
}

fn options(highlight: bool) -> AssOptions {
    AssOptions {
        highlight,
        ..AssOptions::default()
    }
}

fn one_group(times: &[(u32, u32, &str)]) -> Vec<Group> {
    vec![Group {
        words: words_with_times(times),
    }]
}

#[test]
fn timestamp_format_basic() {
    let ass = render_ass(&one_group(&[(1_000, 3_725_450, "word")]), &options(false));
    assert!(ass.contains("Dialogue: 0,0:00:01.00,1:02:05.45,"));
}

#[test]
fn timestamp_rounds_not_truncates() {
    let ass = render_ass(&one_group(&[(1_005, 1_105, "word")]), &options(false));
    assert!(ass.contains("Dialogue: 0,0:00:01.01,0:00:01.11,"));
}

#[test]
fn header_contains_required_sections_and_event_columns() {
    let ass = render_ass(&[], &options(false));
    assert!(ass.contains("[Script Info]"));
    assert!(ass.contains("[V4+ Styles]"));
    assert!(ass.contains("[Events]"));
    assert!(ass.contains(
        "Format: Layer, Start, End, Style, Name, MarginL, MarginR, MarginV, Effect, Text"
    ));
}

#[test]
fn style_line_maps_colors_to_ass_bgr_order() {
    let mut options = options(false);
    options.style = AssStyle {
        primary: Rgb {
            r: 255,
            g: 255,
            b: 0,
        },
        secondary: Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
        outline_color: Rgb { r: 0, g: 0, b: 0 },
        ..AssStyle::default()
    };
    let ass = render_ass(&[], &options);
    assert!(ass.contains("Style: Default,Arial,64,&H0000FFFF,&H00FFFFFF,&H00000000,"));
}

#[test]
fn plain_mode_has_no_karaoke_tags() {
    let ass = render_ass(
        &one_group(&[(0, 100, "Halo"), (100, 200, "dunia")]),
        &options(false),
    );
    assert!(!ass.contains("\\k"));
}

#[test]
fn highlight_uses_exact_cumulative_word_durations() {
    let ass = render_ass(
        &[
            Group {
                words: words_with_times(&[(1_000, 1_500, "Halo"), (1_500, 2_200, "dunia,")]),
            },
            Group {
                words: words_with_times(&[(2_400, 3_000, "selamat"), (3_000, 3_600, "datang")]),
            },
            Group {
                words: words_with_times(&[(3_800, 3_950, "di"), (3_950, 4_500, "kanal")]),
            },
        ],
        &options(true),
    );
    assert!(ass.contains("{\\kf50}Halo {\\kf70}dunia,"));
    assert!(ass.contains("{\\kf60}selamat {\\kf60}datang"));
    assert!(ass.contains("{\\kf15}di {\\kf55}kanal"));
}

#[test]
fn highlight_rounding_does_not_accumulate_drift() {
    let ass = render_ass(
        &one_group(&[(0, 333, "satu"), (333, 666, "dua"), (666, 999, "tiga")]),
        &options(true),
    );
    assert!(ass.contains("{\\kf33}satu {\\kf34}dua {\\kf33}tiga"));
}

#[test]
fn highlight_inserts_gap_tag() {
    let ass = render_ass(
        &one_group(&[(1_000, 1_100, "awal"), (1_300, 1_600, "akhir")]),
        &options(true),
    );
    assert!(ass.contains("{\\kf10}awal {\\k20}{\\kf30}akhir"));
}

#[test]
fn braces_and_newlines_are_escaped() {
    let ass = render_ass(&one_group(&[(0, 100, "{halo}\ndunia")]), &options(false));
    assert!(ass.contains("(halo)\\Ndunia"));
}

#[test]
fn comma_in_text_is_preserved() {
    let ass = render_ass(&one_group(&[(0, 100, "dunia,")]), &options(false));
    assert!(ass.contains(",,dunia,"));
}

#[test]
fn dialogue_uses_first_start_and_last_end() {
    let ass = render_ass(
        &one_group(&[(1_000, 1_500, "a"), (1_700, 2_200, "b")]),
        &options(false),
    );
    assert!(ass.contains("Dialogue: 0,0:00:01.00,0:00:02.20,"));
}

#[test]
fn empty_groups_only_emit_header() {
    let ass = render_ass(&[], &options(false));
    assert!(!ass.contains("Dialogue:"));
}

#[test]
fn groups_from_grouping_api_render_in_word_order() {
    let words = words_with_times(&[(0, 100, "satu"), (100, 200, "dua"), (200, 300, "tiga")]);
    let groups = group_words(&words, WordsPerLine::Two);
    let ass = render_ass(&groups, &options(false));
    assert!(ass.contains(",,satu dua\nDialogue:"));
    assert!(ass.contains(",,tiga\n"));
}
