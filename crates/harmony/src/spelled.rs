//! Spelled pitch classes and intervals on the line of fifths.
//!
//! A spelled pitch class is a position on the line of fifths (…F=−1, C=0, G=1, D=2…), so
//! C♯ = 7 and D♭ = −5 are different even though both are pitch class 1. A spelled interval is a
//! pair (fifths, octaves), meaning `fifths` perfect fifths plus `octaves` octaves. That makes
//! intervals an abelian group ℤ², adding them is vector addition, and:
//!
//! * semitones = 7·fifths + 12·octaves
//! * diatonic steps = 4·fifths + 7·octaves
//!
//! This is the representation of Temperley's line of fifths and of DCMLab's `pitchtypes`.

use core::fmt;
use core::ops::{Add, Neg, Sub};

use crate::pc::PitchClass;

/// Letters in line-of-fifths order, starting at F = −1.
const LOF_LETTERS: [char; 7] = ['F', 'C', 'G', 'D', 'A', 'E', 'B'];

/// A spelled pitch class such as E♭ or B𝄫.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Debug, Default)]
pub struct SpelledPc {
    fifths: i32,
}

impl SpelledPc {
    pub const fn from_fifths(fifths: i32) -> Self {
        Self { fifths }
    }

    /// `letter` in A–G (either case), `accidentals` > 0 for sharps, < 0 for flats.
    pub fn new(letter: char, accidentals: i32) -> Option<Self> {
        let idx = LOF_LETTERS
            .iter()
            .position(|&l| l == letter.to_ascii_uppercase())?;
        Some(Self {
            fifths: idx as i32 - 1 + 7 * accidentals,
        })
    }

    pub const fn fifths(self) -> i32 {
        self.fifths
    }

    pub const fn pc(self) -> PitchClass {
        PitchClass::new(7 * self.fifths)
    }

    pub const fn letter(self) -> char {
        LOF_LETTERS[(self.fifths + 1).rem_euclid(7) as usize]
    }

    /// Sharps (+) or flats (−).
    pub const fn accidentals(self) -> i32 {
        (self.fifths + 1).div_euclid(7)
    }

    /// Pitch class of the natural letter.
    pub const fn natural_pc(self) -> PitchClass {
        Self {
            fifths: (self.fifths + 1).rem_euclid(7) - 1,
        }
        .pc()
    }

    /// The octave number that makes this spelling sound as MIDI note `m` (B♯3 = 60 = C4).
    /// `m` must have this pitch class.
    pub fn octave_for_midi(self, m: u8) -> i32 {
        let natural = m as i32 - self.accidentals();
        natural.div_euclid(12) - 1
    }

    /// The spellings of `pc` that use at most `max_accidentals` sharps or flats, in
    /// line-of-fifths order (flattest first).
    pub fn spellings(pc: PitchClass, max_accidentals: i32) -> impl Iterator<Item = Self> {
        // 7 is its own inverse mod 12, so fifths ≡ 7·pc (mod 12).
        let base = (7 * pc.value() as i32).rem_euclid(12);
        (-3..=2)
            .map(move |k| Self {
                fifths: base + 12 * k,
            })
            .filter(move |s| s.accidentals().abs() <= max_accidentals)
    }

    /// Ascending interval from `self` up to `other`, within an octave (P1 ≤ i < P8 in steps).
    pub fn interval_to(self, other: Self) -> SpelledInterval {
        let f = other.fifths - self.fifths;
        // Choose octaves so the diatonic step count 4f + 7o is in 0..7.
        let o = -(4 * f).div_euclid(7);
        SpelledInterval {
            fifths: f,
            octaves: o,
        }
    }

    /// e.g. "E♭", "B𝄫", "F♯" (`unicode`) or "Eb", "Bbb", "F#".
    pub fn name(self, unicode: bool) -> String {
        let mut s = String::new();
        s.push(self.letter());
        push_accidentals(&mut s, self.accidentals(), unicode);
        s
    }

    /// Parses "Eb", "e♭", "F##", "Bx", "C𝄫". Whitespace is not allowed.
    pub fn parse(text: &str) -> Option<Self> {
        let mut chars = text.chars();
        let letter = chars.next()?;
        let mut acc = 0;
        for c in chars {
            acc += match c {
                '#' | '♯' => 1,
                'b' | '♭' => -1,
                'x' | '𝄪' => 2,
                '𝄫' => -2,
                '♮' => 0,
                _ => return None,
            };
        }
        Self::new(letter, acc)
    }
}

fn push_accidentals(s: &mut String, acc: i32, unicode: bool) {
    match (acc, unicode) {
        (2, true) => s.push('𝄪'),
        (-2, true) => s.push('𝄫'),
        (a, true) if a > 0 => (0..a).for_each(|_| s.push('♯')),
        (a, true) => (0..-a).for_each(|_| s.push('♭')),
        (a, false) if a > 0 => (0..a).for_each(|_| s.push('#')),
        (a, false) => (0..-a).for_each(|_| s.push('b')),
    }
}

impl Add<SpelledInterval> for SpelledPc {
    type Output = Self;
    fn add(self, i: SpelledInterval) -> Self {
        Self {
            fifths: self.fifths + i.fifths,
        }
    }
}

impl Sub<SpelledInterval> for SpelledPc {
    type Output = Self;
    fn sub(self, i: SpelledInterval) -> Self {
        Self {
            fifths: self.fifths - i.fifths,
        }
    }
}

impl fmt::Display for SpelledPc {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name(true))
    }
}

/// Interval quality. `Diminished(1)` is d, `Diminished(2)` is dd, and so on.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum IntervalQuality {
    Diminished(u8),
    Minor,
    Perfect,
    Major,
    Augmented(u8),
}

/// A spelled interval: `fifths` perfect fifths plus `octaves` octaves.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub struct SpelledInterval {
    fifths: i32,
    octaves: i32,
}

/// Fifths coordinate of the perfect or major interval for each simple generic class
/// (unison, 2nd, …, 7th).
const CENTRE: [i32; 7] = [0, 2, 4, -1, 1, 3, 5];
const PERFECT: [bool; 7] = [true, false, false, true, true, false, false];

impl SpelledInterval {
    pub const UNISON: Self = Self {
        fifths: 0,
        octaves: 0,
    };
    pub const OCTAVE: Self = Self {
        fifths: 0,
        octaves: 1,
    };
    pub const FIFTH: Self = Self {
        fifths: 1,
        octaves: 0,
    };

    pub const fn from_coords(fifths: i32, octaves: i32) -> Self {
        Self { fifths, octaves }
    }

    /// An ascending interval from its quality and number (1 = unison, 9 = ninth, …).
    /// Returns `None` for impossible pairs such as a major fifth or a perfect third.
    pub fn new(quality: IntervalQuality, number: u8) -> Option<Self> {
        let steps = number.checked_sub(1)? as i32;
        let generic = (steps % 7) as usize;
        let shift = match (quality, PERFECT[generic]) {
            (IntervalQuality::Perfect, true) | (IntervalQuality::Major, false) => 0,
            (IntervalQuality::Minor, false) => -7,
            (IntervalQuality::Augmented(k), _) => 7 * k as i32,
            (IntervalQuality::Diminished(k), true) => -7 * k as i32,
            (IntervalQuality::Diminished(k), false) => -7 * (k as i32 + 1),
            _ => return None,
        };
        let fifths = CENTRE[generic] + shift;
        // steps = 4·fifths + 7·octaves
        let octaves = (steps - 4 * fifths).div_euclid(7);
        debug_assert_eq!(4 * fifths + 7 * octaves, steps);
        Some(Self { fifths, octaves })
    }

    pub const fn fifths(self) -> i32 {
        self.fifths
    }

    pub const fn octaves(self) -> i32 {
        self.octaves
    }

    pub const fn semitones(self) -> i32 {
        7 * self.fifths + 12 * self.octaves
    }

    /// Diatonic steps: 0 for a unison, 2 for a third, 8 for a ninth. Negative when descending.
    pub const fn steps(self) -> i32 {
        4 * self.fifths + 7 * self.octaves
    }

    /// Generic interval number of the ascending form: 1 = unison, 3 = third, 10 = tenth.
    pub const fn number(self) -> i32 {
        self.steps().abs() + 1
    }

    pub fn quality(self) -> IntervalQuality {
        let i = if self.steps() < 0 { -self } else { self };
        let generic = i.steps().rem_euclid(7) as usize;
        let d = i.fifths - CENTRE[generic];
        match (PERFECT[generic], d) {
            (_, 0) => {
                if PERFECT[generic] {
                    IntervalQuality::Perfect
                } else {
                    IntervalQuality::Major
                }
            }
            (false, -7) => IntervalQuality::Minor,
            (_, d) if d > 0 => IntervalQuality::Augmented((d / 7) as u8),
            (true, d) => IntervalQuality::Diminished((-d / 7) as u8),
            (false, d) => IntervalQuality::Diminished((-d / 7 - 1) as u8),
        }
    }

    /// The interval reduced to less than an octave (the same pitch-class relation).
    pub fn simple(self) -> Self {
        Self {
            fifths: self.fifths,
            octaves: -(4 * self.fifths).div_euclid(7),
        }
    }

    /// e.g. "M3", "m9", "A4", "d7", "P8".
    pub fn name(self) -> String {
        let q = match self.quality() {
            IntervalQuality::Perfect => String::from("P"),
            IntervalQuality::Major => String::from("M"),
            IntervalQuality::Minor => String::from("m"),
            IntervalQuality::Augmented(k) => "A".repeat(k as usize),
            IntervalQuality::Diminished(k) => "d".repeat(k as usize),
        };
        let sign = if self.steps() < 0 { "-" } else { "" };
        format!("{sign}{q}{}", self.number())
    }
}

impl Add for SpelledInterval {
    type Output = Self;
    fn add(self, o: Self) -> Self {
        Self {
            fifths: self.fifths + o.fifths,
            octaves: self.octaves + o.octaves,
        }
    }
}

impl Sub for SpelledInterval {
    type Output = Self;
    fn sub(self, o: Self) -> Self {
        Self {
            fifths: self.fifths - o.fifths,
            octaves: self.octaves - o.octaves,
        }
    }
}

impl Neg for SpelledInterval {
    type Output = Self;
    fn neg(self) -> Self {
        Self {
            fifths: -self.fifths,
            octaves: -self.octaves,
        }
    }
}

impl fmt::Display for SpelledInterval {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name())
    }
}

#[cfg(test)]
mod tests {
    use super::IntervalQuality::*;
    use super::*;
    use proptest::prelude::*;

    fn iv(q: IntervalQuality, n: u8) -> SpelledInterval {
        SpelledInterval::new(q, n).unwrap()
    }
    fn sp(s: &str) -> SpelledPc {
        SpelledPc::parse(s).unwrap()
    }

    #[test]
    fn letters_and_accidentals() {
        let names: Vec<String> = (-15..=15)
            .map(|f| SpelledPc::from_fifths(f).name(false))
            .collect();
        assert_eq!(names[15], "C");
        assert_eq!(SpelledPc::from_fifths(-9).name(true), "B𝄫");
        assert_eq!(SpelledPc::from_fifths(12).name(true), "B♯");
        assert_eq!(SpelledPc::from_fifths(-5).name(true), "D♭");
        assert_eq!(SpelledPc::from_fifths(14).name(false), "C##");
        assert_eq!(sp("Bbb").pc(), PitchClass::new(9));
        assert_eq!(sp("B#").pc(), PitchClass::new(0));
        assert_eq!(sp("f♯").fifths(), 6);
        assert!(SpelledPc::parse("H").is_none());
        assert!(SpelledPc::parse("C4").is_none());
    }

    #[test]
    fn interval_table() {
        let cases = [
            (Perfect, 1, 0),
            (Minor, 2, 1),
            (Major, 2, 2),
            (Minor, 3, 3),
            (Major, 3, 4),
            (Perfect, 4, 5),
            (Augmented(1), 4, 6),
            (Diminished(1), 5, 6),
            (Perfect, 5, 7),
            (Augmented(1), 5, 8),
            (Minor, 6, 8),
            (Major, 6, 9),
            (Diminished(1), 7, 9),
            (Minor, 7, 10),
            (Major, 7, 11),
            (Perfect, 8, 12),
            (Minor, 9, 13),
            (Augmented(1), 9, 15),
            (Augmented(1), 11, 18),
            (Major, 13, 21),
            (Diminished(2), 5, 5),
        ];
        for (q, n, st) in cases {
            let i = iv(q, n);
            assert_eq!(i.semitones(), st, "{q:?}{n}");
            assert_eq!(i.number(), n as i32);
            assert_eq!(i.quality(), q);
        }
        assert!(SpelledInterval::new(Major, 5).is_none());
        assert!(SpelledInterval::new(Perfect, 3).is_none());
        assert!(SpelledInterval::new(Major, 0).is_none());
    }

    #[test]
    fn names() {
        assert_eq!(iv(Major, 3).name(), "M3");
        assert_eq!(iv(Diminished(1), 7).name(), "d7");
        assert_eq!(iv(Augmented(1), 9).name(), "A9");
        assert_eq!(SpelledInterval::OCTAVE.name(), "P8");
        assert_eq!((-iv(Minor, 3)).name(), "-m3");
    }

    #[test]
    fn chord_spelling() {
        let c = sp("C");
        let dim7: Vec<String> = [
            iv(Perfect, 1),
            iv(Minor, 3),
            iv(Diminished(1), 5),
            iv(Diminished(1), 7),
        ]
        .iter()
        .map(|&i| (c + i).name(true))
        .collect();
        assert_eq!(dim7, ["C", "E♭", "G♭", "B𝄫"]);
        assert_eq!((c + iv(Augmented(1), 9)).name(true), "D♯");
        assert_eq!((sp("Ab") + iv(Major, 3)).name(true), "C");
        assert_eq!((sp("F#") + iv(Major, 7)).name(true), "E♯");
    }

    #[test]
    fn interval_between_spelled_pcs() {
        assert_eq!(sp("C").interval_to(sp("Eb")).name(), "m3");
        assert_eq!(sp("C").interval_to(sp("D#")).name(), "A2");
        assert_eq!(sp("B").interval_to(sp("F")).name(), "d5");
        assert_eq!(sp("F").interval_to(sp("B")).name(), "A4");
        assert_eq!(sp("G").interval_to(sp("C")).name(), "P4");
        assert_eq!(sp("C").interval_to(sp("C")).name(), "P1");
    }

    #[test]
    fn spellings_of_pc() {
        let s: Vec<String> = SpelledPc::spellings(PitchClass::new(1), 1)
            .map(|s| s.name(false))
            .collect();
        assert_eq!(s, ["Db", "C#"]);
        let s: Vec<String> = SpelledPc::spellings(PitchClass::new(0), 2)
            .map(|s| s.name(false))
            .collect();
        assert_eq!(s, ["Dbb", "C", "B#"]);
    }

    #[test]
    fn octave_for_midi() {
        assert_eq!(sp("C").octave_for_midi(60), 4);
        assert_eq!(sp("B#").octave_for_midi(60), 3);
        assert_eq!(sp("Cb").octave_for_midi(59), 4);
        assert_eq!(sp("Bbb").octave_for_midi(57), 3);
    }

    proptest! {
        #[test]
        fn group_laws(f1 in -30i32..30, o1 in -5i32..5, f2 in -30i32..30, o2 in -5i32..5) {
            let a = SpelledInterval::from_coords(f1, o1);
            let b = SpelledInterval::from_coords(f2, o2);
            prop_assert_eq!(a + b - b, a);
            prop_assert_eq!(a + b, b + a);
            prop_assert_eq!((a + b).semitones(), a.semitones() + b.semitones());
            prop_assert_eq!((a + b).steps(), a.steps() + b.steps());
            prop_assert_eq!(a + (-a), SpelledInterval::UNISON);
        }

        #[test]
        fn quality_number_roundtrip(f in -20i32..20, o in -3i32..6) {
            let i = SpelledInterval::from_coords(f, o);
            prop_assume!(i.steps() >= 0);
            let back = SpelledInterval::new(i.quality(), i.number() as u8).unwrap();
            prop_assert_eq!(back, i);
        }

        #[test]
        fn spelled_pc_agrees_with_z12(f in -30i32..30, g in -30i32..30) {
            let a = SpelledPc::from_fifths(f);
            let b = SpelledPc::from_fifths(g);
            let i = a.interval_to(b);
            prop_assert!((0..7).contains(&i.steps()));
            prop_assert_eq!(a + i, b);
            prop_assert_eq!(PitchClass::new(i.semitones()), PitchClass::new(a.pc().up_to(b.pc()) as i32));
            prop_assert_eq!(SpelledPc::parse(&a.name(false)), Some(a));
            prop_assert_eq!(SpelledPc::new(a.letter(), a.accidentals()), Some(a));
        }
    }
}
