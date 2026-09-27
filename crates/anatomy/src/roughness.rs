//! Sensory roughness after Sethares (1993), from Plomp & Levelt's dissonance curve.

use harmony::Midi;

/// Tone model for roughness and playback.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Timbre {
    Sine,
    /// Harmonics 1–6 with amplitudes 0.88^(k−1).
    #[default]
    Harmonic6,
}

/// Dissonance of two sine partials.
pub fn dissonance(f1: f64, f2: f64, a1: f64, a2: f64) -> f64 {
    let fm = f1.min(f2);
    let df = (f2 - f1).abs();
    let s = 0.24 / (0.0207 * fm + 18.96);
    a1.min(a2) * ((-3.51 * s * df).exp() - (-5.75 * s * df).exp())
}

pub fn partials(f: f64, timbre: Timbre) -> Vec<(f64, f64)> {
    match timbre {
        Timbre::Sine => vec![(f, 1.0)],
        Timbre::Harmonic6 => (1..=6)
            .map(|k| (f * k as f64, 0.88f64.powi(k - 1)))
            .collect(),
    }
}

/// Summed dissonance of every partial pair of two tones.
pub fn pair_roughness(f1: f64, f2: f64, timbre: Timbre) -> f64 {
    let p1 = partials(f1, timbre);
    let p2 = partials(f2, timbre);
    p1.iter()
        .flat_map(|&(a, x)| p2.iter().map(move |&(b, y)| dissonance(a, b, x, y)))
        .sum()
}

/// Roughness of the reference dyad C4–D♭4 at this A4 and timbre.
pub fn reference(a4: f64, timbre: Timbre) -> f64 {
    pair_roughness(Midi(60).freq(a4), Midi(61).freq(a4), timbre)
}

/// Pair roughness relative to C4–D♭4, clamped to 0..1.
pub fn normalized(f1: f64, f2: f64, timbre: Timbre, reference: f64) -> f64 {
    (pair_roughness(f1, f2, timbre) / reference).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn symmetric_and_bounded() {
        for t in [Timbre::Sine, Timbre::Harmonic6] {
            let r = reference(440.0, t);
            for (a, b) in [(200.0, 213.0), (130.8, 196.0), (440.0, 880.0)] {
                assert!((pair_roughness(a, b, t) - pair_roughness(b, a, t)).abs() < 1e-12);
                let n = normalized(a, b, t, r);
                assert!((0.0..=1.0).contains(&n));
            }
        }
    }

    #[test]
    fn unison_is_smooth_semitone_is_rough() {
        assert_eq!(dissonance(440.0, 440.0, 1.0, 1.0), 0.0);
        let t = Timbre::Harmonic6;
        let r = reference(440.0, t);
        let semitone = normalized(Midi(55).freq(440.0), Midi(56).freq(440.0), t, r);
        let fifth = normalized(Midi(55).freq(440.0), Midi(62).freq(440.0), t, r);
        assert_eq!(semitone, 1.0);
        assert!(fifth < 0.5);
    }
}
