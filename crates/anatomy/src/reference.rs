//! Exact port of the design prototype's analysis (`ANALYSIS.md`, and the `<script>` block of
//! `Chord Anatomy.dc.html`). Floating-point operations follow the JavaScript's order so the
//! differential test in `anatomy-oracle` can compare to 1e-9.
//!
//! The weights are the prototype's design choices, not psychoacoustic constants.

use harmony::{PcSet, PitchClass, Spelling};

use crate::degree::{Degree, Seventh, Third};
use crate::periodicity::Periodicity;
use crate::{
    clamp01, distinct_pcs, roughness, Analysis, Axes, Options, Pair, Reading, Tag, Tuning,
};

pub const BRIGHT: [f64; 12] = [
    0.0, -1.6, 0.4, -1.2, 1.3, -0.2, 0.8, 0.1, -1.3, 0.9, -0.5, 1.2,
];

/// The prototype's reading of the chord from `root`.
pub fn reading(
    root: PitchClass,
    pcs: &[PitchClass],
    bass: PitchClass,
    spelling: Spelling,
) -> Reading {
    let rel: PcSet = pcs
        .iter()
        .map(|&p| PitchClass::new(root.up_to(p) as i32))
        .collect();
    let h = |i: u8| rel.has(i);
    let p5 = h(7);
    let has_sev = h(11) || h(10);
    let third = if h(4) {
        Third::Major
    } else if h(3) {
        Third::Minor
    } else if h(5) {
        Third::Sus4
    } else if !has_sev && h(2) {
        Third::Sus2
    } else {
        Third::None
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
    let dim = third == Third::Minor && h(6) && !p5;
    let aug = third == Third::Major && h(8) && !p5;
    if p5 {
        deg[7] = Some(Degree::Fifth);
    }
    if dim {
        deg[6] = Some(Degree::FlatFifth);
    }
    if aug {
        deg[8] = Some(Degree::SharpFifth);
    }
    let sev = if h(11) {
        deg[11] = Some(Degree::Seventh);
        Some(Seventh::Major)
    } else if h(10) {
        deg[10] = Some(Degree::FlatSeventh);
        Some(Seventh::Dominant)
    } else if dim && h(9) {
        deg[9] = Some(Degree::DoubleFlatSeventh);
        Some(Seventh::Diminished)
    } else {
        None
    };
    let mut nat: Vec<u8> = Vec::new();
    let mut alts: Vec<&str> = Vec::new();
    if h(1) {
        deg[1] = Some(Degree::FlatNine);
        alts.push("♭9");
    }
    if h(2) && deg[2].is_none() {
        deg[2] = Some(Degree::Nine);
        nat.push(9);
    }
    if h(3) && deg[3].is_none() {
        deg[3] = Some(Degree::SharpNine);
        alts.push("♯9");
    }
    if h(5) && deg[5].is_none() {
        deg[5] = Some(Degree::Eleven);
        nat.push(11);
    }
    if h(6) && deg[6].is_none() {
        if p5 || sev.is_some() {
            deg[6] = Some(Degree::SharpEleven);
            alts.push("♯11");
        } else {
            deg[6] = Some(Degree::FlatFifth);
            alts.push("♭5");
        }
    }
    if h(8) && deg[8].is_none() {
        if sev.is_some() {
            deg[8] = Some(Degree::FlatThirteen);
            alts.push("♭13");
        } else {
            deg[8] = Some(Degree::FlatSix);
            alts.push("♭6");
        }
    }
    if h(9) && deg[9].is_none() {
        if sev.is_some() {
            deg[9] = Some(Degree::Thirteen);
            nat.push(13);
        } else {
            deg[9] = Some(Degree::Six);
            nat.push(6);
        }
    }
    if h(10) && deg[10].is_none() {
        deg[10] = Some(Degree::FlatSeventh);
        alts.push("♭7");
    }

    let hi = match sev {
        Some(Seventh::Major | Seventh::Dominant) => {
            nat.iter().copied().filter(|&x| x >= 9).fold(7, u8::max)
        }
        _ => 7,
    };
    let name = |pc: PitchClass| spelling.pc_name(pc);
    let mut s = String::from(name(root));
    if dim {
        s += match sev {
            Some(Seventh::Diminished) => "°7",
            Some(Seventh::Dominant) => "ø7",
            Some(Seventh::Major) => "°(maj7)",
            None => "°",
        };
    } else {
        if third == Third::Minor {
            s += "m";
        }
        match sev {
            Some(Seventh::Major) => {
                if third == Third::Minor {
                    s += &format!("(maj{hi})");
                } else {
                    s += &format!("maj{hi}");
                }
            }
            Some(Seventh::Dominant) => s += &hi.to_string(),
            _ => {
                if aug {
                    s += "+";
                }
                let six = nat.contains(&6);
                let nine = nat.contains(&9);
                s += match (six, nine) {
                    (true, true) => "6/9",
                    (true, false) => "6",
                    (false, true) => "add9",
                    (false, false) => "",
                };
                if nat.contains(&11) {
                    s += "add11";
                }
            }
        }
    }
    let mut alt_list: Vec<&str> = if aug && sev.is_some() {
        vec!["♯5"]
    } else {
        vec![]
    };
    alt_list.extend(alts);
    match third {
        Third::Sus4 => s += "sus4",
        Third::Sus2 => s += "sus2",
        _ => {}
    }
    if !alt_list.is_empty() {
        if sev.is_some() {
            s += &alt_list.concat();
        } else {
            s += &format!("({})", alt_list.join(","));
        }
    }
    if third == Third::None {
        s += "(no3)";
    }
    if bass != root {
        s += "/";
        s += name(bass);
    }
    let root_present = h(0);
    let score = (if root_present { 3.0 } else { -1.5 })
        + (if root == bass { 2.0 } else { 0.0 })
        + (match third {
            Third::Major | Third::Minor => 2.0,
            Third::None => -2.5,
            _ => -0.5,
        })
        + (if p5 {
            1.0
        } else if dim || aug {
            0.3
        } else {
            0.0
        })
        + (if sev.is_some() { 0.8 } else { 0.0 })
        - 0.4 * nat.len() as f64
        - 0.9 * alt_list.len() as f64;

    let tone_names = std::array::from_fn(|ic| {
        rel.has(ic as u8)
            .then(|| name(root + ic as i32).to_string())
    });
    Reading {
        root,
        root_name: name(root).into(),
        harte: String::new(),
        symbol: s,
        score,
        prob: 0.0,
        root_present,
        third,
        seventh: sev,
        perfect_fifth: p5,
        diminished: dim,
        augmented: aug,
        alterations: alt_list.len(),
        rel,
        degrees: deg,
        tone_names,
    }
}

/// All 12 readings, sorted by score (stable, like `Array.prototype.sort`), with softmax
/// probabilities at temperature 1/0.8.
pub fn readings(pcs: &[PitchClass], bass: PitchClass, spelling: Spelling) -> Vec<Reading> {
    let mut all: Vec<Reading> = (0..12)
        .map(|r| reading(PitchClass::new(r), pcs, bass, spelling))
        .collect();
    sort_and_normalize(&mut all, 0.8);
    all
}

pub(crate) fn sort_and_normalize(all: &mut [Reading], beta: f64) {
    all.sort_by(|a, b| b.score.partial_cmp(&a.score).expect("finite scores"));
    let ex: Vec<f64> = all.iter().map(|r| (r.score * beta).exp()).collect();
    let z: f64 = ex.iter().sum();
    for (r, e) in all.iter_mut().zip(ex) {
        r.prob = e / z;
    }
}

/// Every pair of notes with its roughness in the current tuning and timbre.
pub fn pairs(notes: &[u8], p: &Periodicity, opts: &Options) -> Vec<Pair> {
    let fr = match opts.tuning {
        Tuning::Just => &p.just_freqs,
        Tuning::Et => &p.et_freqs,
    };
    let reference = roughness::reference(opts.a4, opts.timbre);
    let n = notes.len();
    let mut out = Vec::with_capacity(n * (n - 1) / 2);
    for i in 0..n {
        for j in i + 1..n {
            out.push(Pair {
                i,
                j,
                semitones: notes[j] - notes[i],
                roughness: roughness::normalized(fr[i], fr[j], opts.timbre, reference),
            });
        }
    }
    out
}

pub fn harshness(pairs: &[Pair]) -> f64 {
    let mut harsh = 0.0;
    for p in pairs {
        let ic = p.semitones % 12;
        if ic == 1 || ic == 11 {
            harsh += match p.semitones {
                1 => 1.0,
                13 => 0.75,
                11 => 0.55,
                _ => 0.35,
            };
        } else if ic == 6 {
            harsh += 0.25;
        } else if p.semitones == 2 {
            harsh += 0.3;
        }
    }
    let fifth_frame = pairs.iter().any(|p| p.semitones % 12 == 7);
    clamp01(harsh / 1.6 * if fifth_frame { 1.2 } else { 1.0 })
}

pub fn analyze(notes: &[u8], opts: &Options) -> Analysis {
    let n = notes.len();
    let bass_pc = PitchClass::of_midi(notes[0]);
    let pcs = distinct_pcs(notes);
    let p = Periodicity::new(notes, opts.ratio_set, opts.a4);
    let all = readings(&pcs, bass_pc, opts.spelling);
    let best = &all[0];
    let pairs = pairs(notes, &p, opts);

    let sum_r: f64 = pairs.iter().map(|p| p.roughness).sum();
    let tension = clamp01(1.0 - (-sum_r * 0.9).exp());
    let harsh = harshness(&pairs);
    let fifth_frame = pairs.iter().any(|p| p.semitones % 12 == 7);
    let rel = best.rel;
    let cross = rel.has(3) && rel.has(4);
    let b9dom = rel.has(1) && rel.has(4);
    let ambiguity = if pcs.len() < 2 {
        0.0
    } else {
        clamp01(all[1].prob / all[0].prob * 1.1)
    };
    let period_cycles = p.bass_cycles as f64;
    let pn = clamp01(period_cycles.log2() / 7.0);
    let complexity = clamp01(
        0.4 * pn
            + 0.25 * clamp01(best.alterations as f64 / 3.0)
            + 0.15 * clamp01((pcs.len() as f64 - 3.0) / 4.0)
            + 0.1 * ambiguity
            + if !fifth_frame && tension > 0.3 {
                0.15
            } else {
                0.0
            },
    );
    let (mut br, mut c) = (0.0, 0);
    for &pc in &pcs {
        let ic = best.root.up_to(pc);
        if ic != 0 {
            br += BRIGHT[ic as usize];
            c += 1;
        }
    }
    let br = if c > 0 { br / c as f64 } else { 0.0 };
    let reg = (notes.iter().map(|&m| m as f64).sum::<f64>() / n as f64 - 60.0) / 24.0;
    let brightness = clamp01(0.5 + br / 2.4 + reg * 0.25);
    let stability = clamp01(
        0.3 + if fifth_frame { 0.25 } else { 0.0 }
            + if best.root == bass_pc { 0.2 } else { 0.0 }
            + if best.root_present { 0.1 } else { 0.0 }
            - 0.35 * tension
            - 0.15 * ambiguity
            + if p.bass_cycles <= 6 { 0.15 } else { 0.0 },
    );
    let aggression = clamp01(
        0.45 * if cross { 1.0 } else { 0.0 }
            + 0.3 * harsh
            + 0.25 * tension
            + if b9dom { 0.25 } else { 0.0 },
    );
    let valence = clamp01(brightness * 0.6 + stability * 0.4 - 0.25 * harsh);
    let arousal = clamp01(0.5 * tension + 0.3 * aggression + 0.2 * complexity);

    let tags = tags(notes, &pcs, &all, &pairs, &p, ambiguity, opts.spelling);
    let freqs = match opts.tuning {
        Tuning::Just => &p.just_freqs,
        Tuning::Et => &p.et_freqs,
    };
    let beats = crate::periodicity::beats(
        notes,
        opts.ratio_set,
        freqs,
        crate::max_partial(opts.timbre),
    );
    let mut a = Analysis {
        beats,
        note_names: notes
            .iter()
            .map(|&m| harmony::Midi(m).name(opts.spelling))
            .collect(),
        notes: notes.to_vec(),
        pcs,
        bass_pc,
        periodicity: p,
        readings: all,
        pairs,
        axes: Axes {
            tension,
            harshness: harsh,
            aggression,
            complexity,
            brightness,
            stability,
            ambiguity,
        },
        valence,
        arousal,
        tags,
        extras: None,
        information: None,
    };
    let smoothed = crate::periodicity::smoothed_log_periodicity(notes, opts.ratio_set);
    a.information = Some(crate::information::measure(
        &a,
        opts.tuning,
        opts.timbre,
        smoothed,
    ));
    a
}

fn tags(
    notes: &[u8],
    pcs: &[PitchClass],
    all: &[Reading],
    pairs: &[Pair],
    p: &Periodicity,
    ambiguity: f64,
    spelling: Spelling,
) -> Vec<Tag> {
    let _ = pcs;
    let best = &all[0];
    let rel = best.rel;
    let r = best.root;
    let nm = |i: i32| spelling.pc_name(r + i);
    let cross = rel.has(3) && rel.has(4);
    let b9dom = rel.has(1) && rel.has(4);
    let mut tags: Vec<Tag> = Vec::new();
    let mut push = |word, weight, why: String| {
        tags.push(Tag {
            word,
            weight,
            why,
            notes: Vec::new(),
        })
    };
    if cross || b9dom {
        push(
            "aggressive",
            1.0,
            if cross {
                format!(
                    "Major 3rd ({}) against ♯9 ({}): one scale degree in two colours at once, a cross-relation a semitone apart.",
                    nm(4),
                    nm(3)
                )
            } else {
                format!(
                    "♭9 ({}) over a major-3rd dominant: the root is shadowed a semitone above.",
                    nm(1)
                )
            },
        );
        if rel.has(7) {
            push(
                "harsh",
                0.95,
                format!(
                    "The pure 5th ({}–{}) keeps the frame stable, so the clash is exposed rather than blended.",
                    nm(0),
                    nm(7)
                ),
            );
        } else if rel.has(8) {
            push(
                "complex",
                0.95,
                format!(
                    "The ♯5 ({}) replaces the pure 5th. With no stable frame the clash reads as layered and unresolved.",
                    nm(8)
                ),
            );
        }
    }
    if let Some(close) = pairs.iter().find(|p| p.semitones == 1) {
        push(
            "grinding",
            0.85,
            format!(
                "{}–{} sit a semitone apart in the same register: near-maximal roughness.",
                spelling.pc_name(PitchClass::of_midi(notes[close.i])),
                spelling.pc_name(PitchClass::of_midi(notes[close.j]))
            ),
        );
    }
    if ambiguity > 0.55 {
        push(
            "ambiguous",
            0.8,
            format!(
                "Reads almost equally as {} and {}; the ear can hear either root.",
                all[0].symbol, all[1].symbol
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
        push(
            "lush",
            0.7,
            format!(
                "Major 7th ({}) sits a semitone under the octave; the friction stays soft because the triad beneath is consonant.",
                nm(11)
            ),
        );
    }
    if rel.has(4) && rel.has(10) {
        push(
            "driving",
            0.65,
            format!(
                "Tritone between 3 ({}) and ♭7 ({}) pulls inward toward resolution.",
                nm(4),
                nm(10)
            ),
        );
    }
    if best.degrees[6] == Some(Degree::SharpEleven) {
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
    let n = notes.len();
    let adj: Vec<u8> = notes.windows(2).map(|w| w[1] - w[0]).collect();
    if n >= 3 && adj.iter().all(|&d| d == 5 || d == 6) {
        push(
            "open",
            0.6,
            "Stacked 4ths: no triadic centre, evenly spaced and modern.".into(),
        );
    }
    if n >= 3 && adj.iter().all(|&d| d == 3) {
        push(
            "unstable",
            0.7,
            "Stacked minor 3rds divide the octave symmetrically; every note could be the root."
                .into(),
        );
    }
    if p.bass_cycles <= 6 && n > 1 {
        push(
            "fused",
            0.5,
            format!(
                "Low periodicity: the notes line up as harmonics {}, heard almost as one tone-colour.",
                p.harmonics_text(":")
            ),
        );
    } else if p.bass_cycles >= 40 {
        push(
            "cloudy",
            0.5,
            format!(
                "Long common period ({} bass cycles): the summed wave takes a long time to repeat.",
                p.bass_cycles
            ),
        );
    }
    if tags.is_empty() && best.third == Third::Major && best.perfect_fifth {
        tags.push(Tag {
            notes: Vec::new(),
            word: "bright",
            weight: 0.4,
            why: "Major 3rd over a pure 5th: the reference consonance of the major triad.".into(),
        });
    }
    if tags.is_empty() && best.third == Third::Minor && best.perfect_fifth {
        tags.push(Tag {
            notes: Vec::new(),
            word: "somber",
            weight: 0.4,
            why: "Minor 3rd over a pure 5th: stable, but shaded.".into(),
        });
    }
    tags.sort_by(|a, b| b.weight.partial_cmp(&a.weight).expect("finite weights"));
    tags
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{interval_name, Options};

    fn run(notes: &[u8]) -> Analysis {
        crate::analyze(notes, &Options::prototype()).unwrap()
    }

    fn top(a: &Analysis, k: usize) -> Vec<(String, f64)> {
        a.readings
            .iter()
            .take(k)
            .map(|r| (r.symbol.clone(), (r.score * 10.0).round() / 10.0))
            .collect()
    }

    fn words(a: &Analysis) -> Vec<&'static str> {
        a.tags.iter().map(|t| t.word).collect()
    }

    #[test]
    fn default_voicing() {
        let a = run(&[48, 55, 56, 63]);
        assert_eq!(a.periodicity.harmonics, [10, 15, 16, 24]);
        assert_eq!(a.periodicity.bass_cycles, 10);
        assert!((a.periodicity.period * 1000.0 - 76.45).abs() < 0.005);
        assert_eq!(
            top(&a, 3),
            [
                ("Cm(♭6)".into(), 7.1),
                ("A♭maj7/C".into(), 6.8),
                ("E♭6add11/C".into(), 4.2)
            ]
        );
        let fm9 = a.readings.iter().find(|r| r.symbol == "Fm9/C").unwrap();
        assert!((fm9.score - 1.9).abs() < 1e-9);
        assert!(!fm9.root_present);
        let r = &a.readings[10];
        assert_eq!(r.symbol, "D♭maj9♯11(no3)/C");
        assert!((r.score - -3.5).abs() < 1e-9);
        assert_eq!(&words(&a)[..3], ["grinding", "ambiguous", "dark"]);
        assert_eq!(a.summary(), "Grinding and ambiguous");
        // G3–A♭3 is clamped to 1.
        assert_eq!(a.pair(1, 2).unwrap().roughness, 1.0);
    }

    #[test]
    fn c7_sharp9() {
        let a = run(&[48, 52, 55, 58, 63]);
        assert_eq!(a.periodicity.harmonics, [20, 25, 30, 36, 48]);
        assert!((a.periodicity.period * 1000.0 - 152.9).abs() < 0.05);
        assert_eq!(top(&a, 1), [("C7♯9".into(), 7.9)]);
        assert_eq!(words(&a), ["aggressive", "harsh", "driving"]);
    }

    #[test]
    fn c7_sharp9_sharp5() {
        let a = run(&[48, 52, 56, 58, 63]);
        assert_eq!(a.periodicity.harmonics, [20, 25, 32, 36, 48]);
        assert_eq!(top(&a, 1), [("C7♯5♯9".into(), 6.3)]);
        assert_eq!(words(&a), ["aggressive", "complex", "driving"]);
    }

    #[test]
    fn c_major() {
        let a = run(&[48, 52, 55]);
        assert_eq!(a.periodicity.harmonics, [4, 5, 6]);
        assert!((a.periodicity.period * 1000.0 - 30.58).abs() < 0.005);
        assert_eq!(
            top(&a, 3),
            [
                ("C".into(), 8.0),
                ("Em(♭6)/C".into(), 4.1),
                ("Am7/C".into(), 2.3)
            ]
        );
        assert_eq!(words(&a), ["fused"]);
    }

    #[test]
    fn interval_names() {
        assert_eq!(interval_name(0), "P1");
        assert_eq!(interval_name(6), "TT");
        assert_eq!(interval_name(15), "m10");
        assert_eq!(interval_name(18), "A11");
        assert_eq!(interval_name(24), "P15");
        assert_eq!(interval_name(27), "m3+2oct");
    }

    #[test]
    fn known_awkward_symbols_are_reproduced() {
        // ANALYSIS.md lists these as known limitations of the prototype's builder.
        let a = run(&[48, 55, 56, 63]);
        let names: Vec<&str> = a.readings.iter().map(|r| r.symbol.as_str()).collect();
        assert!(
            names.contains(&"A°(maj7)♭7/C") || names.contains(&"E♭6sus4(♭9)/C"),
            "{names:?}"
        );
    }
}
