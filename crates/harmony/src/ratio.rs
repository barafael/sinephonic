//! Exact frequency ratios for just intonation.

use core::fmt;
use core::ops::Mul;

pub const fn gcd(mut a: u64, mut b: u64) -> u64 {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

pub const fn lcm(a: u64, b: u64) -> u64 {
    if a == 0 || b == 0 {
        0
    } else {
        a / gcd(a, b) * b
    }
}

/// A positive rational number, always in lowest terms.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Ratio {
    num: u64,
    den: u64,
}

impl Ratio {
    pub const UNISON: Self = Self { num: 1, den: 1 };

    /// Panics if `den` is 0.
    pub const fn new(num: u64, den: u64) -> Self {
        assert!(den != 0, "zero denominator");
        let g = gcd(num, den);
        Self {
            num: num / g,
            den: den / g,
        }
    }

    pub const fn num(self) -> u64 {
        self.num
    }

    pub const fn den(self) -> u64 {
        self.den
    }

    pub fn to_f64(self) -> f64 {
        self.num as f64 / self.den as f64
    }

    pub fn cents(self) -> f64 {
        1200.0 * self.to_f64().log2()
    }

    /// Multiplied by 2ᵏ.
    pub const fn octaves_up(self, k: u32) -> Self {
        Self::new(self.num << k, self.den)
    }

    /// Tenney height log₂(n·d): the harmonic distance of the interval.
    pub fn tenney_height(self) -> f64 {
        ((self.num as f64) * (self.den as f64)).log2()
    }

    /// Euler's gradus suavitatis of the interval, Γ(n·d).
    pub fn gradus(self) -> u32 {
        gradus(self.num * self.den)
    }

    /// Largest prime factor of n·d (the interval's prime limit); 1 for the unison.
    pub fn prime_limit(self) -> u64 {
        factorize(self.num * self.den).last().map_or(1, |&(p, _)| p)
    }
}

impl Mul for Ratio {
    type Output = Self;
    fn mul(self, o: Self) -> Self {
        // Cross-reduce first to keep the numbers small.
        let g1 = gcd(self.num, o.den);
        let g2 = gcd(o.num, self.den);
        Self::new(
            (self.num / g1) * (o.num / g2),
            (self.den / g2) * (o.den / g1),
        )
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.num, self.den)
    }
}

/// Prime factorisation by trial division, as (prime, exponent) pairs in ascending order.
pub fn factorize(mut n: u64) -> Vec<(u64, u32)> {
    let mut out = Vec::new();
    let mut p = 2;
    while p * p <= n {
        let mut e = 0;
        while n.is_multiple_of(p) {
            n /= p;
            e += 1;
        }
        if e > 0 {
            out.push((p, e));
        }
        p += if p == 2 { 1 } else { 2 };
    }
    if n > 1 {
        out.push((n, 1));
    }
    out
}

/// Euler's gradus suavitatis Γ(N) = 1 + Σ eₚ·(p − 1) over the prime factorisation of N.
pub fn gradus(n: u64) -> u32 {
    1 + factorize(n)
        .iter()
        .map(|&(p, e)| e * (p as u32 - 1))
        .sum::<u32>()
}

/// Which just ratio stands for each of the 12 interval classes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum RatioSet {
    /// 1/1 16/15 9/8 6/5 5/4 4/3 45/32 3/2 8/5 5/3 9/5 15/8 (the prototype's default).
    #[default]
    FiveLimit,
    /// The five-limit set with a 7/5 tritone and a 7/4 minor seventh.
    SevenLimit,
    /// Stolzenburg (2015) rational tuning #2: the smallest-denominator fraction within 1.1 % of
    /// each equal-tempered step. It matches the five-limit set except for a 7/5 tritone.
    Stolzenburg,
}

const R5: [(u64, u64); 12] = [
    (1, 1),
    (16, 15),
    (9, 8),
    (6, 5),
    (5, 4),
    (4, 3),
    (45, 32),
    (3, 2),
    (8, 5),
    (5, 3),
    (9, 5),
    (15, 8),
];

impl RatioSet {
    pub const ALL: [Self; 3] = [Self::FiveLimit, Self::SevenLimit, Self::Stolzenburg];

    pub fn label(self) -> &'static str {
        match self {
            Self::FiveLimit => "5-limit",
            Self::SevenLimit => "7-limit",
            Self::Stolzenburg => "Stolzenburg",
        }
    }

    /// The ratio for interval class `ic` (0..12).
    pub fn class_ratio(self, ic: u8) -> Ratio {
        let (p, q) = match (self, ic % 12) {
            (Self::SevenLimit | Self::Stolzenburg, 6) => (7, 5),
            (Self::SevenLimit, 10) => (7, 4),
            (_, i) => R5[i as usize],
        };
        Ratio::new(p, q)
    }

    /// The ratio for an ascending interval of `semitones`, without octave folding:
    /// `class_ratio(d mod 12) · 2^(d div 12)`, so a tenth is 5/2.
    pub fn ratio(self, semitones: u32) -> Ratio {
        self.class_ratio((semitones % 12) as u8)
            .octaves_up(semitones / 12)
    }

    /// Like [`ratio`](Self::ratio) but for descending intervals too, following Stolzenburg:
    /// `class_ratio(d mod 12) · 2^⌊d/12⌋`, so a minor third down is 6/5 · ½ = 3/5.
    pub fn signed_ratio(self, semitones: i32) -> Ratio {
        if semitones >= 0 {
            return self.ratio(semitones as u32);
        }
        let r = self.class_ratio(semitones.rem_euclid(12) as u8);
        let down = (-semitones.div_euclid(12)) as u32;
        Ratio::new(r.num(), r.den() << down)
    }
}

/// Puts ratios over a common fundamental: the smallest integers hᵢ with hᵢ/hⱼ = rᵢ/rⱼ.
/// For 1, 3/2, 8/5, 12/5 this gives 10, 15, 16, 24. Returns `None` on overflow.
pub fn common_harmonics(ratios: &[Ratio]) -> Option<Vec<u64>> {
    let mut l: u64 = 1;
    for r in ratios {
        l = (l / gcd(l, r.den)).checked_mul(r.den)?;
    }
    let a: Vec<u64> = ratios
        .iter()
        .map(|r| r.num.checked_mul(l / r.den))
        .collect::<Option<_>>()?;
    let g = a.iter().copied().fold(0, gcd);
    if g == 0 {
        return Some(a);
    }
    Some(a.into_iter().map(|x| x / g).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduction_and_arithmetic() {
        assert_eq!(Ratio::new(12, 8), Ratio::new(3, 2));
        assert_eq!(Ratio::new(3, 2) * Ratio::new(4, 3), Ratio::new(2, 1));
        assert_eq!(Ratio::new(45, 32).octaves_up(1), Ratio::new(45, 16));
        assert!((Ratio::new(3, 2).cents() - 701.955).abs() < 1e-3);
        for a in 1..40u64 {
            for b in 1..40u64 {
                let r = Ratio::new(a, b);
                assert_eq!(gcd(r.num(), r.den()), 1);
            }
        }
    }

    #[test]
    fn tables() {
        let s = RatioSet::FiveLimit;
        assert_eq!(s.ratio(0), Ratio::UNISON);
        assert_eq!(s.ratio(16), Ratio::new(5, 2));
        assert_eq!(s.ratio(15), Ratio::new(12, 5));
        assert_eq!(s.ratio(24), Ratio::new(4, 1));
        assert_eq!(RatioSet::SevenLimit.ratio(10), Ratio::new(7, 4));
        assert_eq!(RatioSet::SevenLimit.ratio(6), Ratio::new(7, 5));
        assert_eq!(RatioSet::Stolzenburg.ratio(10), Ratio::new(9, 5));
        assert_eq!(RatioSet::Stolzenburg.ratio(18), Ratio::new(14, 5));
        assert_eq!(s.signed_ratio(-3), Ratio::new(5, 6));
        assert_eq!(s.signed_ratio(-9), Ratio::new(3, 5));
        assert_eq!(s.signed_ratio(-12), Ratio::new(1, 2));
        assert_eq!(s.signed_ratio(-15), Ratio::new(5, 12));
        assert_eq!(s.signed_ratio(7), Ratio::new(3, 2));
        // Every table entry is within 1.2 % (≈ 21¢) of equal temperament.
        for set in RatioSet::ALL {
            for ic in 0..12u8 {
                let dev = set.class_ratio(ic).cents() - 100.0 * ic as f64;
                assert!(dev.abs() < 32.0, "{set:?} {ic}: {dev}");
            }
        }
    }

    #[test]
    fn harmonics() {
        let s = RatioSet::FiveLimit;
        let h = |d: &[u32]| {
            common_harmonics(&d.iter().map(|&x| s.ratio(x)).collect::<Vec<_>>()).unwrap()
        };
        assert_eq!(h(&[0, 7, 8, 15]), [10, 15, 16, 24]);
        assert_eq!(h(&[0, 4, 7]), [4, 5, 6]);
        assert_eq!(h(&[0, 4, 7, 10, 15]), [20, 25, 30, 36, 48]);
        assert_eq!(h(&[0, 4, 8, 10, 15]), [20, 25, 32, 36, 48]);
        assert_eq!(h(&[0, 12]), [1, 2]);
    }

    #[test]
    fn heights() {
        assert_eq!(Ratio::new(3, 2).tenney_height(), 6f64.log2());
        assert_eq!(Ratio::new(9, 8).gradus(), 8);
        assert_eq!(Ratio::new(1, 1).gradus(), 1);
        assert_eq!(gradus(72), 8);
        assert_eq!(Ratio::new(7, 4).prime_limit(), 7);
        assert_eq!(Ratio::new(45, 32).prime_limit(), 5);
        assert_eq!(factorize(360), [(2, 3), (3, 2), (5, 1)]);
    }
}
