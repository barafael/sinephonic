//! Information-theoretic measures of a voicing, all in bits.
//!
//! * **Harmonic entropy** (Erlich): the listener's uncertainty about which simple ratio an
//!   interval stands for. Each fraction n/d with n·d ≤ 10 000 is a candidate, weighted
//!   1/√(n·d) (simpler ratios have wider basins) and by a Gaussian of the mistuning with
//!   s = 17¢ (1 %, the pitch-discrimination width). Low at 1/1, 3/2, 5/4; high between them.
//! * **Description length**: Σ log₂ hᵢ, the bits needed to write the chord as harmonics
//!   h₀ : h₁ : … (Tenney height of the chord).
//! * **Period information**: log₂ of the common period in bass cycles, and Stolzenburg's
//!   smoothed version.
//! * **Root entropy**: Shannon entropy of the 12 root readings' probabilities; 0 when one root
//!   is certain, log₂ 12 ≈ 3.58 when all are equal.
//! * **Interval entropy**: Shannon entropy of the interval-class vector: how varied the
//!   intervals are (all-interval tetrachords are maximal, stacked-thirds sets minimal).
//! * **Spectral entropy**: Shannon entropy of the combined partial spectrum after merging
//!   partials within 20¢ (they fuse): coinciding partials make the spectrum simpler.

use std::sync::OnceLock;

use harmony::ratio::gcd;

use crate::explore::spectrum;
use crate::{Analysis, Timbre, Tuning};

/// Perceptual width of the Gaussian, in cents.
pub const HE_SPREAD: f64 = 17.0;
/// Tenney-height limit of the candidate ratios.
pub const HE_LIMIT: u64 = 10_000;
/// Widest interval in the basis (3 octaves); wider intervals are folded down by octaves.
const HE_MAX_CENTS: f64 = 3600.0;

struct Basis {
    /// (cents, weight) sorted by cents.
    ratios: Vec<(f64, f64)>,
    /// Harmonic entropy over one octave, for normalisation.
    min: f64,
    max: f64,
}

fn basis() -> &'static Basis {
    static B: OnceLock<Basis> = OnceLock::new();
    B.get_or_init(|| {
        let mut ratios = Vec::new();
        for d in 1..=(HE_LIMIT as f64).sqrt() as u64 {
            for n in d..=HE_LIMIT / d {
                if gcd(n, d) != 1 {
                    continue;
                }
                let c = 1200.0 * (n as f64 / d as f64).log2();
                if c > HE_MAX_CENTS + 5.0 * HE_SPREAD {
                    break;
                }
                ratios.push((c, 1.0 / ((n * d) as f64).sqrt()));
            }
        }
        ratios.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut b = Basis {
            ratios,
            min: 0.0,
            max: 1.0,
        };
        let curve: Vec<f64> = (0..=1200).map(|c| entropy_with(&b, c as f64)).collect();
        b.min = curve.iter().copied().fold(f64::INFINITY, f64::min);
        b.max = curve.iter().copied().fold(0.0, f64::max);
        b
    })
}

fn entropy_with(b: &Basis, cents: f64) -> f64 {
    let lo = b.ratios.partition_point(|r| r.0 < cents - 5.0 * HE_SPREAD);
    let hi = b.ratios.partition_point(|r| r.0 <= cents + 5.0 * HE_SPREAD);
    let ps: Vec<f64> = b.ratios[lo..hi]
        .iter()
        .map(|&(c, w)| w * (-(cents - c).powi(2) / (2.0 * HE_SPREAD * HE_SPREAD)).exp())
        .collect();
    let z: f64 = ps.iter().sum();
    if z <= 0.0 {
        return 0.0;
    }
    -ps.iter()
        .filter(|&&p| p > 0.0)
        .map(|&p| (p / z) * (p / z).log2())
        .sum::<f64>()
}

/// Harmonic entropy of an interval of `cents`, in bits.
pub fn harmonic_entropy(cents: f64) -> f64 {
    let mut c = cents.abs();
    while c > HE_MAX_CENTS {
        c -= 1200.0;
    }
    entropy_with(basis(), c)
}

/// Harmonic entropy scaled to 0..1 by its minimum and maximum over one octave.
pub fn harmonic_entropy_norm(cents: f64) -> f64 {
    let b = basis();
    ((harmonic_entropy(cents) - b.min) / (b.max - b.min)).clamp(0.0, 1.0)
}

/// Harmonic entropy curve over `0..=max_cents`, scaled to 0..1 by the one-octave range.
pub fn harmonic_entropy_curve(max_cents: f64, steps: usize) -> Vec<(f64, f64)> {
    (0..=steps)
        .map(|s| {
            let c = max_cents * s as f64 / steps as f64;
            (c, harmonic_entropy_norm(c))
        })
        .collect()
}

/// Shannon entropy in bits of a distribution given by non-negative weights.
pub fn shannon(weights: impl IntoIterator<Item = f64>) -> f64 {
    let w: Vec<f64> = weights.into_iter().filter(|&x| x > 0.0).collect();
    let z: f64 = w.iter().sum();
    if z <= 0.0 {
        return 0.0;
    }
    (-w.iter().map(|&x| (x / z) * (x / z).log2()).sum::<f64>()).max(0.0)
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Information {
    /// Mean and maximum harmonic entropy over the voicing's pairs (bits).
    pub harmonic_entropy: f64,
    pub harmonic_entropy_max: f64,
    /// Mean harmonic entropy scaled to 0..1 over one octave's range.
    pub harmonic_entropy_norm: f64,
    /// Σ log₂ hᵢ.
    pub description_bits: f64,
    /// log₂ of the period in bass cycles.
    pub period_bits: f64,
    /// Stolzenburg's smoothed log₂ periodicity.
    pub smoothed_period_bits: f64,
    /// Entropy of the root readings (bits) and as a fraction of log₂ 12.
    pub root_entropy: f64,
    pub root_entropy_norm: f64,
    /// Entropy of the interval-class vector (bits) and as a fraction of log₂ 6.
    pub interval_entropy: f64,
    pub interval_entropy_norm: f64,
    /// Entropy of the fused partial spectrum (bits), and its maximum log₂(partials).
    pub spectral_entropy: f64,
    pub spectral_entropy_max: f64,
    /// Composite complexity, 0..1 (see [`complexity`]).
    pub complexity: f64,
}

/// Composite complexity: harmonic uncertainty (0.3), period information (0.25), root
/// uncertainty (0.2), intervallic variety (0.15) and size (0.1), each scaled to 0..1.
pub fn complexity(i: &Information, pitch_classes: usize) -> f64 {
    let size = ((pitch_classes as f64 - 3.0) / 4.0).clamp(0.0, 1.0);
    (0.3 * i.harmonic_entropy_norm
        + 0.25 * (i.smoothed_period_bits / 7.0).clamp(0.0, 1.0)
        + 0.2 * i.root_entropy_norm
        + 0.15 * i.interval_entropy_norm
        + 0.1 * size)
        .clamp(0.0, 1.0)
}

/// All measures for an analysis. `smoothed_period_bits` comes from the periodicity module so
/// it isn't computed twice.
pub fn measure(
    a: &Analysis,
    tuning: Tuning,
    timbre: Timbre,
    smoothed_period_bits: f64,
) -> Information {
    let freqs = a.freqs(tuning);
    let pair_he: Vec<f64> = a
        .pairs
        .iter()
        .map(|p| harmonic_entropy(1200.0 * (freqs[p.j] / freqs[p.i]).log2()))
        .collect();
    let mean_he = pair_he.iter().sum::<f64>() / pair_he.len().max(1) as f64;
    let b = basis();
    let harmonic_entropy_norm = ((mean_he - b.min) / (b.max - b.min)).clamp(0.0, 1.0);

    let root_entropy = shannon(a.readings.iter().map(|r| r.prob));
    let iv = harmony::PcSet::from_midi(&a.notes).interval_vector();
    let interval_entropy = shannon(iv.iter().map(|&n| n as f64));

    // Merge partials within 20¢: they fuse into one component.
    let (mut parts, _) = spectrum(a, tuning, timbre);
    parts.sort_by(|x, y| x.freq.total_cmp(&y.freq));
    let mut merged: Vec<(f64, f64)> = Vec::new();
    for p in &parts {
        match merged.last_mut() {
            Some((f, amp)) if 1200.0 * (p.freq / *f).log2() < 20.0 => *amp += p.amp,
            _ => merged.push((p.freq, p.amp)),
        }
    }
    let spectral_entropy = shannon(merged.iter().map(|m| m.1));

    let mut info = Information {
        harmonic_entropy: mean_he,
        harmonic_entropy_max: pair_he.iter().copied().fold(0.0, f64::max),
        harmonic_entropy_norm,
        description_bits: a
            .periodicity
            .harmonics
            .iter()
            .map(|&h| (h as f64).log2())
            .sum(),
        period_bits: (a.periodicity.bass_cycles as f64).log2(),
        smoothed_period_bits,
        root_entropy,
        root_entropy_norm: root_entropy / 12f64.log2(),
        interval_entropy,
        interval_entropy_norm: interval_entropy / 6f64.log2(),
        spectral_entropy,
        spectral_entropy_max: (parts.len() as f64).log2(),
        complexity: 0.0,
    };
    info.complexity = complexity(&info, a.pcs.len());
    info
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn harmonic_entropy_has_minima_at_simple_ratios() {
        let he = harmonic_entropy;
        assert!(he(0.0) < he(50.0));
        assert!(he(1200.0) < he(1150.0));
        assert!(he(702.0) < he(650.0) && he(702.0) < he(750.0));
        assert!(he(386.0) < he(440.0));
        // The fifth is clearer than the major third, which is clearer than the tritone.
        assert!(he(700.0) < he(400.0) && he(400.0) < he(600.0));
        // 12-TET fifths are 2¢ off 3/2: nearly as clear.
        assert!((he(700.0) - he(701.955)).abs() < 0.1);
        // Octave folding for compound intervals beyond the basis.
        assert!((he(4900.0) - he(3700.0)).abs() < 1e-12);
        assert!((0.0..=1.0).contains(&harmonic_entropy_norm(600.0)));
    }

    #[test]
    fn shannon_basics() {
        assert_eq!(shannon([1.0, 1.0]), 1.0);
        assert_eq!(shannon([5.0]), 0.0);
        assert!((shannon([1.0; 12]) - 12f64.log2()).abs() < 1e-12);
        assert_eq!(shannon([0.0, 0.0]), 0.0);
    }

    #[test]
    fn measures_order_sensibly() {
        use crate::{analyze, Options};
        let info = |n: &[u8]| {
            analyze(n, &Options::default())
                .unwrap()
                .information
                .unwrap()
        };
        let fifth = info(&[48, 55]);
        let major = info(&[48, 52, 55]);
        let maj7 = info(&[48, 52, 55, 59]);
        let alt = info(&[48, 52, 56, 58, 63]);
        let dim7 = info(&[48, 51, 54, 57]);
        assert!(fifth.complexity < major.complexity && major.complexity < maj7.complexity);
        assert!(
            maj7.complexity < alt.complexity,
            "{} {}",
            maj7.complexity,
            alt.complexity
        );
        assert!(major.complexity < dim7.complexity);
        assert!(major.description_bits < maj7.description_bits);
        // dim7 has maximal root uncertainty among these; the major triad the least.
        assert!(dim7.root_entropy > maj7.root_entropy && major.root_entropy < maj7.root_entropy);
        // Stacked thirds reuse few interval classes; an all-interval tetrachord uses all six.
        let all_interval = info(&[48, 49, 52, 54]);
        assert!(all_interval.interval_entropy > dim7.interval_entropy);
        assert!((all_interval.interval_entropy - 6f64.log2()).abs() < 1e-9);
        // Just intonation fuses coinciding partials: lower spectral entropy than none merging.
        assert!(major.spectral_entropy < major.spectral_entropy_max);
    }
}
