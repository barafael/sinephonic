//! MIDI note numbers, frequencies, note names and the note-text parser.

use crate::pc::PitchClass;
use crate::spelled::SpelledPc;

/// How to name black keys when no chord context says otherwise.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Spelling {
    #[default]
    Flats,
    Sharps,
}

const FLATS: [&str; 12] = [
    "C", "D♭", "D", "E♭", "E", "F", "G♭", "G", "A♭", "A", "B♭", "B",
];
const SHARPS: [&str; 12] = [
    "C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B",
];

impl Spelling {
    pub fn pc_name(self, pc: PitchClass) -> &'static str {
        match self {
            Self::Flats => FLATS[pc.value() as usize],
            Self::Sharps => SHARPS[pc.value() as usize],
        }
    }

    pub fn spell(self, pc: PitchClass) -> SpelledPc {
        SpelledPc::parse(self.pc_name(pc)).expect("table names parse")
    }
}

/// A MIDI note number, 60 = C4.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug)]
pub struct Midi(pub u8);

impl Midi {
    pub const fn pc(self) -> PitchClass {
        PitchClass::of_midi(self.0)
    }

    /// Scientific-pitch octave: MIDI 60 is in octave 4.
    pub const fn octave(self) -> i32 {
        self.0 as i32 / 12 - 1
    }

    /// Twelve-tone equal-tempered frequency.
    pub fn freq(self, a4: f64) -> f64 {
        a4 * 2f64.powf((self.0 as f64 - 69.0) / 12.0)
    }

    /// e.g. "E♭4" with the default spelling for black keys.
    pub fn name(self, spelling: Spelling) -> String {
        format!("{}{}", spelling.pc_name(self.pc()), self.octave())
    }

    /// e.g. "Eb4": the form the text field shows.
    pub fn ascii_name(self, spelling: Spelling) -> String {
        self.name(spelling).replace('♭', "b").replace('♯', "#")
    }

    /// With an explicit spelling. The octave follows the letter, so B♯3 is MIDI 60.
    pub fn spelled_name(self, s: SpelledPc) -> String {
        debug_assert_eq!(s.pc(), self.pc());
        format!("{}{}", s.name(true), s.octave_for_midi(self.0))
    }
}

/// Sorts, removes duplicates and drops anything outside 0..=127.
pub fn normalize(notes: impl IntoIterator<Item = i32>) -> Vec<u8> {
    let mut v: Vec<u8> = notes
        .into_iter()
        .filter(|m| (0..128).contains(m))
        .map(|m| m as u8)
        .collect();
    v.sort_unstable();
    v.dedup();
    v
}

/// A note token as the user wrote it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ParsedNote {
    pub midi: i32,
    pub spelled: SpelledPc,
}

/// Parses note text such as `"C3 G3 Ab3 Eb4"` or `"C G Ab Eb"`.
///
/// * Tokens are separated by whitespace, `,`, `–` or `—`, and must look like
///   `[A-Ga-g][#b♯♭]*(-?digit)?`. Other tokens are skipped.
/// * With an octave, `midi = (octave + 1)·12 + pc`, so `Cb4` is 59.
/// * Without one, the first note goes in octave 3 and each later note takes the lowest pitch
///   above the previous note.
///
/// Returned in input order and not normalised (see [`parse_notes`]).
pub fn parse_tokens(text: &str) -> Vec<ParsedNote> {
    let mut out: Vec<ParsedNote> = Vec::new();
    let mut prev: Option<i32> = None;
    for tok in text
        .split(|c: char| c.is_whitespace() || matches!(c, ',' | '–' | '—'))
        .filter(|t| !t.is_empty())
    {
        let Some((spelled, octave)) = parse_token(tok) else {
            continue;
        };
        let pc = spelled.natural_pc().value() as i32 + spelled.accidentals();
        let m = match (octave, prev) {
            (Some(o), _) => (o + 1) * 12 + pc,
            (None, None) => 48 + pc.rem_euclid(12),
            (None, Some(p)) => p + 1 + (pc - (p + 1)).rem_euclid(12),
        };
        prev = Some(m);
        out.push(ParsedNote { midi: m, spelled });
    }
    out
}

fn parse_token(tok: &str) -> Option<(SpelledPc, Option<i32>)> {
    let mut chars = tok.chars().peekable();
    let letter = chars
        .next()
        .filter(|c| matches!(c.to_ascii_uppercase(), 'A'..='G'))?;
    let mut acc = 0;
    while let Some(&c) = chars.peek() {
        match c {
            '#' | '♯' => acc += 1,
            'b' | '♭' => acc -= 1,
            _ => break,
        }
        chars.next();
    }
    let rest: String = chars.collect();
    let octave = match rest.as_str() {
        "" => None,
        r => {
            let digits = r.strip_prefix('-').unwrap_or(r);
            if digits.len() != 1 || !digits.chars().all(|c| c.is_ascii_digit()) {
                return None;
            }
            Some(r.parse::<i32>().ok()?)
        }
    };
    Some((SpelledPc::new(letter, acc)?, octave))
}

/// [`parse_tokens`], then sorted, deduplicated and limited to MIDI 0..=127.
pub fn parse_notes(text: &str) -> Vec<u8> {
    normalize(parse_tokens(text).into_iter().map(|n| n.midi))
}

/// The text-field form of a voicing: `"C3 G3 Ab3 Eb4"`.
pub fn format_notes(notes: &[u8], spelling: Spelling) -> String {
    notes
        .iter()
        .map(|&m| Midi(m).ascii_name(spelling))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frequencies_and_names() {
        assert_eq!(Midi(69).freq(440.0), 440.0);
        assert!((Midi(60).freq(440.0) - 261.6256).abs() < 1e-3);
        assert_eq!(Midi(63).name(Spelling::Flats), "E♭4");
        assert_eq!(Midi(63).name(Spelling::Sharps), "D♯4");
        assert_eq!(Midi(56).ascii_name(Spelling::Flats), "Ab3");
        assert_eq!(Midi(36).name(Spelling::Flats), "C2");
        assert_eq!(Midi(0).octave(), -1);
        assert_eq!(
            Midi(60).spelled_name(SpelledPc::parse("B#").unwrap()),
            "B♯3"
        );
    }

    #[test]
    fn parser_matches_the_handoff() {
        assert_eq!(parse_notes("C G Ab Eb"), [48, 55, 56, 63]);
        assert_eq!(parse_notes("C3 G3 Ab3 Eb4"), [48, 55, 56, 63]);
        assert_eq!(parse_notes("c3, g3 – a♭3 — e♭4"), [48, 55, 56, 63]);
        assert_eq!(parse_notes("C E G Bb D#"), [48, 52, 55, 58, 63]);
        // Octave-less notes climb from the previous note, even across octaves.
        assert_eq!(parse_notes("E C"), [52, 60]);
        assert_eq!(parse_notes("C C"), [48, 60]);
        // Unicode and repeated accidentals.
        assert_eq!(parse_notes("B♭2 F##3"), [46, 55]);
        // Wraparound spellings.
        assert_eq!(parse_notes("Cb4"), [59]);
        assert_eq!(parse_notes("B#3"), [60]);
        assert_eq!(parse_notes("Cb"), [59]);
        // Negative octave, junk tokens, out of range.
        assert_eq!(parse_notes("C-1 H3 C44 xyz G"), [0, 7]);
        assert_eq!(parse_notes("G9 G#9"), [127]);
        assert_eq!(parse_notes(""), Vec::<u8>::new());
        // Duplicates collapse, order doesn't matter.
        assert_eq!(parse_notes("G3 C3 G3"), [48, 55]);
    }

    #[test]
    fn parser_keeps_spelling() {
        let t = parse_tokens("C E♭ G♭ Bbb");
        let names: Vec<String> = t.iter().map(|n| n.spelled.name(true)).collect();
        assert_eq!(names, ["C", "E♭", "G♭", "B𝄫"]);
        assert_eq!(
            t.iter().map(|n| n.midi).collect::<Vec<_>>(),
            [48, 51, 54, 57]
        );
    }

    #[test]
    fn format_roundtrip() {
        let v = [36u8, 48, 55, 56, 63, 84];
        assert_eq!(parse_notes(&format_notes(&v, Spelling::Flats)), v);
        assert_eq!(parse_notes(&format_notes(&v, Spelling::Sharps)), v);
    }
}
