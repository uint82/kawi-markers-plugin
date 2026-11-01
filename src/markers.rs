use std::collections::BTreeMap;

use kawi_plugin_sdk::types::DecorationStyle;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Note,
    Xxx,
    Todo,
    Hack,
    Fixme,
}

impl Kind {
    pub const ALL: [Self; 5] = [Self::Todo, Self::Fixme, Self::Hack, Self::Note, Self::Xxx];

    pub fn keyword(self) -> &'static str {
        match self {
            Self::Note => "NOTE",
            Self::Xxx => "XXX",
            Self::Todo => "TODO",
            Self::Hack => "HACK",
            Self::Fixme => "FIXME",
        }
    }

    pub fn style(self) -> DecorationStyle {
        match self {
            Self::Note => DecorationStyle::Success,
            Self::Xxx => DecorationStyle::Accent,
            Self::Todo => DecorationStyle::Info,
            Self::Hack => DecorationStyle::Warning,
            Self::Fixme => DecorationStyle::Error,
        }
    }

    pub fn sign(self) -> &'static str {
        match self {
            Self::Note => "N",
            Self::Xxx => "X",
            Self::Todo => "T",
            Self::Hack => "H",
            Self::Fixme => "!",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Marker {
    pub line: u32,
    pub start: u32,
    pub end: u32,
    pub kind: Kind,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Counts {
    pub todo: usize,
    pub fixme: usize,
    pub hack: usize,
    pub note: usize,
    pub xxx: usize,
}

impl Counts {
    pub fn from_markers(markers: &[Marker]) -> Self {
        let mut counts = Self::default();
        for marker in markers {
            counts.add(marker.kind);
        }
        counts
    }

    pub fn total(&self) -> usize {
        self.todo + self.fixme + self.hack + self.note + self.xxx
    }

    pub fn style(&self) -> DecorationStyle {
        if self.fixme > 0 {
            DecorationStyle::Error
        } else if self.hack > 0 {
            DecorationStyle::Warning
        } else if self.todo > 0 {
            DecorationStyle::Info
        } else if self.xxx > 0 {
            DecorationStyle::Accent
        } else if self.note > 0 {
            DecorationStyle::Success
        } else {
            DecorationStyle::Muted
        }
    }

    pub fn status_text(&self) -> String {
        format!("markers {}", self.total())
    }

    pub fn message(&self) -> String {
        if self.total() == 0 {
            return "no markers".to_string();
        }
        let details = self.details();
        format!("{} markers: {details}", self.total())
    }

    fn add(&mut self, kind: Kind) {
        match kind {
            Kind::Note => self.note += 1,
            Kind::Xxx => self.xxx += 1,
            Kind::Todo => self.todo += 1,
            Kind::Hack => self.hack += 1,
            Kind::Fixme => self.fixme += 1,
        }
    }

    fn details(&self) -> String {
        let entries = [
            ("TODO", self.todo),
            ("FIXME", self.fixme),
            ("HACK", self.hack),
            ("NOTE", self.note),
            ("XXX", self.xxx),
        ];
        entries
            .into_iter()
            .filter(|(_, count)| *count > 0)
            .map(|(label, count)| format!("{label} {count}"))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

pub fn scan(text: &str) -> Vec<Marker> {
    let mut markers = Vec::new();
    for (line_index, line) in text.split('\n').enumerate() {
        let Ok(line_number) = u32::try_from(line_index) else {
            break;
        };
        for kind in Kind::ALL {
            collect_line(line_number, line, kind, &mut markers);
        }
    }
    markers.sort_by_key(|marker| (marker.line, marker.start));
    markers
}

pub fn sign_lines(markers: &[Marker]) -> Vec<(u32, Kind)> {
    let mut lines = BTreeMap::new();
    for marker in markers {
        lines
            .entry(marker.line)
            .and_modify(|kind: &mut Kind| {
                if marker.kind > *kind {
                    *kind = marker.kind;
                }
            })
            .or_insert(marker.kind);
    }
    lines.into_iter().collect()
}

fn collect_line(line: u32, text: &str, kind: Kind, markers: &mut Vec<Marker>) {
    for (start, _) in text.match_indices(kind.keyword()) {
        let end = start + kind.keyword().len();
        if !has_word_boundaries(text, start, end) {
            continue;
        }
        let Ok(start) = u32::try_from(start) else {
            continue;
        };
        let Ok(end) = u32::try_from(end) else {
            continue;
        };
        markers.push(Marker {
            line,
            start,
            end,
            kind,
        });
    }
}

fn has_word_boundaries(text: &str, start: usize, end: usize) -> bool {
    let before = text[..start].chars().next_back();
    let after = text[end..].chars().next();
    !before.is_some_and(is_word_character) && !after.is_some_and(is_word_character)
}

fn is_word_character(character: char) -> bool {
    character.is_alphanumeric() || character == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_markers_with_utf8_byte_columns() {
        let markers = scan("é TODO and FIXME\nNOTE");
        assert_eq!(markers.len(), 3);
        assert_eq!(
            markers[0],
            Marker {
                line: 0,
                start: 3,
                end: 7,
                kind: Kind::Todo,
            }
        );
        assert_eq!(
            markers[1],
            Marker {
                line: 0,
                start: 12,
                end: 17,
                kind: Kind::Fixme,
            }
        );
        assert_eq!(
            markers[2],
            Marker {
                line: 1,
                start: 0,
                end: 4,
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
}
