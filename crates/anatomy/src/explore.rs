//! Exploration tools: chords one small voice-leading step away, the partial spectrum on the
//! chord's harmonic series, and the dyadic dissonance curve.

use crate::roughness::{self, partials, Timbre};
use crate::{analyze, Analysis, Options, Tuning};

/// A voicing reached by moving one note.
#[derive(Clone, PartialEq, Debug)]
pub struct Neighbour {
    pub notes: Vec<u8>,
    /// Index of the moved note in the original voicing.
    pub moved: usize,
    /// Semitones moved: ±1 or ±2.
    pub step: i8,
    pub symbol: String,
    pub tension: f64,
}

/// Parsimonious voice leading (after Cohn and Tymoczko): every voicing reached by moving a
/// single note by a semitone or a whole tone without landing on another note. Neighbours are
/// in note order, then step order (−2, −1, +1, +2).
pub fn neighbours(notes: &[u8], opts: &Options) -> Vec<Neighbour> {
    let mut out = Vec::new();
    for (i, &m) in notes.iter().enumerate() {
        for step in [-2i8, -1, 1, 2] {
            let to = m as i16 + step as i16;
            if !(0..128).contains(&to) || notes.contains(&(to as u8)) {
                continue;
            }
            let mut v = notes.to_vec();
            v[i] = to as u8;
            v.sort_unstable();
            if let Some(a) = analyze(&v, opts) {
                out.push(Neighbour {
                    notes: v,
                    moved: i,
                    step,
                    symbol: a.best().symbol.clone(),
                    tension: a.axes.tension,
                });
            }
        }
    }
    out
}

/// One partial of one note.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Partial {
    pub note: usize,
    /// Partial number, 1 = fundamental.
    pub k: u32,
    pub freq: f64,
    pub amp: f64,
    /// Its place on the chord's just harmonic series: harmonics[note] · k.
    pub harmonic: u64,
}

/// Two partials of different notes on the same harmonic of the common fundamental.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Coincidence {
    pub lower: Partial,
    pub upper: Partial,
    /// Beat rate in the current tuning, |f₁ − f₂| (0 in just intonation).
    pub beat: f64,
}

/// Every partial of every note, and the partials that coincide in just intonation. In 12-TET
/// the coincidences separate and beat; this is how temperament shows up in the sound.
pub fn spectrum(a: &Analysis, tuning: Tuning, timbre: Timbre) -> (Vec<Partial>, Vec<Coincidence>) {
    let freqs = a.freqs(tuning);
    let mut parts = Vec::new();
    for (i, &f) in freqs.iter().enumerate() {
        for (k, (pf, amp)) in partials(f, timbre).into_iter().enumerate() {
            parts.push(Partial {
                note: i,
                k: k as u32 + 1,
                freq: pf,
                amp,
                harmonic: a.periodicity.harmonics[i] * (k as u64 + 1),
            });
        }
    }
    let mut hits = Vec::new();
    for (x, p) in parts.iter().enumerate() {
        for q in &parts[x + 1..] {
            if p.note != q.note && p.harmonic == q.harmonic {
                let (lower, upper) = if p.note < q.note { (*p, *q) } else { (*q, *p) };
                hits.push(Coincidence {
                    lower,
                    upper,
                    beat: (p.freq - q.freq).abs(),
                });
            }
        }
    }
    (parts, hits)
}

/// Sethares' dissonance curve: the excess roughness of a dyad over `low` as the upper tone
/// sweeps `0..=max_cents`, relative to C4–D♭4 like the matrix (not clamped).
pub fn dissonance_curve(
    low: f64,
    timbre: Timbre,
    a4: f64,
    max_cents: f64,
    steps: usize,
) -> Vec<(f64, f64)> {
    let c4 = harmony::Midi(60).freq(a4);
    let reference = roughness::excess_roughness(c4, c4 * 2f64.powf(1.0 / 12.0), timbre);
    (0..=steps)
        .map(|s| {
            let c = max_cents * s as f64 / steps as f64;
            let hi = low * 2f64.powf(c / 1200.0);
            (c, roughness::excess_roughness(low, hi, timbre) / reference)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn neighbours_of_c_major() {
        let n = neighbours(&[48, 52, 55], &Options::default());
        let find = |moved: usize, step: i8| {
            n.iter()
                .find(|x| x.moved == moved && x.step == step)
                .unwrap()
        };
        assert_eq!(find(1, -1).symbol, "Cm"); // E → E♭
        assert_eq!(find(2, 2).symbol, "Am/C"); // G → A
        assert_eq!(find(2, 1).symbol, "C+"); // G → G♯
        assert_eq!(find(0, -1).symbol, "Em/B"); // C → B
        assert!(n.iter().all(|x| x.notes.len() == 3));
        // Moves onto another note are skipped: none of these collide.
        assert_eq!(n.len(), 12);
        let close = neighbours(&[60, 61], &Options::default());
        assert!(!close.iter().any(|x| x.moved == 0 && x.step == 1));
    }

    #[test]
    fn just_partials_coincide_exactly() {
        let a = analyze(
            &[48, 52, 55],
            &Options {
                tuning: Tuning::Just,
                ..Options::default()
            },
        )
        .unwrap();
        let (parts, hits) = spectrum(&a, Tuning::Just, Timbre::Harmonic6);
        assert_eq!(parts.len(), 18);
        // 4:5:6 — C's 5th partial meets E's 4th (harmonic 20), C's 3rd meets G's 2nd (12)…
        assert!(hits
            .iter()
            .any(|h| h.harmonic() == 20 && h.lower.k == 5 && h.upper.k == 4));
        assert!(hits.iter().all(|h| h.beat < 1e-9));
        let (_, et) = spectrum(&a, Tuning::Et, Timbre::Harmonic6);
        let ce = et.iter().find(|h| h.harmonic() == 20).unwrap();
        assert!((ce.beat - 5.19).abs() < 0.02, "{}", ce.beat);
    }

    #[test]
    fn curve_has_minima_at_simple_ratios() {
        let f = 261.63;
        let c = dissonance_curve(f, Timbre::Harmonic6, 440.0, 1200.0, 1200);
        let at = |cents: usize| c[cents].1;
        // The fifth (702¢) is a local minimum against its surroundings; the semitone is rough.
        assert!(at(702) < at(650) && at(702) < at(750));
        assert!(at(100) > at(702));
        assert!(at(0) < 1e-9, "{}", at(0));
    }
}

impl Coincidence {
    pub fn harmonic(&self) -> u64 {
        self.lower.harmonic
    }
}
