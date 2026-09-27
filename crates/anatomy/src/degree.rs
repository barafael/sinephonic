//! Chord degrees: how an interval above the root functions in a reading.

use harmony::spelled::{IntervalQuality::*, SpelledInterval};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Degree {
    Root,
    Second,
    FlatThird,
    Third,
    Fourth,
    FlatFifth,
    Fifth,
    SharpFifth,
    FlatSix,
    Six,
    DoubleFlatSeventh,
    FlatSeventh,
    Seventh,
    FlatNine,
    Nine,
    SharpNine,
    Eleven,
    SharpEleven,
    FlatThirteen,
    Thirteen,
}

impl Degree {
    pub fn label(self) -> &'static str {
        match self {
            Self::Root => "R",
            Self::Second => "2",
            Self::FlatThird => "♭3",
            Self::Third => "3",
            Self::Fourth => "4",
            Self::FlatFifth => "♭5",
            Self::Fifth => "5",
            Self::SharpFifth => "♯5",
            Self::FlatSix => "♭6",
            Self::Six => "6",
            Self::DoubleFlatSeventh => "♭♭7",
            Self::FlatSeventh => "♭7",
            Self::Seventh => "7",
            Self::FlatNine => "♭9",
            Self::Nine => "9",
            Self::SharpNine => "♯9",
            Self::Eleven => "11",
            Self::SharpEleven => "♯11",
            Self::FlatThirteen => "♭13",
            Self::Thirteen => "13",
        }
    }

    /// Harte-style degree: "b3", "#11", "bb7".
    pub fn harte(self) -> String {
        self.label()
            .replace('♭', "b")
            .replace('♯', "#")
            .replace('R', "1")
    }

    /// The spelled interval above the root, so the chord tone can be spelled correctly
    /// (a ♯9 over C is D♯, a ♭♭7 is B𝄫).
    pub fn interval(self) -> SpelledInterval {
        let (q, n) = match self {
            Self::Root => (Perfect, 1),
            Self::Second => (Major, 2),
            Self::FlatThird => (Minor, 3),
            Self::Third => (Major, 3),
            Self::Fourth => (Perfect, 4),
            Self::FlatFifth => (Diminished(1), 5),
            Self::Fifth => (Perfect, 5),
            Self::SharpFifth => (Augmented(1), 5),
            Self::FlatSix => (Minor, 6),
            Self::Six => (Major, 6),
            Self::DoubleFlatSeventh => (Diminished(1), 7),
            Self::FlatSeventh => (Minor, 7),
            Self::Seventh => (Major, 7),
            Self::FlatNine => (Minor, 9),
            Self::Nine => (Major, 9),
            Self::SharpNine => (Augmented(1), 9),
            Self::Eleven => (Perfect, 11),
            Self::SharpEleven => (Augmented(1), 11),
            Self::FlatThirteen => (Minor, 13),
            Self::Thirteen => (Major, 13),
        };
        SpelledInterval::new(q, n).expect("valid degree interval")
    }

    /// Semitones above the root, mod 12.
    pub fn ic(self) -> u8 {
        self.interval().semitones().rem_euclid(12) as u8
    }

    pub fn is_alteration(self) -> bool {
        matches!(
            self,
            Self::FlatNine
                | Self::SharpNine
                | Self::SharpEleven
                | Self::FlatThirteen
                | Self::FlatSix
                | Self::FlatFifth
        )
    }
}

/// How the third is (or is not) filled.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Third {
    Major,
    Minor,
    Sus4,
    Sus2,
    None,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Seventh {
    Major,
    Dominant,
    Diminished,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intervals_match_labels() {
        let all = [
            (Degree::Root, 0),
            (Degree::Second, 2),
            (Degree::FlatThird, 3),
            (Degree::Third, 4),
            (Degree::Fourth, 5),
            (Degree::FlatFifth, 6),
            (Degree::Fifth, 7),
            (Degree::SharpFifth, 8),
            (Degree::FlatSix, 8),
            (Degree::Six, 9),
            (Degree::DoubleFlatSeventh, 9),
            (Degree::FlatSeventh, 10),
            (Degree::Seventh, 11),
            (Degree::FlatNine, 1),
            (Degree::Nine, 2),
            (Degree::SharpNine, 3),
            (Degree::Eleven, 5),
            (Degree::SharpEleven, 6),
            (Degree::FlatThirteen, 8),
            (Degree::Thirteen, 9),
        ];
        for (d, ic) in all {
            assert_eq!(d.ic(), ic, "{d:?}");
        }
        assert_eq!(Degree::DoubleFlatSeventh.harte(), "bb7");
        assert_eq!(Degree::Root.harte(), "1");
    }
}
