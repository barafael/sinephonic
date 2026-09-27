//! Pitch classes (ℤ/12) and pitch-class sets.

use core::fmt;
use core::ops::{Add, Neg, Sub};

use crate::forte_table::FORTE;

/// A pitch class, 0 = C … 11 = B.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Default)]
pub struct PitchClass(u8);

impl PitchClass {
    pub const C: Self = Self(0);

    /// Any integer, reduced mod 12.
    pub const fn new(v: i32) -> Self {
        Self(v.rem_euclid(12) as u8)
    }

    pub const fn of_midi(m: u8) -> Self {
        Self(m % 12)
    }

    pub const fn value(self) -> u8 {
        self.0
    }

    /// Ascending distance from `self` to `other`, in 0..12.
    pub const fn up_to(self, other: Self) -> u8 {
        (other.0 + 12 - self.0) % 12
    }

    pub const fn is_black_key(self) -> bool {
        matches!(self.0, 1 | 3 | 6 | 8 | 10)
    }
}

impl Add<i32> for PitchClass {
    type Output = Self;
    fn add(self, n: i32) -> Self {
        Self::new(self.0 as i32 + n)
    }
}

impl Sub<i32> for PitchClass {
    type Output = Self;
    fn sub(self, n: i32) -> Self {
        Self::new(self.0 as i32 - n)
    }
}

/// Inversion about C (I₀).
impl Neg for PitchClass {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-(self.0 as i32))
    }
}

impl From<u8> for PitchClass {
    fn from(v: u8) -> Self {
        Self(v % 12)
    }
}

const MASK: u16 = 0x0fff;

/// A set of pitch classes as a 12-bit mask (bit k = pitch class k).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
pub struct PcSet(u16);

impl PcSet {
    pub const EMPTY: Self = Self(0);
    pub const CHROMATIC: Self = Self(MASK);

    pub const fn from_bits(bits: u16) -> Self {
        Self(bits & MASK)
    }

    pub const fn bits(self) -> u16 {
        self.0
    }

    pub fn from_midi(notes: &[u8]) -> Self {
        notes.iter().map(|&m| PitchClass::of_midi(m)).collect()
    }

    pub const fn contains(self, pc: PitchClass) -> bool {
        self.0 & (1 << pc.0) != 0
    }

    /// `contains` for a raw interval class; convenient for root-relative sets.
    pub const fn has(self, ic: u8) -> bool {
        self.0 & (1 << (ic % 12)) != 0
    }

    pub fn insert(&mut self, pc: PitchClass) {
        self.0 |= 1 << pc.0;
    }

    pub const fn with(self, pc: PitchClass) -> Self {
        Self(self.0 | (1 << pc.0))
    }

    pub const fn len(self) -> usize {
        self.0.count_ones() as usize
    }

    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    pub fn iter(self) -> impl Iterator<Item = PitchClass> {
        (0..12u8)
            .filter(move |&k| self.0 & (1 << k) != 0)
            .map(PitchClass)
    }

    /// Tₙ: rotate every member up by `n` semitones.
    pub const fn transpose(self, n: i32) -> Self {
        let n = n.rem_euclid(12) as u32;
        if n == 0 {
            return self;
        }
        Self(((self.0 << n) | (self.0 >> (12 - n))) & MASK)
    }

    /// I₀: map every pc x to −x.
    pub const fn invert(self) -> Self {
        // Reversing the 12 bits maps k to 11 − k; one more step up gives −k.
        let rev = (self.0.reverse_bits() >> 4) & MASK;
        Self(rev).transpose(1)
    }

    /// The set seen from `root`: each member becomes its interval above the root.
    pub const fn relative_to(self, root: PitchClass) -> Self {
        self.transpose(-(root.0 as i32))
    }

    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    pub const fn intersection(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }

    pub const fn complement(self) -> Self {
        Self(!self.0 & MASK)
    }

    /// Prime form (Rahn's convention): the smallest bitmask over all 24 Tₙ and TₙI images.
    /// Comparing masks as integers compares the largest members first, which is Rahn's rule,
    /// and the minimum always contains 0.
    pub fn prime_form(self) -> Self {
        let inv = self.invert();
        (0..12)
            .flat_map(|n| [self.transpose(n), inv.transpose(n)])
            .min()
            .unwrap_or(self)
    }

    /// The Tₙ-class representative: the smallest rotation (inversion not allowed).
    pub fn transposition_class(self) -> Self {
        (0..12).map(|n| self.transpose(n)).min().unwrap_or(self)
    }

    /// Number of n in 0..12 with Tₙ(S) = S: 1 for most sets, 3 for aug, 4 for dim7,
    /// 6 for the whole-tone scale.
    pub fn transpositional_symmetry(self) -> usize {
        (0..12).filter(|&n| self.transpose(n) == self).count()
    }

    /// Number of n with TₙI(S) = S.
    pub fn inversional_symmetry(self) -> usize {
        let inv = self.invert();
        (0..12).filter(|&n| inv.transpose(n) == self).count()
    }

    /// Interval-class vector ⟨ic1 … ic6⟩.
    pub fn interval_vector(self) -> [u8; 6] {
        let pcs: Vec<u8> = self.iter().map(|p| p.0).collect();
        let mut v = [0u8; 6];
        for (i, &a) in pcs.iter().enumerate() {
            for &b in &pcs[i + 1..] {
                let d = b - a;
                let ic = d.min(12 - d);
                v[ic as usize - 1] += 1;
            }
        }
        v
    }

    /// Forte's catalogue entry for this set's class, if it has one (every non-empty set does).
    pub fn forte(self) -> Option<Forte> {
        if self.is_empty() {
            return None;
        }
        let prime = self.prime_form();
        let (card, ordinal, _, iv) = *FORTE
            .iter()
            .find(|(_, _, bits, _)| PcSet(*bits).prime_form() == prime)?;
        let z = FORTE
            .iter()
            .any(|&(c, o, _, v)| c == card && o != ordinal && v == iv);
        Some(Forte {
            cardinality: card,
            ordinal,
            z,
        })
    }
}

impl FromIterator<PitchClass> for PcSet {
    fn from_iter<I: IntoIterator<Item = PitchClass>>(iter: I) -> Self {
        let mut s = Self::EMPTY;
        for pc in iter {
            s.insert(pc);
        }
        s
    }
}

impl fmt::Debug for PcSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

/// Written as `(0,3,7)`, with decimal 10 and 11 instead of t and e.
impl fmt::Display for PcSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("(")?;
        for (i, pc) in self.iter().enumerate() {
            if i > 0 {
                f.write_str(",")?;
            }
            write!(f, "{}", pc.0)?;
        }
        f.write_str(")")
    }
}

/// A Forte set-class name such as `4-Z15`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Forte {
    pub cardinality: u8,
    pub ordinal: u8,
    /// Shares its interval vector with another class of the same size.
    pub z: bool,
}

impl fmt::Display for Forte {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let z = if self.z { "Z" } else { "" };
        write!(f, "{}-{}{}", self.cardinality, z, self.ordinal)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn set(pcs: &[u8]) -> PcSet {
        pcs.iter().map(|&p| PitchClass::from(p)).collect()
    }

    #[test]
    fn transpose_composes_and_inversion_is_involution() {
        for bits in 0..4096u16 {
            let s = PcSet::from_bits(bits);
            assert_eq!(s.invert().invert(), s);
            assert_eq!(s.transpose(12), s);
            assert_eq!(s.transpose(5).transpose(9), s.transpose(14));
            assert_eq!(s.len(), s.transpose(7).len());
            // TₙI = I T₋ₙ
            assert_eq!(s.invert().transpose(3), s.transpose(-3).invert());
        }
    }

    #[test]
    fn invert_maps_x_to_minus_x() {
        assert_eq!(set(&[0, 4, 7]).invert(), set(&[0, 8, 5]));
        assert_eq!(set(&[1]).invert(), set(&[11]));
    }

    #[test]
    fn prime_forms_are_invariant_and_there_are_224_classes() {
        let mut classes = HashSet::new();
        for bits in 0..4096u16 {
            let s = PcSet::from_bits(bits);
            let p = s.prime_form();
            for n in 0..12 {
                assert_eq!(s.transpose(n).prime_form(), p);
                assert_eq!(s.invert().transpose(n).prime_form(), p);
            }
            if !s.is_empty() {
                assert!(p.has(0));
            }
            classes.insert(p);
        }
        assert_eq!(classes.len(), 224);
    }

    #[test]
    fn known_prime_forms() {
        assert_eq!(set(&[0, 4, 7]).prime_form(), set(&[0, 3, 7])); // major = minor under TₙI
        assert_eq!(set(&[7, 11, 2, 5]).prime_form(), set(&[0, 2, 5, 8])); // dominant 7th
                                                                          // Rahn and Forte differ on 5-20; Rahn's is (0,1,5,6,8).
        assert_eq!(set(&[0, 1, 3, 7, 8]).prime_form(), set(&[0, 1, 5, 6, 8]));
    }

    #[test]
    fn interval_vectors() {
        assert_eq!(set(&[0, 4, 7]).interval_vector(), [0, 0, 1, 1, 1, 0]);
        assert_eq!(set(&[0, 3, 6, 9]).interval_vector(), [0, 0, 4, 0, 0, 2]);
        assert_eq!(PcSet::CHROMATIC.interval_vector(), [12, 12, 12, 12, 12, 6]);
    }

    #[test]
    fn forte_table_is_consistent() {
        // Every catalogue entry's interval vector matches our computation, and every
        // non-empty set resolves to exactly one entry.
        let mut seen = HashSet::new();
        for &(card, _, bits, iv) in FORTE.iter() {
            let s = PcSet::from_bits(bits);
            assert_eq!(s.len(), card as usize);
            assert_eq!(s.interval_vector(), iv, "{s}");
            assert!(seen.insert(s.prime_form()), "duplicate class {s}");
        }
        assert_eq!(seen.len(), 223);
    }

    #[test]
    fn forte_names() {
        let name = |pcs: &[u8]| set(pcs).forte().unwrap().to_string();
        assert_eq!(name(&[0, 4, 7]), "3-11");
        assert_eq!(name(&[0, 3, 6, 9]), "4-28");
        assert_eq!(name(&[0, 1, 4, 6]), "4-Z15");
        assert_eq!(name(&[0, 1, 3, 7]), "4-Z29");
        assert_eq!(name(&[0, 4, 7, 10]), "4-27");
        assert_eq!(name(&[0, 2, 4, 6, 8, 10]), "6-35");
        // Scriabin's mystic chord: C F♯ B♭ E A D.
        assert_eq!(name(&[0, 6, 10, 4, 9, 2]), "6-34");
        assert_eq!(name(&[0, 2, 4, 5, 7, 9, 11]), "7-35");
        assert!(PcSet::EMPTY.forte().is_none());
    }

    #[test]
    fn symmetry() {
        assert_eq!(set(&[0, 3, 6, 9]).transpositional_symmetry(), 4);
        assert_eq!(set(&[0, 4, 8]).transpositional_symmetry(), 3);
        assert_eq!(set(&[0, 4, 7]).transpositional_symmetry(), 1);
        assert_eq!(set(&[0, 4, 7]).inversional_symmetry(), 0);
        assert_eq!(set(&[0, 2, 7]).inversional_symmetry(), 1);
    }

    #[test]
    fn relative_to_root() {
        let cm_b6 = set(&[0, 7, 8, 3]);
        assert_eq!(cm_b6.relative_to(PitchClass::new(8)), set(&[4, 11, 0, 7]));
        assert_eq!(PitchClass::new(10).up_to(PitchClass::new(2)), 4);
    }
}
