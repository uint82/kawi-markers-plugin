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

    pub fn group(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Xxx => "xxx",
            Self::Todo => "todo",
            Self::Hack => "hack",
            Self::Fixme => "fixme",
        }
    }

    pub fn colon_group(self) -> &'static str {
        match self {
            Self::Note => "note_colon",
            Self::Xxx => "xxx_colon",
            Self::Todo => "todo_colon",
            Self::Hack => "hack_colon",
            Self::Fixme => "fixme_colon",
        }
    }

    pub fn text_group(self) -> &'static str {
        match self {
            Self::Note => "note_text",
            Self::Xxx => "xxx_text",
            Self::Todo => "todo_text",
            Self::Hack => "hack_text",
            Self::Fixme => "fixme_text",
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
    pub badge_start: u32,
    pub badge_end: u32,
    pub colon_start: u32,
    pub colon_end: u32,
    pub text_start: u32,
    pub text_end: u32,
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
    markers.sort_by_key(|marker| (marker.line, marker.badge_start));
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
        let Some(marker) = marker(line, text, kind, start, end) else {
            continue;
        };
        markers.push(marker);
    }
}

fn marker(line: u32, text: &str, kind: Kind, start: usize, end: usize) -> Option<Marker> {
    let colon_end = colon_end(text, end);
    let (badge_start, badge_end, colon_start, colon_end) = match colon_end {
        Some(colon_end) => (
            badge_start(text, start),
            colon_end,
            colon_end - 1,
            colon_end,
        ),
        None => (start, end, end, end),
    };
    let text_start = skip_whitespace(text, badge_end);
    let text_end = text.trim_end().len();
    Some(Marker {
        line,
        badge_start: u32::try_from(badge_start).ok()?,
        badge_end: u32::try_from(badge_end).ok()?,
        colon_start: u32::try_from(colon_start).ok()?,
        colon_end: u32::try_from(colon_end).ok()?,
        text_start: u32::try_from(text_start).ok()?,
        text_end: u32::try_from(text_end).ok()?,
        kind,
    })
}

fn colon_end(text: &str, end: usize) -> Option<usize> {
    let after = &text[end..];
    let colon = after.find(':')?;
    after[..colon]
        .chars()
        .all(char::is_whitespace)
        .then_some(end + colon + 1)
}

fn badge_start(text: &str, start: usize) -> usize {
    let leading = preceding_padding(text, start);
    start - leading
}

fn preceding_padding(text: &str, start: usize) -> usize {
    text[..start]
        .chars()
        .next_back()
        .filter(|character| character.is_whitespace())
        .map(char::len_utf8)
        .unwrap_or(0)
}

fn skip_whitespace(text: &str, start: usize) -> usize {
    text[start..]
        .char_indices()
        .find(|(_, character)| !character.is_whitespace())
        .map(|(offset, _)| start + offset)
        .unwrap_or(text.len())
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
mod tests;
