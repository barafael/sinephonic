//! A plain-text corpus of voicings with expected readings, for regression tests and for
//! judging the model by ear and eye.
//!
//! One voicing per line:
//!
//! ```text
//! # comment or section heading
//! E3 G3 C4        = C/E
//! Eb3 G3 Bb3 C4   = E♭6 ~ Cm7/E♭ ! disputed: both are standard
//! C3 E3 G3        = C   @ fused
//! ```
//!
//! * `= symbol` is the expected best reading.
//! * `~ a, b` lists other acceptable best readings.
//! * `@ tag tag` lists tags that must appear.
//! * `! note` marks the entry as disputed: reported, never failed.

use crate::{Analysis, Options};

#[derive(Clone, PartialEq, Debug)]
pub struct Entry {
    pub line: usize,
    pub section: String,
    pub text: String,
    pub notes: Vec<u8>,
    pub expected: String,
    pub alternates: Vec<String>,
    pub tags: Vec<String>,
    pub disputed: Option<String>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Verdict {
    /// Best reading is the expected one.
    Match,
    /// Best reading is one of the alternates.
    Alternate,
    /// Best reading is something else; holds the rank of the expected reading, if found.
    Miss { expected_rank: Option<usize> },
}

#[derive(Clone, PartialEq, Debug)]
pub struct Outcome {
    pub verdict: Verdict,
    pub best: String,
    pub missing_tags: Vec<String>,
}

impl Outcome {
    pub fn passed(&self) -> bool {
        !matches!(self.verdict, Verdict::Miss { .. }) && self.missing_tags.is_empty()
    }
}

pub fn parse(text: &str) -> Result<Vec<Entry>, String> {
    let mut out = Vec::new();
    let mut section = String::new();
    for (k, raw) in text.lines().enumerate() {
        let line = k + 1;
        let t = raw.trim();
        if t.is_empty() {
            continue;
        }
        if let Some(h) = t.strip_prefix('#') {
            section = h.trim().to_string();
            continue;
        }
        let (body, disputed) = match t.split_once('!') {
            Some((b, d)) => (b, Some(d.trim().to_string())),
            None => (t, None),
        };
        let (body, tags) = match body.split_once('@') {
            Some((b, tg)) => (b, tg.split_whitespace().map(String::from).collect()),
            None => (body, Vec::new()),
        };
        let (body, alternates) = match body.split_once('~') {
            Some((b, a)) => (
                b,
                a.split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect(),
            ),
            None => (body, Vec::new()),
        };
        let (notes_text, expected) = body
            .split_once('=')
            .ok_or_else(|| format!("line {line}: missing `= symbol`"))?;
        let notes = harmony::midi::parse_notes(notes_text);
        if notes.len() < 2 {
            return Err(format!(
                "line {line}: fewer than two notes in {notes_text:?}"
            ));
        }
        out.push(Entry {
            line,
            section: section.clone(),
            text: notes_text.trim().to_string(),
            notes,
            expected: expected.trim().to_string(),
            alternates,
            tags,
            disputed,
        });
    }
    Ok(out)
}

pub fn judge(entry: &Entry, a: &Analysis) -> Outcome {
    let best = a.best().symbol.clone();
    let verdict = if best == entry.expected {
        Verdict::Match
    } else if entry.alternates.contains(&best) {
        Verdict::Alternate
    } else {
        Verdict::Miss {
            expected_rank: a.readings.iter().position(|r| r.symbol == entry.expected),
        }
    };
    let missing_tags = entry
        .tags
        .iter()
        .filter(|t| !a.tags.iter().any(|x| x.word == t.as_str()))
        .cloned()
        .collect();
    Outcome {
        verdict,
        best,
        missing_tags,
    }
}

pub fn run(entries: &[Entry], opts: &Options) -> Vec<(Entry, Outcome, Analysis)> {
    entries
        .iter()
        .map(|e| {
            let a = crate::analyze(&e.notes, opts).expect("two or more notes");
            let o = judge(e, &a);
            (e.clone(), o, a)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_fields() {
        let e = parse("# Sevenths\nEb3 G3 Bb3 C4 = E♭6 ~ Cm7/E♭, X ! both standard\nC3 E3 G3 = C @ fused bright\n").unwrap();
        assert_eq!(e.len(), 2);
        assert_eq!(e[0].section, "Sevenths");
        assert_eq!(e[0].notes, [51, 55, 58, 60]);
        assert_eq!(e[0].expected, "E♭6");
        assert_eq!(e[0].alternates, ["Cm7/E♭", "X"]);
        assert_eq!(e[0].disputed.as_deref(), Some("both standard"));
        assert_eq!(e[1].tags, ["fused", "bright"]);
        assert!(parse("C3 E3").is_err());
    }
}
