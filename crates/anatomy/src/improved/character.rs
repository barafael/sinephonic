//! Character tags whose explanations are checked against the voicing.
//!
//! The prototype's texts describe pitch-class relations as if the chord were always in close
//! root position ("Minor 3rd over a pure 5th: stable" for C–F–A♭, a six-four). Here every claim
//! about order, register, interval or tuning is derived from the actual notes:
//!
//! * inversions are named, and a six-four or a 7th in the bass is flagged as unsettled;
//! * a tritone's resolution direction follows its voicing (d5 closes inward, A4 opens outward);
//! * a major 7th voiced under its root is a clash, not "lush";
//! * "pure" is only said of just intervals, and periodicity texts qualify 12-TET;
//! * semitone roughness quotes the measured value.
//!
//! `tests/claims.rs` checks these statements against the notes for random voicings.

use harmony::{PcSet, SpelledPc};

use crate::degree::{Degree, Seventh, Third};
use crate::periodicity::Periodicity;
use crate::{interval_name, Options, Pair, Reading, Tag, Tuning};

/// Where the chord's notes actually are.
pub(crate) struct Voicing<'a> {
    pub notes: &'a [u8],
    pub best: &'a Reading,
    pub names: &'a [String],
}

impl Voicing<'_> {
    /// Indices of the notes that are `ic` semitones above the root (mod 12), lowest first.
    fn with(&self, ic: u8) -> Vec<usize> {
        (0..self.notes.len())
            .filter(|&i| {
                self.best
                    .root
                    .up_to(harmony::PitchClass::of_midi(self.notes[i]))
                    == ic
            })
            .collect()
    }

    fn lowest(&self, ic: u8) -> Option<usize> {
        self.with(ic).first().copied()
    }

    /// Note name without octave.
    fn name(&self, i: usize) -> &str {
        self.names[i].trim_end_matches(|c: char| c.is_ascii_digit() || c == '-')
    }

    /// The function of the bass note in the best reading.
    pub fn bass_degree(&self) -> Option<Degree> {
        self.best
            .degree_of(harmony::PitchClass::of_midi(self.notes[0]))
    }
}

/// The inversion of the best reading, from the bass note's function.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Position {
    Root,
    /// 3rd in the bass (6/3, 6/5).
    First,
    /// 5th in the bass (6/4, 4/3).
    Second,
    /// 7th in the bass (4/2).
    Third,
    /// The bass is an extension, an alteration or the root is absent.
    Other,
}

pub(crate) fn position(v: &Voicing) -> Position {
    if !v.best.root_present {
        return Position::Other;
    }
    match v.bass_degree() {
        Some(Degree::Root) => Position::Root,
        Some(Degree::Third | Degree::FlatThird) => Position::First,
        Some(Degree::Fifth) => Position::Second,
        Some(Degree::Seventh | Degree::FlatSeventh | Degree::DoubleFlatSeventh) => Position::Third,
        _ => Position::Other,
    }
}

pub(crate) fn tags(
    v: &Voicing,
    all: &[Reading],
    pairs: &[Pair],
    p: &Periodicity,
    ambiguity: f64,
    set: PcSet,
    opts: &Options,
) -> Vec<Tag> {
    let best = v.best;
    let rel = best.rel;
    let root = SpelledPc::parse(&best.root_name).expect("root name parses");
    let nm = |ic: u8| -> String {
        best.tone_names[ic as usize]
            .clone()
            .unwrap_or_else(|| (root + Degree::Root.interval()).name(true))
    };
    let pos = position(v);
    let cross = rel.has(3) && rel.has(4);
    let b9 = rel.has(1) && rel.has(4);
    let dominant = best.third == Third::Major && best.seventh == Some(Seventh::Dominant);
    let mut tags: Vec<Tag> = Vec::new();
    let mut push = |word, weight, why: String| {
        tags.push(Tag {
            word,
            weight,
            why,
            notes: Vec::new(),
        })
    };

    if cross || b9 {
        let why = if cross {
            let (t, s) = (v.lowest(4).expect("3rd"), v.lowest(3).expect("♯9"));
            let (lo, hi) = if t < s { (t, s) } else { (s, t) };
            format!(
                "Major 3rd ({}) against ♯9 ({}), a {} apart as voiced: one scale degree in two colours at once, a cross-relation.",
                nm(4),
                nm(3),
                interval_name(v.notes[hi] - v.notes[lo])
            )
        } else {
            format!(
                "♭9 ({}) over a major 3rd{}: the root is shadowed by its upper semitone neighbour.",
                nm(1),
                if dominant { " and ♭7" } else { "" }
            )
        };
        push("aggressive", 1.0, why);
        if rel.has(7) {
            push(
                "harsh",
                0.95,
                format!(
                    "The root–5th frame ({}–{}) stays consonant, so the clash is exposed rather than blended.",
                    nm(0),
                    nm(7)
                ),
            );
        } else if rel.has(8) {
            push(
                "complex",
                0.95,
                format!(
                    "The ♯5 ({}) replaces the perfect 5th. With no stable frame the clash reads as layered and unresolved.",
                    nm(8)
                ),
            );
        }
    }

    if let Some(close) = pairs
        .iter()
        .filter(|p| p.semitones == 1)
        .max_by(|a, b| a.roughness.total_cmp(&b.roughness))
    {
        let degree = if close.roughness >= 0.8 {
            "near-maximal roughness".to_string()
        } else {
            format!(
                "rough, though the register softens it (r = {:.2})",
                close.roughness
            )
        };
        push(
            "grinding",
            0.85,
            format!(
                "{}–{} sit a semitone apart: {degree}.",
                v.name(close.i),
                v.name(close.j)
            ),
        );
    }

    let symmetry = set.transpositional_symmetry();
    if symmetry > 1 {
        push(
            "symmetric",
            0.75,
            format!(
                "Transposing by {} semitones gives the same notes: the chord divides the octave evenly, so no note is privileged as root.",
                12 / symmetry
            ),
        );
    }

    if ambiguity > 0.55 {
        let (a, b) = (&all[0], &all[1]);
        let why = if b.prob / a.prob > 0.8 {
            format!(
                "Reads almost equally as {} and {}; the ear can hear either root.",
                a.symbol, b.symbol
            )
        } else {
            format!(
                "Also reads convincingly as {} ({:.0}% against {:.0}% for {}).",
                b.symbol,
                b.prob * 100.0,
                a.prob * 100.0,
                a.symbol
            )
        };
        push("ambiguous", 0.8, why);
    }

    if matches!(pos, Position::Second) {
        let why = format!(
            "The 5th ({}) is in the bass, so the root ({}) sits a 4th above it: a six-four, which traditionally leans toward resolution.",
            nm(7),
            nm(0)
        );
        push("unsettled", 0.72, why);
    } else if matches!(pos, Position::Third) {
        let ic = v.best.root.up_to(harmony::PitchClass::of_midi(v.notes[0]));
        push(
            "unsettled",
            0.72,
            format!(
                "The 7th ({}) is in the bass, a dissonance under the whole chord that traditionally steps down.",
                nm(ic)
            ),
        );
    }

    if best.third == Third::Minor && rel.has(8) && best.seventh.is_none() {
        push(
            "dark",
            0.7,
            format!(
                "Minor 3rd plus ♭6 ({}): both are lowered neighbours, the darkest pairing over a minor frame.",
                nm(8)
            ),
        );
    }

    if best.seventh == Some(Seventh::Major) && best.third == Third::Major && !cross {
        // Is a 7th voiced directly under a root (m2 or m9)?
        let clash = v.with(11).into_iter().find_map(|s| {
            v.with(0)
                .into_iter()
                .find(|&r| r > s && matches!(v.notes[r] - v.notes[s], 1 | 13))
                .map(|r| (s, r))
        });
        match clash {
            Some((s, r)) => push(
                "piquant",
                0.7,
                format!(
                    "The major 7th ({}) sits under the root ({}) as a {}: the semitone clash is exposed, not cushioned.",
                    v.name(s),
                    v.name(r),
                    interval_name(v.notes[r] - v.notes[s])
                ),
            ),
            None => push(
                "lush",
                0.7,
                format!(
                    "Major 7th ({}) a semitone short of the root's octave; the friction stays soft because the triad beneath is consonant.",
                    nm(11)
                ),
            ),
        }
    }

    if rel.has(4) && rel.has(10) {
        let (t, s) = (v.lowest(4).expect("3rd"), v.lowest(10).expect("♭7"));
        let how = if t < s {
            "voiced as a diminished 5th, the pair closes inward when it resolves"
        } else {
            "voiced as an augmented 4th, the pair opens outward when it resolves"
        };
        push(
            "driving",
            0.65,
            format!("Tritone between 3 ({}) and ♭7 ({}): {how}.", nm(4), nm(10)),
        );
    }

    if best.degrees[6] == Some(Degree::SharpEleven) && best.third == Third::Major {
        push(
            "luminous",
            0.6,
            format!(
                "♯11 ({}) raises the 4th, removing its friction with the 3rd.",
                nm(6)
            ),
        );
    }

    if matches!(best.third, Third::Sus4 | Third::Sus2) {
        push(
            "suspended",
            0.6,
            "No 3rd: the chord withholds major or minor.".into(),
        );
    }

    // Lahdelma & Eerola tested plain seventh chords: four pitch classes, no extensions.
    let plain = best.alterations == 0 && best.perfect_fifth && rel.len() == 4;
    let m7 = best.third == Third::Minor && best.seventh == Some(Seventh::Dominant);
    let maj7 = best.third == Third::Major && best.seventh == Some(Seventh::Major);
    if plain && (m7 || maj7) {
        push(
            "bittersweet",
            0.55,
            "Listeners hear minor-7th and major-7th chords as nostalgic, longing (Lahdelma & Eerola 2016): consonant enough to be warm, with one mild clash.".into(),
        );
    }

    let n = v.notes.len();
    let adj: Vec<u8> = v.notes.windows(2).map(|w| w[1] - w[0]).collect();
    if n >= 3 && adj.iter().all(|&d| d == 5 || d == 6) {
        let aug = if adj.contains(&6) {
            " (not all perfect)"
        } else {
            ""
        };
        push(
            "open",
            0.6,
            format!("Stacked 4ths{aug}: no triadic centre, evenly spaced and modern."),
        );
    }
    if n >= 3 && adj.iter().all(|&d| d == 3) {
        let why = if symmetry == 4 {
            "Stacked minor 3rds divide the octave symmetrically; every note could be the root."
                .to_string()
        } else {
            format!(
                "Stacked minor 3rds: a diminished sonority whose tritone ({}–{}) wants to resolve.",
                v.name(0),
                v.name(2)
            )
        };
        push("unstable", 0.7, why);
    }

    let et = opts.tuning == Tuning::Et;
    let approx = if et { "approximate " } else { "are " };
    let worst = p.cents.iter().fold(0.0f64, |m, c| m.max(c.abs()));
    let detune = if et && worst >= 1.0 {
        format!(" (12-TET detunes them by up to {worst:.0}¢)")
    } else {
        String::new()
    };
    let h = p.harmonics_text(":");
    let pcs = rel.len();
    // Periodicity ladder: how quickly the notes' common period comes round, in bass cycles.
    match p.bass_cycles {
        1 if pcs == 1 => push("doubled", 0.5, "Only octave doublings of one pitch class: every partial of the upper notes is already a partial of the bass.".into()),
        1 => push(
            "overtone",
            0.5,
            format!("The upper notes {approx}harmonics {h} of the bass itself: they lie on its own overtone series."),
        ),
        2 | 3 if pcs == 2 => push(
            "hollow",
            0.55,
            format!("A bare {} with nothing between: the notes {approx}harmonics {h} of one fundamental, the plainest relation after the octave.", if p.bass_cycles == 2 { "fifth" } else { "fourth" }),
        ),
        2..=4 => push(
            "fused",
            0.5,
            format!("The notes {approx}harmonics {h} of one fundamental{detune}: the period is so short they merge into a single tone-colour."),
        ),
        5..=8 => push(
            "blended",
            0.45,
            format!("Harmonics {h} of one fundamental{detune}: a short common period, so the notes blend while staying distinct."),
        ),
        40..=119 => push(
            "cloudy",
            0.5,
            if et {
                format!("Long common period: even the just approximation needs {} bass cycles to repeat, and in 12-TET the summed wave never repeats exactly.", p.bass_cycles)
            } else {
                format!("Long common period ({} bass cycles): the summed wave takes a long time to repeat.", p.bass_cycles)
            },
        ),
        c if c >= 120 => push(
            "diffuse",
            0.55,
            format!("A common period of {c} bass cycles: no audible shared fundamental, the notes are heard as separate strands rather than one sonority."),
        ),
        _ => {}
    }
    // Where the harmonic series points: the chord's own root, or elsewhere.
    if pcs >= 3 && (2..=24).contains(&p.bass_cycles) {
        let below = 12.0 * (p.bass_cycles as f64).log2();
        let fundamental = harmony::PitchClass::new(v.notes[0] as i32 - below.round() as i32);
        let name = best.tone_names[best.root.up_to(fundamental) as usize]
            .clone()
            .unwrap_or_else(|| opts.spelling.pc_name(fundamental).to_string());
        let octaves = (below / 12.0).round() as i32;
        if fundamental == best.root && best.root_present {
            push(
                "grounded",
                0.42,
                format!("The notes {approx}harmonics {h} of {name}, {octaves} octave{} below the bass: the harmonic series confirms the named root.", if octaves == 1 { "" } else { "s" }),
            );
        } else if fundamental != best.root {
            push(
                "offset",
                0.45,
                format!("The notes {approx}harmonics {h} of {name}, not of the named root {}: the harmonic series points elsewhere.", nm(0)),
            );
        }
    }
    // Register and spacing.
    let span = v.notes[n - 1] - v.notes[0];
    if let Some((i, j)) = (0..n.saturating_sub(1))
        .map(|i| (i, i + 1))
        .find(|&(i, j)| v.notes[j] - v.notes[i] <= 4 && v.notes[i] < 48)
    {
        push(
            "muddy",
            0.78,
            format!(
                "{}–{} is a {} below C3, under the low-interval limit: their partials crowd into the same critical bands.",
                v.names[i],
                v.names[j],
                interval_name(v.notes[j] - v.notes[i])
            ),
        );
    }
    if n >= 3 && span <= 12 && adj.iter().all(|&d| d <= 4) {
        push("close", 0.35, "Close position: every note within an octave, adjacent notes a third or less apart, so the sound is compact.".into());
    } else if n >= 3 && span >= 24 && adj.iter().any(|&d| d >= 10) {
        push(
            "spread",
            0.35,
            format!("Spans {:.1} octaves with gaps of up to {} semitones: each note stands apart in its own register.", span as f64 / 12.0, adj.iter().max().copied().unwrap_or(0)),
        );
    }
    let mean = v.notes.iter().map(|&m| m as f64).sum::<f64>() / n as f64;
    if v.notes[0] >= 67 {
        push("airy", 0.35, format!("Everything sits above G4 (mean pitch {}), where partials thin out and chords sound light.", harmony::Midi(mean.round() as u8).name(opts.spelling)));
    }

    // A plain triad's colour, only when nothing more specific was said.
    // Texture and spacing words don't count; harmonic-character words do.
    const TEXTURE: [&str; 14] = [
        "unsettled",
        "doubled",
        "overtone",
        "hollow",
        "fused",
        "blended",
        "cloudy",
        "diffuse",
        "grounded",
        "offset",
        "muddy",
        "close",
        "spread",
        "airy",
    ];
    let quiet = tags.iter().all(|t| TEXTURE.contains(&t.word));
    let fifth = if et { "perfect 5th" } else { "pure 5th" };
    if quiet && best.perfect_fifth && matches!(best.third, Third::Major | Third::Minor) {
        let major = best.third == Third::Major;
        let (word, quality) = if major {
            ("bright", "Major")
        } else {
            ("somber", "Minor")
        };
        let why = match pos {
            Position::Root if major => {
                format!("Major 3rd and {fifth} over the root in the bass: the reference consonance of the major triad.")
            }
            Position::Root => format!("Minor 3rd and {fifth} over the root in the bass: stable, but shaded."),
            Position::First => format!(
                "{quality} triad with its 3rd ({}) in the bass: consonant, but lighter and less anchored than root position.",
                nm(if major { 4 } else { 3 })
            ),
            Position::Second => format!(
                "{quality} triad over its 5th: {} colour without root-position stability.",
                if major { "bright" } else { "shaded" }
            ),
            _ => format!("{quality} triad: {} colour.", if major { "bright" } else { "shaded" }),
        };
        tags.push(Tag {
            notes: Vec::new(),
            word,
            weight: 0.48,
            why,
        });
    }

    for t in &mut tags {
        t.notes = evidence(v, t.word, pairs);
    }
    tags.sort_by(|a, b| b.weight.partial_cmp(&a.weight).expect("finite weights"));
    tags
}

/// The notes a tag's explanation talks about, so they can be played on their own.
fn evidence(v: &Voicing, word: &str, pairs: &[Pair]) -> Vec<usize> {
    let two = |a: u8, b: u8| -> Vec<usize> { v.lowest(a).into_iter().chain(v.lowest(b)).collect() };
    let rel = v.best.rel;
    match word {
        "aggressive" if rel.has(3) && rel.has(4) => two(4, 3),
        "aggressive" => two(1, 4),
        "harsh" => two(0, 7),
        "complex" => two(4, 8),
        "grinding" => pairs
            .iter()
            .filter(|p| p.semitones == 1)
            .max_by(|a, b| a.roughness.total_cmp(&b.roughness))
            .map_or(Vec::new(), |p| vec![p.i, p.j]),
        "unsettled" => std::iter::once(0).chain(v.lowest(0)).collect(),
        "dark" => two(3, 8),
        "lush" | "piquant" => two(11, 0),
        "driving" => two(4, 10),
        "luminous" => two(4, 6),
        _ => Vec::new(),
    }
}
