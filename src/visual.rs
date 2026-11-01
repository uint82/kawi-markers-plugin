use kawi_plugin_sdk::editor_signs;
use kawi_plugin_sdk::types::{Decoration, Highlight, Position, VirtualText};
use kawi_plugin_sdk::ui_statusline;

use crate::markers::{self, Counts, Marker};

const STATUSLINE_PRIORITY: u32 = 20;

pub fn decorations(
    markers: &[Marker],
    counts: &Counts,
    show_virtual_text: bool,
) -> Vec<Decoration> {
    let mut decorations = Vec::with_capacity(markers.len().saturating_mul(2).saturating_add(1));
    for marker in markers {
        push_marker(&mut decorations, marker);
    }
    if show_virtual_text {
        if let Some(marker) = markers.first() {
            decorations.push(virtual_text(marker, counts));
        }
    }
    decorations
}

pub fn signs(markers: &[Marker], show_signs: bool) -> Vec<editor_signs::Sign> {
    if !show_signs {
        return Vec::new();
    }
    markers::sign_lines(markers)
        .into_iter()
        .map(|(line, kind)| editor_signs::Sign {
            line,
            text: kind.sign().to_string(),
            group: kind.group().to_string(),
        })
        .collect()
}

pub fn statusline(counts: &Counts) -> Vec<ui_statusline::StatuslineSegment> {
    if counts.total() == 0 {
        return Vec::new();
    }
    vec![ui_statusline::StatuslineSegment {
        text: counts.status_text(),
        style: counts.style(),
        priority: STATUSLINE_PRIORITY,
    }]
}

fn push_marker(decorations: &mut Vec<Decoration>, marker: &Marker) {
    decorations.push(highlight(
        marker,
        marker.kind.group(),
        marker.badge_start,
        marker.badge_end,
    ));
    if marker.text_start < marker.text_end {
        decorations.push(highlight(
            marker,
            marker.kind.text_group(),
            marker.text_start,
            marker.text_end,
        ));
    }
}

fn highlight(marker: &Marker, group: &str, start: u32, end: u32) -> Decoration {
    Decoration::Highlight(Highlight {
        start: Position {
            line: marker.line,
            byte_column: start,
        },
        end: Position {
            line: marker.line,
            byte_column: end,
        },
        group: group.to_string(),
    })
}

fn virtual_text(marker: &Marker, counts: &Counts) -> Decoration {
    Decoration::VirtualText(VirtualText {
        line: marker.line,
        text: counts.message(),
        group: marker.kind.group().to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::markers::{scan, Kind};
    use kawi_plugin_sdk::types::DecorationStyle;

    fn groups(decorations: &[Decoration]) -> Vec<&str> {
        decorations
            .iter()
            .map(|decoration| match decoration {
                Decoration::Highlight(highlight) => highlight.group.as_str(),
                Decoration::VirtualText(text) => text.group.as_str(),
            })
            .collect()
    }

    #[test]
    fn builds_badge_and_text_highlights() {
        let markers = scan("TODO: fix");
        let decorations = decorations(&markers, &Counts::from_markers(&markers), true);
        assert_eq!(groups(&decorations), vec!["todo", "todo_text", "todo"]);
        match &decorations[0] {
            Decoration::Highlight(highlight) => {
                assert_eq!(highlight.start.byte_column, 0);
                assert_eq!(highlight.end.byte_column, 6);
            }
            Decoration::VirtualText(_) => panic!("expected highlight"),
        }
        match &decorations[1] {
            Decoration::Highlight(highlight) => {
                assert_eq!(highlight.start.byte_column, 6);
                assert_eq!(highlight.end.byte_column, 9);
            }
            Decoration::VirtualText(_) => panic!("expected highlight"),
        }
    }

    #[test]
    fn signs_use_the_badge_group() {
        let markers = scan("TODO FIXME");
        let signs = signs(&markers, true);
        assert_eq!(signs.len(), 1);
        assert_eq!(signs[0].group, "fixme");
        assert_eq!(signs[0].text, "!");
    }

    #[test]
    fn virtual_text_can_be_disabled() {
        let markers = scan("TODO: fix");
        let counts = Counts::from_markers(&markers);
        let decorations = decorations(&markers, &counts, false);
        assert_eq!(groups(&decorations), vec!["todo", "todo_text"]);
    }

    #[test]
    fn statusline_uses_counts() {
        let counts = Counts::from_markers(&scan("TODO"));
        let segments = statusline(&counts);
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "markers 1");
        assert_eq!(segments[0].style, DecorationStyle::Info);
    }

    #[test]
    fn kind_groups_are_stable() {
        assert_eq!(Kind::Todo.group(), "todo");
        assert_eq!(Kind::Todo.text_group(), "todo_text");
        assert_eq!(Kind::Fixme.group(), "fixme");
    }
}
