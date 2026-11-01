use super::*;

#[test]
fn finds_markers_with_utf8_byte_columns() {
    let markers = scan("é TODO and FIXME\nNOTE");
    assert_eq!(markers.len(), 3);
    assert_eq!(
        markers[0],
        Marker {
            line: 0,
            badge_start: 3,
            badge_end: 7,
            text_start: 8,
            text_end: 17,
            kind: Kind::Todo,
        }
    );
    assert_eq!(
        markers[1],
        Marker {
            line: 0,
            badge_start: 12,
            badge_end: 17,
            text_start: 17,
            text_end: 17,
            kind: Kind::Fixme,
        }
    );
    assert_eq!(
        markers[2],
        Marker {
            line: 1,
            badge_start: 0,
            badge_end: 4,
            text_start: 4,
            text_end: 4,
            kind: Kind::Note,
        }
    );
}

#[test]
fn ignores_embedded_and_lowercase_words() {
    assert!(scan("TODOLIST todo Todo").is_empty());
}

#[test]
fn accepts_punctuation_boundaries() {
    assert_eq!(scan("TODO: fix").len(), 1);
    assert_eq!(scan("(FIXME)").len(), 1);
}

#[test]
fn colon_adds_badge_padding_and_a_text_span() {
    let markers = scan("TODO: fix");
    assert_eq!(
        markers[0],
        Marker {
            line: 0,
            badge_start: 0,
            badge_end: 6,
            text_start: 6,
            text_end: 9,
            kind: Kind::Todo,
        }
    );
}

#[test]
fn badge_padding_includes_one_surrounding_space() {
    let markers = scan(" TODO:  fix");
    assert_eq!(
        markers[0],
        Marker {
            line: 0,
            badge_start: 0,
            badge_end: 7,
            text_start: 8,
            text_end: 11,
            kind: Kind::Todo,
        }
    );
}

#[test]
fn colon_without_text_has_an_empty_text_span() {
    let markers = scan("TODO:");
    assert_eq!(markers[0].badge_end, 5);
    assert_eq!(markers[0].text_start, 5);
    assert_eq!(markers[0].text_end, 5);
}

#[test]
fn markers_without_a_colon_keep_the_keyword_badge() {
    let markers = scan("TODO fix");
    assert_eq!(markers[0].badge_start, 0);
    assert_eq!(markers[0].badge_end, 4);
    assert_eq!(markers[0].text_start, 5);
    assert_eq!(markers[0].text_end, 8);
}

#[test]
fn colon_may_be_separated_by_spaces() {
    let markers = scan("TODO : fix");
    assert_eq!(markers[0].badge_start, 0);
    assert_eq!(markers[0].badge_end, 7);
    assert_eq!(markers[0].text_start, 7);
    assert_eq!(markers[0].text_end, 10);
}

#[test]
fn utf8_prefixes_shift_badge_and_text_spans() {
    let markers = scan("é TODO: fix");
    assert_eq!(markers[0].badge_start, 2);
    assert_eq!(markers[0].badge_end, 9);
    assert_eq!(markers[0].text_start, 9);
    assert_eq!(markers[0].text_end, 12);
}

#[test]
fn multiple_markers_on_a_line_are_sorted() {
    let markers = scan("TODO: one FIXME: two");
    assert_eq!(markers.len(), 2);
    assert_eq!(markers[0].kind, Kind::Todo);
    assert_eq!(markers[1].kind, Kind::Fixme);
    assert_eq!(markers[0].text_end, 20);
    assert_eq!(markers[1].badge_start, 9);
}

#[test]
fn sign_lines_keep_the_highest_severity() {
    let markers = scan("TODO FIXME\nNOTE");
    assert_eq!(
        sign_lines(&markers),
        vec![(0, Kind::Fixme), (1, Kind::Note)]
    );
}

#[test]
fn counts_summarize_each_kind() {
    let counts = Counts::from_markers(&scan("TODO TODO FIXME NOTE"));
    assert_eq!(counts.total(), 4);
    assert_eq!(counts.message(), "4 markers: TODO 2, FIXME 1, NOTE 1");
    assert_eq!(counts.status_text(), "markers 4");
    assert_eq!(counts.style(), DecorationStyle::Error);
}

#[test]
fn empty_counts_are_muted() {
    let counts = Counts::default();
    assert_eq!(counts.message(), "no markers");
    assert_eq!(counts.style(), DecorationStyle::Muted);
}
