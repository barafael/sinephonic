//! Psychoacoustic and theoretical measures from the literature.

use harmony::{PcSet, PitchClass};

/// Parncutt's root-support weights (Parncutt 2006 revision of the 1988 model), by interval
/// class above a candidate root: P1 10, M2 1, M3 3, P5 5, m7 2. They are Terhardt's
/// subharmonics (n = 1, 9, 5, 3, 7) folded into one octave.
pub const PARNCUTT_WEIGHTS: [f64; 12] =
    [10.0, 0.0, 1.0, 0.0, 3.0, 0.0, 0.0, 5.0, 0.0, 0.0, 2.0, 0.0];

/// Root support w(r) for every candidate root 0..12.
pub fn parncutt_support(pcs: PcSet) -> [f64; 12] {
    std::array::from_fn(|r| {
        pcs.relative_to(PitchClass::new(r as i32))
            .iter()
            .map(|ic| PARNCUTT_WEIGHTS[ic.value() as usize])
            .sum()
    })
}

/// Root ambiguity √(Σw / max w): 1 when one root takes all the support, higher when several
/// roots compete.
pub fn parncutt_ambiguity(support: &[f64; 12]) -> f64 {
    let max = support.iter().copied().fold(0.0, f64::max);
    if max == 0.0 {
        return 1.0;
    }
    (support.iter().sum::<f64>() / max).sqrt()
}

/// Hindemith's interval ranking (Craft of Musical Composition, Series 2) by semitones mod 12,
/// best first, with which note of the interval is its root. The tritone has no root.
const HINDEMITH: [(u8, bool); 10] = [
    (7, false),  // P5: lower note
    (5, true),   // P4: upper
    (4, false),  // M3: lower
    (8, true),   // m6: upper
    (3, false),  // m3: lower
    (9, true),   // M6: upper
    (2, true),   // M2: upper
    (10, false), // m7: lower
    (1, true),   // m2: upper
    (11, false), // M7: lower
];

/// The root of the voicing's best interval: rank the intervals, and among equals take the
/// lowest occurrence in the voicing. `notes` sorted ascending.
pub fn hindemith_root(notes: &[u8]) -> Option<PitchClass> {
    for &(ic, upper) in &HINDEMITH {
        let mut found: Option<(u8, u8)> = None;
        for (i, &a) in notes.iter().enumerate() {
            for &b in &notes[i + 1..] {
                if (b - a) % 12 == ic {
                    // Lowest occurrence: lowest lower note, then the closest upper note.
                    if found.is_none_or(|(fa, fb)| (a, b) < (fa, fb)) {
                        found = Some((a, b));
                    }
                }
            }
        }
        if let Some((a, b)) = found {
            return Some(PitchClass::of_midi(if upper { b } else { a }));
        }
    }
    None
}

/// Huron (1994) interval-class consonance values for ic1..ic6.
pub const HURON: [f64; 6] = [-1.428, -0.582, 0.594, 0.386, 1.240, -0.453];

/// Aggregate dyadic consonance: interval vector · Huron's values.
pub fn huron_consonance(iv: [u8; 6]) -> f64 {
    iv.iter().zip(HURON).map(|(&n, w)| n as f64 * w).sum()
}

/// Mean harmonic dissonance of the voicing's pairs on Huron's scale, mapped to 0..1:
/// 0 when every pair is as consonant as a perfect 5th/4th (unisons and octaves count as
/// such), 1 when every pair is a semitone class.
pub fn huron_dissonance(pairs: &[crate::Pair]) -> f64 {
    let best = HURON[4];
    let worst = HURON[0];
    let mean = pairs
        .iter()
        .map(|p| {
            let d = p.semitones % 12;
            match d.min(12 - d) {
                0 => best,
                ic => HURON[ic as usize - 1],
            }
        })
        .sum::<f64>()
        / pairs.len().max(1) as f64;
    (best - mean) / (best - worst)
}

/// Cook & Fujisawa (2006) triad tension and modality, summed over every triple of partials
/// taken from three different notes (6 partials each, amplitudes 1, .88, .76, .64, .58, .52)
/// and divided by the number of note triples. Tension peaks when the two stacked intervals are
/// equal (augmented, diminished, quartal); modality is +1 for a root-position major triad's
/// fundamentals and −1 for a minor one.
pub fn cook_fujisawa(freqs: &[f64]) -> (f64, f64) {
    const AMP: [f64; 6] = [1.0, 0.88, 0.76, 0.64, 0.58, 0.52];
    const ALPHA: f64 = 0.60;
    const EPS: f64 = 1.558;
    let n = freqs.len();
    if n < 3 {
        return (0.0, 0.0);
    }
    let parts: Vec<Vec<(f64, f64)>> = freqs
        .iter()
        .map(|&f| {
            AMP.iter()
                .enumerate()
                .map(|(k, &a)| (12.0 * (f * (k + 1) as f64).log2(), a))
                .collect()
        })
        .collect();
    let (mut t, mut m, mut triples) = (0.0, 0.0, 0);
    for i in 0..n {
        for j in i + 1..n {
            for k in j + 1..n {
                triples += 1;
                for &(pi, ai) in &parts[i] {
                    for &(pj, aj) in &parts[j] {
                        for &(pk, ak) in &parts[k] {
                            let mut p = [pi, pj, pk];
                            p.sort_by(f64::total_cmp);
                            let x = p[1] - p[0];
                            let y = p[2] - p[1];
                            let v = ai * aj * ak;
                            let d = y - x;
                            t += v * (-(d / ALPHA).powi(2)).exp();
                            m += v * (-2.0 * d / EPS) * (-(d.powi(4)) / 4.0).exp();
                        }
                    }
                }
            }
        }
    }
    (t / triples as f64, m / triples as f64)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(p: &[u8]) -> PcSet {
        p.iter().map(|&x| PitchClass::from(x)).collect()
    }

    #[test]
    fn parncutt_roots() {
        let s = parncutt_support(set(&[0, 4, 7]));
        assert_eq!(s[0], 18.0); // C: 10 + 3 + 5
        let best = (0..12).max_by(|&a, &b| s[a].total_cmp(&s[b])).unwrap();
        assert_eq!(best, 0);
        // Minor triad: the root still wins, but less clearly.
        let m = parncutt_support(set(&[0, 3, 7]));
        assert!(parncutt_ambiguity(&m) > parncutt_ambiguity(&s));
        // dim7: four equal candidates.
        let d = parncutt_support(set(&[0, 3, 6, 9]));
        assert_eq!(d[0], d[3]);
    }

    #[test]
    fn hindemith() {
        let r = |n: &[u8]| hindemith_root(n).map(|p| p.value());
        assert_eq!(r(&[48, 52, 55]), Some(0)); // C major: C–G fifth
        assert_eq!(r(&[52, 55, 60]), Some(0)); // first inversion: G–C fourth, root upper
        assert_eq!(r(&[55, 60, 64]), Some(0)); // second inversion: G–C fourth
        assert_eq!(r(&[48, 55, 56, 63]), Some(0)); // C–G is the lowest fifth
        assert_eq!(r(&[48, 54]), None); // tritone alone
        assert_eq!(r(&[48, 51]), Some(0));
    }

    #[test]
    fn huron_values() {
        let maj = huron_consonance(set(&[0, 4, 7]).interval_vector());
        let cluster = huron_consonance(set(&[0, 1, 2]).interval_vector());
        assert!(maj > 2.0 && cluster < -3.0);
    }

    #[test]
    fn cook_modality_sign() {
        let f = |m: &[f64]| {
            m.iter()
                .map(|&x| 440.0 * 2f64.powf((x - 69.0) / 12.0))
                .collect::<Vec<_>>()
        };
        let (t_maj, m_maj) = cook_fujisawa(&f(&[60.0, 64.0, 67.0]));
        let (_, m_min) = cook_fujisawa(&f(&[60.0, 63.0, 67.0]));
        let (t_aug, _) = cook_fujisawa(&f(&[60.0, 64.0, 68.0]));
        assert!(m_maj > 0.0 && m_min < 0.0, "{m_maj} {m_min}");
        assert!(t_aug > t_maj);
    }
}
