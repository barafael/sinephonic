//! The improved model: the prototype's structure, with
//!
//! * chord tones spelled on the line of fifths (C°7 = C E♭ G♭ B𝄫, ♯9 over C = D♯);
//! * a structured symbol builder (7♭5, 13, 9sus4, ø9, power chords; no `°(maj7)♭7`);
//! * root scores that add Parncutt's root support and Hindemith's interval roots to the
//!   template fit;
//! * complexity from information measures: harmonic entropy, period bits, root and interval
//!   entropy (see [`crate::information`]);
//! * set-class identity, named sonorities, Huron consonance, Tenney height, Euler's gradus and
//!   Cook & Fujisawa tension/modality as extras;
//! * tags whose explanations are checked against the voicing (see [`character`]), including
//!   *unsettled* (six-four, 7th in the bass), *piquant*, *symmetric* and *bittersweet*;
//! * a stability penalty for six-fours and 7ths in the bass;
//! * roughness as the excess over the unison baseline ([`crate::roughness::excess_roughness`]).

pub mod character;
pub mod metrics;
pub mod reading;
pub mod sonority;

use harmony::ratio::{gradus, lcm};
use harmony::{Midi, PcSet, PitchClass};

use crate::periodicity::{beats, smoothed_log_periodicity, Periodicity};
use crate::reference::harshness;
use crate::{clamp01, distinct_pcs, max_partial, Analysis, Axes, Extras, Options};
use character::{position, Position, Voicing};

pub fn analyze(notes: &[u8], opts: &Options) -> Analysis {
    let bass_pc = PitchClass::of_midi(notes[0]);
    let pcs = distinct_pcs(notes);
    let set = PcSet::from_midi(notes);
    let p = Periodicity::new(notes, opts.ratio_set, opts.a4);
    let all = reading::readings(notes, opts.spelling, &reading::WEIGHTS);
    let best = &all[0];
    let pairs = excess_pairs(notes, &p, opts);
    let smoothed = smoothed_log_periodicity(notes, opts.ratio_set);

    // The prototype's 1 − exp(−0.9·Σr) saturates: a close C3 major triad scored 0.85 and every
    // five-note chord ≈ 1. Instead blend the mean sensory roughness per pair (register-aware)
    // with Huron's harmonic dissonance (register-free) and a small size term.
    let mean_r = pairs.iter().map(|p| p.roughness).sum::<f64>() / pairs.len() as f64;
    let sensory = clamp01((mean_r - 0.15) / 0.8);
    let harmonic = clamp01(metrics::huron_dissonance(&pairs) / 0.6);
    let size = clamp01((pcs.len() as f64 - 3.0) / 4.0);
    let tension = clamp01(0.45 * sensory + 0.45 * harmonic + 0.1 * size);
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
    // Replaced below by the information-theoretic composite; arousal is updated with it.
    let complexity = 0.0;
    let brightness = brightness(notes, best);
    // A six-four or a 7th in the bass is traditionally unstable.
    let unsettled = matches!(
        position(&Voicing {
            notes,
            best,
            names: &[]
        }),
        Position::Second | Position::Third
    );
    // Structural stability: the chord's frame, independent of how rough it sounds.
    let structure = clamp01(
        0.3 + if fifth_frame { 0.25 } else { 0.0 }
            + if best.root == bass_pc { 0.2 } else { 0.0 }
            + if best.root_present { 0.1 } else { 0.0 }
            - if unsettled { 0.1 } else { 0.0 }
            + if p.bass_cycles <= 6 { 0.15 } else { 0.0 },
    );
    let stability = clamp01(structure - 0.35 * tension - 0.15 * ambiguity);
    let aggression = clamp01(
        0.45 * if cross { 1.0 } else { 0.0 }
            + 0.3 * harsh
            + 0.25 * tension
            + if b9dom { 0.25 } else { 0.0 },
    );
    // Valence and arousal are kept apart: valence from colour (brightness) and structure,
    // arousal from tension, aggression, complexity and structural instability. The prototype
    // subtracted harshness and tension from valence, which left "bright but tense" (Lydian,
    // ♯11 dominants) almost empty.
    let valence = clamp01(brightness + 0.1 * (structure - 0.6) - 0.1 * (harsh - 0.3));
    let arousal = mood_arousal(tension, aggression, complexity, structure);

    let support = metrics::parncutt_support(set);
    let freqs = match opts.tuning {
        crate::Tuning::Just => &p.just_freqs,
        crate::Tuning::Et => &p.et_freqs,
    };
    let (cook_tension, cook_modality) = metrics::cook_fujisawa(freqs);
    let beats = beats(notes, opts.ratio_set, freqs, max_partial(opts.timbre));
    let iv = set.interval_vector();
    let extras = Extras {
        forte: set.forte().map(|f| f.to_string()).unwrap_or_default(),
        prime_form: set.prime_form().to_string(),
        interval_vector: iv,
        sonority: sonority::named(notes),
        root_ambiguity: metrics::parncutt_ambiguity(&support),
        smoothed_periodicity: smoothed,
        huron: metrics::huron_consonance(iv),
        tenney: p.harmonics.iter().map(|&h| (h as f64).log2()).sum(),
        gradus: gradus(p.harmonics.iter().copied().fold(1, lcm)),
        cook_tension,
        cook_modality,
    };

    let note_names: Vec<String> = notes
        .iter()
        .map(|&m| {
            let pc = PitchClass::of_midi(m);
            match best.degree_of(pc) {
                Some(d) => {
                    let s = harmony::SpelledPc::parse(&best.root_name).expect("root name parses")
                        + d.interval();
                    Midi(m).spelled_name(s)
                }
                None => Midi(m).name(opts.spelling),
            }
        })
        .collect();

    let voicing = Voicing {
        notes,
        best: &all[0],
        names: &note_names,
    };
    let tags = character::tags(&voicing, &all, &pairs, &p, ambiguity, set, opts);
    let mut a = Analysis {
        beats,
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
        note_names,
        extras: Some(extras),
        information: None,
    };
    let info = crate::information::measure(&a, opts.tuning, opts.timbre, smoothed);
    a.axes.complexity = info.complexity;
    a.arousal = mood_arousal(
        a.axes.tension,
        a.axes.aggression,
        info.complexity,
        structure,
    );
    a.information = Some(info);
    a
}

/// Pairs with roughness measured as the excess over the unison baseline.
fn excess_pairs(notes: &[u8], p: &Periodicity, opts: &Options) -> Vec<crate::Pair> {
    let fr = match opts.tuning {
        crate::Tuning::Just => &p.just_freqs,
        crate::Tuning::Et => &p.et_freqs,
    };
    let mut out = Vec::new();
    for i in 0..notes.len() {
        for j in i + 1..notes.len() {
            out.push(crate::Pair {
                i,
                j,
                semitones: notes[j] - notes[i],
                roughness: crate::roughness::normalized_excess(fr[i], fr[j], opts.timbre, opts.a4),
            });
        }
    }
    out
}

fn mood_arousal(tension: f64, aggression: f64, complexity: f64, structure: f64) -> f64 {
    // Scaled so the calmest consonances sit near 0.25 and dense dissonances reach the top.
    clamp01(
        1.15 * (0.45 * tension + 0.2 * aggression + 0.2 * complexity + 0.15 * (1.0 - structure)),
    )
}

/// Position of a chord degree on the line of fifths relative to the root, clamped to ±6:
/// the same axis that orders the modes from Locrian (dark) to Lydian (bright). A ♯9 counts as
/// its blue-note ♭3; a ♯5 sits halfway between its sharp spelling (+8) and ♭6 (−4), since
/// the augmented triad it belongs to is symmetric.
fn fifths(d: crate::Degree) -> f64 {
    match d {
        crate::Degree::SharpNine => -3.0,
        crate::Degree::SharpFifth => 2.0,
        d => d.interval().fifths().clamp(-6, 6) as f64,
    }
}

/// Brightness: the mean line-of-fifths position of the chord tones above the root (Lydian
/// ♯11 +6, major 3rd +4, minor 3rd −3, ♭5 −6), plus a register term.
pub fn brightness(notes: &[u8], best: &crate::Reading) -> f64 {
    let tones: Vec<f64> = distinct_pcs(notes)
        .into_iter()
        .filter_map(|pc| best.degree_of(pc))
        .filter(|&d| d != crate::Degree::Root)
        .map(fifths)
        .collect();
    let colour = if tones.is_empty() {
        0.0
    } else {
        tones.iter().sum::<f64>() / tones.len() as f64 / 6.0
    };
    let register =
        (notes.iter().map(|&m| m as f64).sum::<f64>() / notes.len() as f64 - 60.0) / 24.0;
    clamp01(0.5 + 0.5 * colour + 0.15 * register)
}
