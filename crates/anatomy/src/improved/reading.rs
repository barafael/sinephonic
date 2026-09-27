//! Chord structure, spelled symbols, Harte labels and root scoring.

use harmony::{PcSet, PitchClass, SpelledPc, Spelling};

use super::metrics;
use crate::degree::{Degree, Seventh, Third};
use crate::Reading;

/// The fifth as the symbol sees it.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Fifth {
    Perfect,
    /// ♭5 as a chord tone: diminished triads, and 7♭5 dominants.
    Flat,
    /// ♯5 as a chord tone: augmented triads and 7♯5.
    Sharp,
    None,
}

/// A root-relative pitch-class set, parsed into chord structure.
#[derive(Clone, PartialEq, Debug)]
pub struct Structure {
    pub rel: PcSet,
    pub third: Third,
    pub fifth: Fifth,
    pub seventh: Option<Seventh>,
    /// Both a major and a minor seventh.
    pub conflicting_seventh: bool,
    /// Natural extensions (9, 11, 13) and sixths (6, add9, add11).
    pub naturals: Vec<Degree>,
    /// ♭5 (non-diminished), ♯5, ♭9, ♯9, ♯11, ♭13, ♭6 and an added ♭7, in display order.
    pub alterations: Vec<Degree>,
    pub degrees: [Option<Degree>; 12],
}

impl Structure {
    pub fn new(rel: PcSet) -> Self {
        let h = |i: u8| rel.has(i);
        let p5 = h(7);
        let third = if h(4) {
            Third::Major
        } else if h(3) {
            Third::Minor
        } else if h(5) {
            Third::Sus4
        } else if h(2) {
            Third::Sus2
        } else {
            Third::None
        };
        let dim = third == Third::Minor && h(6) && !p5;
        let seventh = if h(11) {
            Some(Seventh::Major)
        } else if h(10) {
            Some(Seventh::Dominant)
        } else if dim && h(9) {
            Some(Seventh::Diminished)
        } else {
            None
        };
        let conflicting_seventh = h(10) && h(11);
        // A natural 9 or 13 on a seventh chord makes a lone tritone read as ♯11, not ♭5.
        let extended = seventh.is_some() && (h(2) && third != Third::Sus2 || h(9));
        let fifth = if p5 {
            Fifth::Perfect
        } else if dim {
            Fifth::Flat
        } else if third == Third::Major && h(8) {
            Fifth::Sharp
        } else if third == Third::Major && h(6) && !extended && seventh != Some(Seventh::Major) {
            // 7♭5 and (♭5); over a major seventh the tritone is the Lydian ♯11.
            Fifth::Flat
        } else {
            Fifth::None
        };

        let mut deg: [Option<Degree>; 12] = [None; 12];
        deg[0] = Some(Degree::Root);
        match third {
            Third::Major => deg[4] = Some(Degree::Third),
            Third::Minor => deg[3] = Some(Degree::FlatThird),
            Third::Sus4 => deg[5] = Some(Degree::Fourth),
            Third::Sus2 => deg[2] = Some(Degree::Second),
            Third::None => {}
        }
        match fifth {
            Fifth::Perfect => deg[7] = Some(Degree::Fifth),
            Fifth::Flat => deg[6] = Some(Degree::FlatFifth),
            Fifth::Sharp => deg[8] = Some(Degree::SharpFifth),
            Fifth::None => {}
        }
        match seventh {
            Some(Seventh::Major) => deg[11] = Some(Degree::Seventh),
            Some(Seventh::Dominant) => deg[10] = Some(Degree::FlatSeventh),
            Some(Seventh::Diminished) => deg[9] = Some(Degree::DoubleFlatSeventh),
            None => {}
        }
        let mut naturals = Vec::new();
        let mut alterations = Vec::new();
        // Chord-tone alterations first, so they sort before the tensions in the symbol.
        if fifth == Fifth::Flat && !dim {
            alterations.push(Degree::FlatFifth);
        }
        if fifth == Fifth::Sharp && seventh.is_some() {
            alterations.push(Degree::SharpFifth);
        }
        let mut put = |ic: usize, d: Degree, natural: bool| {
            if rel.has(ic as u8) && deg[ic].is_none() {
                deg[ic] = Some(d);
                if natural {
                    naturals.push(d);
                } else {
                    alterations.push(d);
                }
            }
        };
        put(1, Degree::FlatNine, false);
        put(2, Degree::Nine, true);
        put(3, Degree::SharpNine, false);
        put(5, Degree::Eleven, true);
        if p5 || seventh.is_some() {
            put(6, Degree::SharpEleven, false);
        } else {
            put(6, Degree::FlatFifth, false);
        }
        if seventh.is_some() {
            put(8, Degree::FlatThirteen, false);
            put(9, Degree::Thirteen, true);
        } else {
            put(8, Degree::FlatSix, false);
            put(9, Degree::Six, true);
        }
        put(10, Degree::FlatSeventh, false);
        Self {
            rel,
            third,
            fifth,
            seventh,
            conflicting_seventh,
            naturals,
            alterations,
            degrees: deg,
        }
    }

    pub fn diminished(&self) -> bool {
        self.third == Third::Minor && self.fifth == Fifth::Flat
    }

    pub fn augmented(&self) -> bool {
        self.third == Third::Major && self.fifth == Fifth::Sharp
    }

    fn has_natural(&self, d: Degree) -> bool {
        self.naturals.contains(&d)
    }

    /// Highest stacked natural extension (9, 11, 13) on a seventh chord, else 7.
    fn top_extension(&self) -> u8 {
        [
            (Degree::Thirteen, 13),
            (Degree::Eleven, 11),
            (Degree::Nine, 9),
        ]
        .into_iter()
        .find(|&(d, _)| self.has_natural(d))
        .map_or(7, |(_, n)| n)
    }

    /// A bare fifth: `{R, 5}`.
    pub fn is_power_chord(&self) -> bool {
        self.rel == PcSet::from_bits(0b000010000001)
    }

    /// Template fit: how well the notes fill a tertian chord from this root, with the root
    /// present or not. The base weights are the prototype's; the rest are in [`Weights`]:
    /// dominant skeletons (3 + ♭7) get a bonus and cheaper alterations, as in jazz practice;
    /// complete diminished and augmented triads count as full triads; added tones over a triad
    /// with no fifth are penalised (D F B is B°/D, not Dm6 without its fifth); both sevenths at
    /// once are penalised; a suspension over a perfect fifth is a proper sus chord.
    pub fn fit(&self, w: &Weights) -> f64 {
        let root = if self.rel.has(0) { 3.0 } else { -1.5 };
        let third = match self.third {
            Third::Major | Third::Minor => 2.0,
            Third::Sus4 | Third::Sus2 if self.fifth == Fifth::Perfect => -0.3,
            Third::Sus4 | Third::Sus2 => -0.5,
            Third::None => -2.5,
        };
        let fifth = match self.fifth {
            Fifth::Perfect => 1.0,
            _ if self.diminished() || self.augmented() => w.full_triad,
            Fifth::Flat => 0.1,
            _ => 0.0,
        };
        let seventh = if self.seventh.is_some() { 0.8 } else { 0.0 };
        let conflict = if self.conflicting_seventh { -1.5 } else { 0.0 };
        let dominant = self.third == Third::Major && self.seventh == Some(Seventh::Dominant);
        let alteration = if dominant {
            w.dominant_alteration
        } else if self.seventh.is_none() {
            w.triad_alteration
        } else {
            0.9
        };
        let added = !self.naturals.is_empty() || !self.alterations.is_empty();
        let incomplete = if self.fifth == Fifth::None && self.seventh.is_none() && added {
            -w.incomplete_added
        } else {
            0.0
        };
        root + third
            + fifth
            + seventh
            + conflict
            + incomplete
            + if dominant { w.dominant } else { 0.0 }
            - 0.4 * self.naturals.len() as f64
            - alteration * self.alterations.len() as f64
    }

    /// The display symbol after the root name, e.g. "m7♭5", "7♯5♯9", "maj9♯11", "6/9", "°7".
    pub fn quality(&self) -> String {
        if self.is_power_chord() {
            return "5".into();
        }
        if self.rel.len() == 1 {
            return String::new(); // octaves of one pitch class
        }
        // A bare fifth with only alterations added: C5(♯11), not C(♯11)(no3).
        if self.third == Third::None
            && self.fifth == Fifth::Perfect
            && self.seventh.is_none()
            && self.naturals.is_empty()
            && !self.alterations.is_empty()
        {
            let alts: Vec<&str> = self
                .alterations
                .iter()
                .map(|&d| alteration_label(d))
                .collect();
            return format!("5({})", alts.join(","));
        }
        let hi = self.top_extension();
        let mut s = String::new();
        let mut paren: Vec<String> = Vec::new();
        if self.diminished() {
            match self.seventh {
                Some(Seventh::Diminished) => {
                    s += "°7";
                    paren.extend(self.naturals.iter().map(|d| d.label().to_string()));
                }
                Some(Seventh::Dominant) => s += &format!("ø{hi}"),
                Some(Seventh::Major) => {
                    s += "°(maj7)";
                    paren.extend(self.naturals.iter().map(|d| d.label().to_string()));
                }
                None => {
                    s += "°";
                    paren.extend(self.naturals.iter().map(|d| format!("add{}", d.label())));
                }
            }
        } else {
            if self.third == Third::Minor {
                s += "m";
            }
            match self.seventh {
                Some(Seventh::Major) if self.third == Third::Minor => s += &format!("(maj{hi})"),
                Some(Seventh::Major) => s += &format!("maj{hi}"),
                Some(_) => s += &hi.to_string(),
                None => {
                    if self.augmented() {
                        s += "+";
                    }
                    let six = self.has_natural(Degree::Six);
                    let nine = self.has_natural(Degree::Nine);
                    s += match (six, nine) {
                        (true, true) => "6/9",
                        (true, false) => "6",
                        (false, true) if self.third == Third::Sus4 => "",
                        (false, true) => "add9",
                        (false, false) => "",
                    };
                    if self.has_natural(Degree::Eleven) {
                        s += "add11";
                    }
                }
            }
        }
        match self.third {
            Third::Sus4 => {
                s += "sus4";
                if self.seventh.is_none()
                    && self.has_natural(Degree::Nine)
                    && !self.has_natural(Degree::Six)
                {
                    s += "add9";
                }
            }
            Third::Sus2 => s += "sus2",
            _ => {}
        }
        let alts: Vec<&str> = self
            .alterations
            .iter()
            .map(|&d| alteration_label(d))
            .collect();
        if self.seventh.is_some() && !self.diminished() {
            s += &alts.concat();
        } else {
            paren.extend(alts.iter().map(|a| a.to_string()));
        }
        if !paren.is_empty() {
            s += &format!("({})", paren.join(","));
        }
        if self.third == Third::None {
            s += "(no3)";
        }
        s
    }

    /// Harte et al. (2005) shorthand and degree list, without root and bass.
    pub fn harte(&self) -> String {
        const SHORTHANDS: [(&str, &[&str]); 17] = [
            ("maj", &["1", "3", "5"]),
            ("min", &["1", "b3", "5"]),
            ("dim", &["1", "b3", "b5"]),
            ("aug", &["1", "3", "#5"]),
            ("maj7", &["1", "3", "5", "7"]),
            ("min7", &["1", "b3", "5", "b7"]),
            ("7", &["1", "3", "5", "b7"]),
            ("dim7", &["1", "b3", "b5", "bb7"]),
            ("hdim7", &["1", "b3", "b5", "b7"]),
            ("minmaj7", &["1", "b3", "5", "7"]),
            ("maj6", &["1", "3", "5", "6"]),
            ("min6", &["1", "b3", "5", "6"]),
            ("9", &["1", "3", "5", "b7", "9"]),
            ("maj9", &["1", "3", "5", "7", "9"]),
            ("min9", &["1", "b3", "5", "b7", "9"]),
            ("sus4", &["1", "4", "5"]),
            ("sus2", &["1", "2", "5"]),
        ];
        let have: Vec<String> = (0..12u8)
            .filter(|&ic| self.rel.has(ic))
            .filter_map(|ic| self.degrees[ic as usize].map(|d| d.harte()))
            .collect();
        // The largest shorthand whose tones are all present, allowing the root and fifth to be
        // omitted (written *1, *5).
        let best = SHORTHANDS
            .iter()
            .filter(|(_, tones)| {
                tones
                    .iter()
                    .all(|t| have.iter().any(|h| h == t) || *t == "1" || *t == "5")
            })
            .max_by_key(|(_, tones)| {
                let present = tones
                    .iter()
                    .filter(|t| have.iter().any(|h| h == *t))
                    .count() as i32;
                let omitted = tones.len() as i32 - present;
                2 * present - 3 * omitted + tones.len() as i32
            });
        let (name, tones): (&str, &[&str]) = best.map_or(("", &[]), |&(n, t)| (n, t));
        let mut extra: Vec<String> = Vec::new();
        for t in tones {
            if !have.iter().any(|h| h == t) {
                extra.push(format!("*{t}"));
            }
        }
        for h in &have {
            if !tones.contains(&h.as_str()) && !(name.is_empty() && h == "1") {
                extra.push(h.clone());
            }
        }
        if name.is_empty() {
            if !self.rel.has(0) {
                extra.insert(0, "*1".into());
            }
            return format!("({})", extra.join(","));
        }
        if extra.is_empty() {
            name.into()
        } else {
            format!("{name}({})", extra.join(","))
        }
    }
}

fn alteration_label(d: Degree) -> &'static str {
    match d {
        Degree::FlatSeventh => "add♭7",
        d => d.label(),
    }
}

fn accidental_cost(s: SpelledPc) -> i32 {
    match s.accidentals().abs() {
        0 => 0,
        1 => 1,
        2 => 4,
        n => 4 * n,
    }
}

/// Spells the root so its chord tones need the fewest accidentals: A♭ for A♭–C–E♭, but
/// G♯ for G♯–B–D♯. White keys are always natural. Ties go to the preferred spelling.
pub fn spell_root(root: PitchClass, st: &Structure, pref: Spelling) -> SpelledPc {
    let candidates: Vec<SpelledPc> = if root.is_black_key() {
        SpelledPc::spellings(root, 1).collect()
    } else {
        SpelledPc::spellings(root, 0).collect()
    };
    let cost = |r: SpelledPc| -> i32 {
        st.degrees
            .iter()
            .flatten()
            .map(|d| accidental_cost(r + d.interval()))
            .sum()
    };
    let flats_first = pref == Spelling::Flats;
    *candidates
        .iter()
        .min_by_key(|&&c| {
            (
                cost(c),
                if (c.accidentals() < 0) == flats_first {
                    0
                } else {
                    1
                },
            )
        })
        .expect("every pitch class has a spelling")
}

/// The bass of a slash chord keeps its chord-tone spelling (D7/F♯, A♭maj7/C) unless that
/// would be B♯, E♯, C♭, F♭ or a double accidental; then it takes the plain name (F♯7/C, not
/// F♯7/B♯).
fn slash_name(s: SpelledPc, pref: Spelling) -> String {
    let odd = s.accidentals() != 0 && !s.pc().is_black_key();
    if odd || s.accidentals().abs() > 1 {
        pref.pc_name(s.pc()).to_string()
    } else {
        s.name(true)
    }
}

/// Weights for combining the evidence for a root. Fitted against `tests/corpus.txt` with
/// `cargo run --release -p anatomy --example tune`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weights {
    /// Root in the bass.
    pub bass: f64,
    /// Times the normalised Parncutt root support (0..1).
    pub parncutt: f64,
    /// Root of the voicing's best interval (Hindemith).
    pub hindemith: f64,
    /// Major 3rd + minor 7th present.
    pub dominant: f64,
    /// Cost per alteration on a dominant (0.9 elsewhere).
    pub dominant_alteration: f64,
    /// Added tones on a triad with neither fifth nor seventh.
    pub incomplete_added: f64,
    /// Credit for the ♭5/♯5 of complete diminished/augmented triads (1.0 for a perfect 5th).
    pub full_triad: f64,
    /// Cost per alteration added to a chord without a seventh, such as the ♭6 of m(♭6).
    pub triad_alteration: f64,
    /// Cost of a hybrid slash chord (F/G): a bass that is only an extension of the chord.
    pub foreign_bass: f64,
    /// Softmax inverse temperature for the probabilities.
    pub temperature: f64,
}

pub const WEIGHTS: Weights = Weights {
    bass: 3.0,
    parncutt: 2.0,
    hindemith: 0.5,
    dominant: 1.0,
    dominant_alteration: 0.3,
    incomplete_added: 2.5,
    full_triad: 1.0,
    triad_alteration: 1.5,
    foreign_bass: 0.3,
    temperature: 0.8,
};

/// One reading. `bass_alone` says the bass pitch class sounds only in the bass, which allows
/// a hybrid slash chord: the upper notes form a chord and the bass is an extension of it
/// (F/G rather than Fadd9/G). The reading takes whichever interpretation fits better.
#[allow(clippy::too_many_arguments)]
pub fn reading(
    root: PitchClass,
    pcs: PcSet,
    bass: PitchClass,
    bass_alone: bool,
    support: f64,
    hindemith: Option<PitchClass>,
    pref: Spelling,
    w: &Weights,
) -> Reading {
    let rel = pcs.relative_to(root);
    let bass_ic = root.up_to(bass) as usize;
    let full = Structure::new(rel);
    let mut st = full.clone();
    let mut fit = full.fit(w);
    let mut hybrid = false;
    if bass != root && bass_alone {
        let bass_degree = full.degrees[bass_ic].expect("every present tone has a degree");
        if full.naturals.contains(&bass_degree) || full.alterations.contains(&bass_degree) {
            let upper = Structure::new(PcSet::from_bits(rel.bits() & !(1 << bass_ic)));
            let upper_fit = upper.fit(w) - w.foreign_bass;
            if upper_fit > fit {
                st = upper;
                st.degrees[bass_ic] = Some(bass_degree);
                fit = upper_fit;
                hybrid = true;
            }
        }
    }
    let r = spell_root(root, &st, pref);
    let tone_names: [Option<String>; 12] =
        std::array::from_fn(|ic| st.degrees[ic].map(|d| (r + d.interval()).name(true)));
    let root_name = r.name(true);
    let mut symbol = format!("{root_name}{}", st.quality());
    let mut harte = format!("{}:{}", r.name(false), st.harte());
    if bass != root {
        symbol += "/";
        symbol += &slash_name(
            r + st.degrees[bass_ic].expect("bass has a degree").interval(),
            pref,
        );
        harte += "/";
        harte += &st.degrees[bass_ic].map_or("?".into(), |d| d.harte());
    }
    let _ = hybrid;
    let score = fit
        + if root == bass { w.bass } else { 0.0 }
        + w.parncutt * support
        + if hindemith == Some(root) {
            w.hindemith
        } else {
            0.0
        };
    Reading {
        root,
        root_name,
        symbol,
        harte,
        score,
        prob: 0.0,
        root_present: rel.has(0),
        third: st.third,
        seventh: st.seventh,
        perfect_fifth: st.fifth == Fifth::Perfect,
        diminished: st.diminished(),
        augmented: st.augmented(),
        alterations: st.alterations.len(),
        rel,
        degrees: st.degrees,
        tone_names,
    }
}

/// All 12 readings of a voicing, best first. `notes` sorted ascending.
pub fn readings(notes: &[u8], pref: Spelling, w: &Weights) -> Vec<Reading> {
    let pcs = PcSet::from_midi(notes);
    let bass = PitchClass::of_midi(notes[0]);
    let support = metrics::parncutt_support(pcs);
    let max = support.iter().copied().fold(0.0, f64::max);
    let hind = metrics::hindemith_root(notes);
    let bass_alone = notes[1..].iter().all(|&m| m % 12 != notes[0] % 12);
    let mut all: Vec<Reading> = (0..12)
        .map(|r| {
            let s = if max > 0.0 { support[r] / max } else { 0.0 };
            reading(
                PitchClass::new(r as i32),
                pcs,
                bass,
                bass_alone,
                s,
                hind,
                pref,
                w,
            )
        })
        .collect();
    crate::reference::sort_and_normalize(&mut all, w.temperature);
    all
}

#[cfg(test)]
mod tests {
    use super::*;

    fn st(ics: &[u8]) -> Structure {
        Structure::new(ics.iter().map(|&i| PitchClass::from(i)).collect())
    }

    fn q(ics: &[u8]) -> String {
        st(ics).quality()
    }

    #[test]
    fn qualities() {
        let cases: &[(&[u8], &str)] = &[
            (&[0, 4, 7], ""),
            (&[0, 3, 7], "m"),
            (&[0, 3, 6], "°"),
            (&[0, 4, 8], "+"),
            (&[0, 4, 7, 11], "maj7"),
            (&[0, 4, 7, 10], "7"),
            (&[0, 3, 7, 10], "m7"),
            (&[0, 3, 6, 10], "ø7"),
            (&[0, 3, 6, 9], "°7"),
            (&[0, 3, 7, 11], "m(maj7)"),
            (&[0, 4, 7, 9], "6"),
            (&[0, 3, 7, 9], "m6"),
            (&[0, 4, 7, 9, 2], "6/9"),
            (&[0, 4, 7, 2], "add9"),
            (&[0, 5, 7], "sus4"),
            (&[0, 2, 7], "sus2"),
            (&[0, 5, 7, 10], "7sus4"),
            (&[0, 5, 7, 10, 2], "9sus4"),
            (&[0, 5, 7, 2], "sus4add9"),
            (&[0, 4, 7, 10, 2], "9"),
            (&[0, 4, 10, 2, 9], "13"),
            (&[0, 3, 7, 10, 2, 5], "m11"),
            (&[0, 4, 7, 11, 2, 6], "maj9♯11"),
            (&[0, 4, 7, 10, 3], "7♯9"),
            (&[0, 4, 8, 10, 3], "7♯5♯9"),
            (&[0, 4, 7, 10, 1], "7♭9"),
            (&[0, 4, 6, 10], "7♭5"),
            (&[0, 4, 10, 6, 2], "9♯11"),
            (&[0, 4, 8, 11], "maj7♯5"),
            (&[0, 4, 6, 11], "maj7♯11"),
            (&[0, 3, 7, 8], "m(♭6)"),
            (&[0, 7], "5"),
            (&[0, 6, 7], "5(♯11)"),
            (&[0, 1, 7], "5(♭9)"),
            (&[0, 7, 10], "7(no3)"),
            (&[0, 3, 6, 10, 2], "ø9"),
            (&[0, 3, 6, 9, 2], "°7(9)"),
            (&[0, 4, 7, 10, 11], "maj7add♭7"),
        ];
        for (ics, want) in cases {
            assert_eq!(&q(ics), want, "{ics:?}");
        }
    }

    #[test]
    fn prototype_known_bad_symbols_are_fixed() {
        // Prototype: A°(maj7)♭7/C for C G A♭ E♭ read from A; and E♭6sus4(♭9)/C.
        let pcs = PcSet::from_midi(&[48, 55, 56, 63]);
        let names: Vec<String> = (0..12)
            .map(|r| {
                reading(
                    PitchClass::new(r),
                    pcs,
                    PitchClass::new(0),
                    true,
                    0.0,
                    None,
                    Spelling::Flats,
                    &WEIGHTS,
                )
                .symbol
            })
            .collect();
        assert!(!names.iter().any(|n| n.contains("°(maj7)♭7")), "{names:?}");
        assert!(names.contains(&"A♭maj7/C".to_string()), "{names:?}");
        assert!(names.contains(&"Cm(♭6)".to_string()), "{names:?}");
    }

    #[test]
    fn spelling() {
        let name = |pcs: &[u8], root: i32| {
            let set: PcSet = pcs.iter().map(|&p| PitchClass::from(p)).collect();
            reading(
                PitchClass::new(root),
                set,
                PitchClass::new(root),
                true,
                1.0,
                None,
                Spelling::Flats,
                &WEIGHTS,
            )
        };
        let dim7 = name(&[0, 3, 6, 9], 0);
        assert_eq!(dim7.symbol, "C°7");
        let tones: Vec<&str> = [0, 3, 6, 9]
            .iter()
            .map(|&p| dim7.tone_name(PitchClass::new(p)))
            .collect();
        assert_eq!(tones, ["C", "E♭", "G♭", "B𝄫"]);
        assert_eq!(name(&[8, 11, 3], 8).symbol, "G♯m"); // G♯ B D♯, not A♭ C♭ E♭
        assert_eq!(name(&[8, 0, 3], 8).symbol, "A♭");
        assert_eq!(name(&[1, 5, 8], 1).symbol, "D♭"); // D♭ F A♭ beats C♯ E♯ G♯
        assert_eq!(name(&[6, 10, 1], 6).symbol, "G♭"); // tie → preferred flats
        let sharp = reading(
            PitchClass::new(6),
            [6u8, 10, 1].iter().map(|&p| PitchClass::from(p)).collect(),
            PitchClass::new(6),
            true,
            1.0,
            None,
            Spelling::Sharps,
            &WEIGHTS,
        );
        assert_eq!(sharp.symbol, "F♯");
        let s9 = name(&[0, 4, 7, 10, 3], 0);
        assert_eq!(s9.tone_name(PitchClass::new(3)), "D♯");
        // Slash bass spelled as a chord tone: D7/F♯.
        let set: PcSet = [2u8, 6, 9, 0]
            .iter()
            .map(|&p| PitchClass::from(p))
            .collect();
        let d7 = reading(
            PitchClass::new(2),
            set,
            PitchClass::new(6),
            true,
            1.0,
            None,
            Spelling::Flats,
            &WEIGHTS,
        );
        assert_eq!(d7.symbol, "D7/F♯");
        assert_eq!(d7.harte, "D:7/3");
    }

    #[test]
    fn harte_labels() {
        let h = |ics: &[u8]| st(ics).harte();
        assert_eq!(h(&[0, 4, 7]), "maj");
        assert_eq!(h(&[0, 3, 7, 8]), "min(b6)");
        assert_eq!(h(&[0, 4, 7, 11]), "maj7");
        assert_eq!(h(&[0, 4, 11]), "maj7(*5)");
        assert_eq!(h(&[0, 3, 6, 9]), "dim7");
        assert_eq!(h(&[0, 3, 6, 10]), "hdim7");
        assert_eq!(h(&[0, 4, 7, 10, 3]), "7(#9)");
        assert_eq!(h(&[0, 4, 7, 10, 2, 9]), "9(13)");
        assert_eq!(h(&[1, 4, 8, 9]), "aug(*1,b9,6)");
    }
}
