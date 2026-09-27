//! Just ratios above the bass, harmonic numbers and the chord's common period.

use harmony::ratio::{common_harmonics, Ratio, RatioSet};
use harmony::Midi;

#[derive(Clone, PartialEq, Debug)]
pub struct Periodicity {
    /// Just ratio of each note above the bass (no octave folding).
    pub ratios: Vec<Ratio>,
    /// Smallest integers proportional to the ratios: the chord as harmonics 10 : 15 : 16 : 24.
    pub harmonics: Vec<u64>,
    /// Bass cycles per chord period (`harmonics[0]`).
    pub bass_cycles: u64,
    /// Equal-tempered bass frequency.
    pub bass_freq: f64,
    /// Common fundamental: `bass_freq / bass_cycles`.
    pub fundamental: f64,
    /// Chord period in seconds: `bass_cycles / bass_freq`.
    pub period: f64,
    pub just_freqs: Vec<f64>,
    pub et_freqs: Vec<f64>,
    /// 12-TET minus just, in cents, per note.
    pub cents: Vec<f64>,
}

impl Periodicity {
    /// `notes` must be sorted ascending and non-empty.
    pub fn new(notes: &[u8], set: RatioSet, a4: f64) -> Self {
        let bass = notes[0];
        let ratios: Vec<Ratio> = notes
            .iter()
            .map(|&m| set.ratio((m - bass) as u32))
            .collect();
        let harmonics = common_harmonics(&ratios).expect("harmonic numbers fit in u64");
        let bass_cycles = harmonics[0];
        let bass_freq = Midi(bass).freq(a4);
        let just_freqs: Vec<f64> = ratios
            .iter()
            .map(|r| bass_freq * r.num() as f64 / r.den() as f64)
            .collect();
        let et_freqs: Vec<f64> = notes.iter().map(|&m| Midi(m).freq(a4)).collect();
        let cents = et_freqs
            .iter()
            .zip(&just_freqs)
            .map(|(e, j)| 1200.0 * (e / j).log2())
            .collect();
        Self {
            ratios,
            harmonics,
            bass_cycles,
            bass_freq,
            fundamental: bass_freq / bass_cycles as f64,
            period: bass_cycles as f64 / bass_freq,
            just_freqs,
            et_freqs,
            cents,
        }
    }

    /// "10:15:16:24"
    pub fn harmonics_text(&self, sep: &str) -> String {
        self.harmonics
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(sep)
    }
}

/// Beating between the nearly coincident partials of two notes.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Beat {
    pub i: usize,
    pub j: usize,
    /// Partial numbers that (nearly) coincide: `lower`·fᵢ ≈ `upper`·fⱼ.
    pub lower_partial: u64,
    pub upper_partial: u64,
    /// |upper·fⱼ − lower·fᵢ| in Hz: 0 in just intonation.
    pub rate: f64,
}

/// For each pair whose just ratio p/q has both terms within `max_partial`, the beat rate of
/// the coinciding partials (p-th of the lower note, q-th of the upper) at frequencies `freqs`.
/// In just intonation every rate is 0; in 12-TET these beats are why the summed wave never
/// repeats exactly. With sine tones (`max_partial` = 1) only unisons could beat.
pub fn beats(notes: &[u8], set: RatioSet, freqs: &[f64], max_partial: u64) -> Vec<Beat> {
    let mut out = Vec::new();
    for i in 0..notes.len() {
        for j in i + 1..notes.len() {
            let r = set.ratio((notes[j] - notes[i]) as u32);
            if r.num() <= max_partial && r.den() <= max_partial {
                out.push(Beat {
                    i,
                    j,
                    lower_partial: r.num(),
                    upper_partial: r.den(),
                    rate: (r.den() as f64 * freqs[j] - r.num() as f64 * freqs[i]).abs(),
                });
            }
        }
    }
    out
}

/// Stolzenburg's (2015) smoothed logarithmic periodicity: take each chord tone in turn as the
/// reference, tune every note from it (descending intervals use [`RatioSet::signed_ratio`]),
/// measure the period in cycles of the lowest tone, and average log₂ of those periods.
/// Lower means more consonant: 0 for an octave, 2 for a major triad, 3.3 for a minor one.
pub fn smoothed_log_periodicity(notes: &[u8], set: RatioSet) -> f64 {
    let total: f64 = notes
        .iter()
        .map(|&reference| {
            let tuned: Vec<Ratio> = notes
                .iter()
                .map(|&m| set.signed_ratio(m as i32 - reference as i32))
                .collect();
            let low = tuned[0];
            let rel: Vec<Ratio> = tuned
                .iter()
                .map(|&r| r * Ratio::new(low.den(), low.num()))
                .collect();
            let h = common_harmonics(&rel).expect("harmonic numbers fit in u64");
            (h[0] as f64).log2()
        })
        .sum();
    total / notes.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_voicing() {
        let p = Periodicity::new(&[48, 55, 56, 63], RatioSet::FiveLimit, 440.0);
        assert_eq!(p.harmonics, [10, 15, 16, 24]);
        assert_eq!(p.bass_cycles, 10);
        assert!((p.period * 1000.0 - 76.45).abs() < 0.01);
        assert!((p.fundamental - 13.08).abs() < 0.01);
        assert!((p.cents[1] - (-1.955)).abs() < 1e-3);
    }

    #[test]
    fn beating() {
        let notes = [48, 52, 55];
        let p = Periodicity::new(&notes, RatioSet::FiveLimit, 440.0);
        assert!(beats(&notes, RatioSet::FiveLimit, &p.just_freqs, 6)
            .iter()
            .all(|b| b.rate < 1e-9));
        let et = beats(&notes, RatioSet::FiveLimit, &p.et_freqs, 6);
        // C3–E3 in 12-TET: 4·164.81 − 5·130.81 ≈ 5.2 Hz; C3–G3: 2·196.00 − 3·130.81 ≈ 0.44 Hz.
        let ce = et.iter().find(|b| (b.i, b.j) == (0, 1)).unwrap();
        assert_eq!((ce.lower_partial, ce.upper_partial), (5, 4));
        assert!((ce.rate - 5.19).abs() < 0.01, "{}", ce.rate);
        let cg = et.iter().find(|b| (b.i, b.j) == (0, 2)).unwrap();
        assert!((cg.rate - 0.445).abs() < 0.01, "{}", cg.rate);
        assert!(beats(&notes, RatioSet::FiveLimit, &p.et_freqs, 1).is_empty());
    }

    #[test]
    fn stolzenburg_example() {
        // First-inversion diminished triad {0,3,9}: periods 15, 25, 6 from each reference tone.
        let s = smoothed_log_periodicity(&[48, 51, 57], RatioSet::Stolzenburg);
        let expected = (15f64.log2() + 25f64.log2() + 6f64.log2()) / 3.0;
        assert!((s - expected).abs() < 1e-12, "{s} vs {expected}");
    }

    #[test]
    fn smoothing() {
        // Major triad: every reference tone yields 4:5:6 again, so log2 4 = 2.
        let s = smoothed_log_periodicity(&[48, 52, 55], RatioSet::FiveLimit);
        assert_eq!(s, 2.0);
        let minor = smoothed_log_periodicity(&[48, 51, 55], RatioSet::FiveLimit);
        assert!(minor > s);
        assert_eq!(
            smoothed_log_periodicity(&[48, 60], RatioSet::FiveLimit),
            0.0
        );
    }
}
