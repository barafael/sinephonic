//! Named sonorities from the repertoire.

use harmony::{PcSet, PitchClass};

struct Named {
    name: &'static str,
    /// Pitch classes of one instance; any transposition matches.
    pcs: &'static [u8],
    /// When set, the voicing's adjacent intervals (bass up) must be exactly these.
    spacing: Option<&'static [u8]>,
}

const NAMED: &[Named] = &[
    // Wagner, Tristan und Isolde, bar 2: F B D♯ G♯.
    Named {
        name: "Tristan chord",
        pcs: &[5, 11, 3, 8],
        spacing: Some(&[6, 4, 5]),
    },
    // Scriabin, Prometheus: C F♯ B♭ E A D in fourths.
    Named {
        name: "Mystic chord",
        pcs: &[0, 6, 10, 4, 9, 2],
        spacing: None,
    },
    // Stravinsky, Petrushka: C major against F♯ major.
    Named {
        name: "Petrushka chord",
        pcs: &[0, 4, 7, 6, 10, 1],
        spacing: None,
    },
    // Miles Davis/Bill Evans, "So What": three fourths and a major third, E A D G B.
    Named {
        name: "So What chord",
        pcs: &[4, 9, 2, 7, 11],
        spacing: Some(&[5, 5, 5, 4]),
    },
    // Schoenberg, Farben (op. 16/3): C G♯ B E A.
    Named {
        name: "Farben chord",
        pcs: &[0, 8, 11, 4, 9],
        spacing: None,
    },
    // La Monte Young: G C C♯ D.
    Named {
        name: "Dream chord",
        pcs: &[7, 0, 1, 2],
        spacing: None,
    },
    // Steely Dan's "mu major": an add2 with the 2 right under the 3, C D E G.
    Named {
        name: "Mu chord",
        pcs: &[0, 2, 4, 7],
        spacing: Some(&[2, 2, 3]),
    },
    // Jimi Hendrix, "Purple Haze": E G♯ D G (7♯9 with the ♯9 on top).
    Named {
        name: "Hendrix chord",
        pcs: &[4, 8, 2, 7],
        spacing: Some(&[4, 6, 5]),
    },
    Named {
        name: "Hendrix chord",
        pcs: &[4, 8, 11, 2, 7],
        spacing: Some(&[7, 5, 3, 5]),
    },
];

/// The repertoire name for this voicing, if any. `notes` sorted ascending, unique.
pub fn named(notes: &[u8]) -> Option<&'static str> {
    let pcs = PcSet::from_midi(notes);
    let spacing: Vec<u8> = notes.windows(2).map(|w| w[1] - w[0]).collect();
    NAMED.iter().find_map(|n| {
        let set: PcSet = n.pcs.iter().map(|&p| PitchClass::from(p)).collect();
        let same_class =
            set.len() == pcs.len() && set.transposition_class() == pcs.transposition_class();
        let spaced = n.spacing.is_none_or(|s| s == spacing.as_slice());
        (same_class && spaced).then_some(n.name)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repertoire() {
        assert_eq!(named(&[53, 59, 63, 68]), Some("Tristan chord"));
        // Same notes, different spacing: just a half-diminished chord.
        assert_eq!(named(&[53, 56, 59, 63]), None);
        assert_eq!(named(&[48, 54, 58, 64, 69, 74]), Some("Mystic chord"));
        assert_eq!(named(&[52, 57, 62, 67, 71]), Some("So What chord"));
        assert_eq!(named(&[50, 55, 60, 65, 69]), Some("So What chord"));
        assert_eq!(named(&[40, 44, 50, 55]), Some("Hendrix chord"));
        assert_eq!(named(&[48, 50, 52, 55]), Some("Mu chord"));
        assert_eq!(named(&[48, 52, 55]), None);
        assert_eq!(
            named(
                &[48, 52, 55, 54, 58, 61]
                    .iter()
                    .copied()
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
            ),
            Some("Petrushka chord")
        );
    }
}
