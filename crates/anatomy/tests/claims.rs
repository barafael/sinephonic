//! Every factual statement in the improved model's tag explanations must be true of the
//! voicing it describes: order, register, intervals, inversion and tuning.

use anatomy::{analyze, interval_name, Analysis, Degree, Options, PitchClass, Third, Tuning};
use proptest::prelude::*;

fn voicing() -> impl Strategy<Value = Vec<u8>> {
    (36u8..66, prop::collection::btree_set(1u8..30, 1..6)).prop_map(|(bass, ups)| {
        std::iter::once(bass)
            .chain(ups.into_iter().map(|u| bass + u))
            .collect()
    })
}

fn plain(s: &str) -> &str {
    s.trim_end_matches(|c: char| c.is_ascii_digit() || c == '-')
}

/// Text between `open` and the next `close` after it.
fn between<'a>(s: &'a str, open: &str, close: char) -> Option<&'a str> {
    let start = s.find(open)? + open.len();
    let end = s[start..].find(close)? + start;
    Some(&s[start..end])
}

/// Indices of notes whose spelled name (without octave) is `name`.
fn notes_named(a: &Analysis, name: &str) -> Vec<usize> {
    (0..a.notes.len())
        .filter(|&i| plain(&a.note_names[i]) == name)
        .collect()
}

fn check(a: &Analysis, tuning: Tuning) -> Result<(), TestCaseError> {
    let best = a.best();
    let bass = plain(&a.note_names[0]).to_string();
    let bass_degree = best.degree_of(a.bass_pc);
    let mut words: Vec<&str> = a.tags.iter().map(|t| t.word).collect();
    words.sort_unstable();
    words.dedup();
    prop_assert_eq!(words.len(), a.tags.len(), "duplicate tags");

    for t in &a.tags {
        let why = t.why.as_str();
        let ctx = format!("{:?} {} [{}]: {}", a.note_names, best.symbol, t.word, why);
        if tuning == Tuning::Et {
            prop_assert!(!why.contains("pure"), "{ctx}: 'pure' in 12-TET");
        }
        if why.contains("over the root in the bass") {
            prop_assert_eq!(best.root, a.bass_pc, "{}", ctx);
        }
        if why.contains("stable, but") {
            prop_assert_eq!(best.root, a.bass_pc, "{}", ctx);
        }
        if let Some(n) = between(why, "The 5th (", ')').filter(|_| why.contains("is in the bass")) {
            prop_assert_eq!(n, bass.as_str(), "{}", ctx);
            prop_assert_eq!(bass_degree, Some(Degree::Fifth), "{}", ctx);
        }
        if let Some(n) = between(why, "The 7th (", ')') {
            prop_assert_eq!(n, bass.as_str(), "{}", ctx);
        }
        if let Some(n) = between(why, "with its 3rd (", ')') {
            prop_assert_eq!(n, bass.as_str(), "{}", ctx);
        }
        match t.word {
            "grinding" => {
                let pair = why.split(" sit ").next().unwrap();
                let (x, y) = pair.split_once('–').unwrap();
                let found = notes_named(a, x).iter().any(|&i| {
                    notes_named(a, y)
                        .iter()
                        .any(|&j| a.notes[j] as i32 - a.notes[i] as i32 == 1)
                });
                prop_assert!(found, "{ctx}: no semitone {x}–{y}");
                if why.contains("near-maximal") {
                    prop_assert!(
                        a.pairs
                            .iter()
                            .any(|p| p.semitones == 1 && p.roughness >= 0.8),
                        "{ctx}"
                    );
                }
            }
            "driving" => {
                let third = between(why, "3 (", ')').unwrap();
                let seventh = between(why, "♭7 (", ')').unwrap();
                let t3 = notes_named(a, third)[0];
                let t7 = notes_named(a, seventh)[0];
                if why.contains("diminished 5th") {
                    prop_assert!(t3 < t7, "{ctx}");
                } else {
                    prop_assert!(why.contains("augmented 4th") && t7 < t3, "{ctx}");
                }
            }
            "aggressive" if why.contains("apart as voiced") => {
                let n3 = between(why, "Major 3rd (", ')').unwrap();
                let s9 = between(why, "♯9 (", ')').unwrap();
                let (i, j) = (notes_named(a, n3)[0], notes_named(a, s9)[0]);
                let (lo, hi) = if i < j { (i, j) } else { (j, i) };
                let iv = interval_name(a.notes[hi] - a.notes[lo]);
                prop_assert!(
                    why.contains(&format!("a {iv} apart")),
                    "{ctx}: expected {iv}"
                );
            }
            "luminous" => prop_assert_eq!(best.third, Third::Major, "{}", ctx),
            "unstable" if why.contains("divide the octave") => {
                prop_assert_eq!(a.pcs.len(), 4, "{}", ctx);
            }
            "piquant" => {
                let s = between(why, "major 7th (", ')').unwrap();
                let r = between(why, "the root (", ')').unwrap();
                let ok = notes_named(a, s).iter().any(|&i| {
                    notes_named(a, r)
                        .iter()
                        .any(|&j| matches!(a.notes[j] as i32 - a.notes[i] as i32, 1 | 13))
                });
                prop_assert!(ok, "{ctx}");
            }
            "unsettled" => prop_assert!(
                matches!(
                    bass_degree,
                    Some(
                        Degree::Fifth
                            | Degree::Seventh
                            | Degree::FlatSeventh
                            | Degree::DoubleFlatSeventh
                    )
                ),
                "{ctx}"
            ),
            "fused" if tuning == Tuning::Et => prop_assert!(why.contains("approximate"), "{ctx}"),
            "fused" => prop_assert!(
                a.periodicity.bass_cycles <= 4 && why.contains("are harmonics"),
                "{ctx}"
            ),
            "blended" => prop_assert!((5..=8).contains(&a.periodicity.bass_cycles), "{ctx}"),
            "diffuse" => prop_assert!(a.periodicity.bass_cycles >= 120, "{ctx}"),
            "doubled" => prop_assert_eq!(a.pcs.len(), 1, "{}", ctx),
            "hollow" => prop_assert_eq!(a.pcs.len(), 2, "{}", ctx),
            "overtone" => prop_assert!(a.periodicity.bass_cycles == 1 && a.pcs.len() > 1, "{ctx}"),
            "muddy" => {
                let (x, rest) = why.split_once('–').unwrap();
                let y = rest.split(' ').next().unwrap();
                let i = a.note_names.iter().position(|n| n == x).unwrap();
                let j = a.note_names.iter().position(|n| n == y).unwrap();
                prop_assert!(a.notes[j] - a.notes[i] <= 4 && a.notes[i] < 48, "{ctx}");
            }
            "close" => prop_assert!(a.notes[a.notes.len() - 1] - a.notes[0] <= 12, "{ctx}"),
            "spread" => prop_assert!(a.notes[a.notes.len() - 1] - a.notes[0] >= 24, "{ctx}"),
            "airy" => prop_assert!(a.notes[0] >= 67, "{ctx}"),
            "grounded" | "offset" => {
                let below = 12.0 * (a.periodicity.bass_cycles as f64).log2();
                let f = PitchClass::new(a.notes[0] as i32 - below.round() as i32);
                prop_assert_eq!(f == best.root, t.word == "grounded", "{}", ctx);
            }
            "cloudy" if tuning == Tuning::Et => {
                prop_assert!(why.contains("never repeats"), "{ctx}")
            }
            _ => {}
        }
    }
    Ok(())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1500))]

    #[test]
    fn tag_claims_hold(notes in voicing(), just in any::<bool>()) {
        let tuning = if just { Tuning::Just } else { Tuning::Et };
        let a = analyze(&notes, &Options { tuning, ..Options::default() }).unwrap();
        check(&a, tuning)?;
    }
}

#[test]
fn six_four_is_not_called_stable() {
    // C–F–A♭: F minor over its 5th.
    let a = analyze(&[48, 53, 56], &Options::default()).unwrap();
    assert_eq!(a.best().symbol, "Fm/C");
    let words: Vec<&str> = a.tags.iter().map(|t| t.word).collect();
    assert_eq!(&words[..2], ["unsettled", "somber"], "{:?}", a.tags);
    assert!(a.tags.iter().all(|t| !t.why.contains("stable, but")));
    assert_eq!(a.summary(), "Unsettled and somber");
    let root_position = analyze(&[53, 56, 60], &Options::default()).unwrap();
    assert!(a.axes.stability < root_position.axes.stability);
}

#[test]
fn tritone_direction_follows_the_voicing() {
    let why = |notes: &[u8]| {
        let a = analyze(notes, &Options::default()).unwrap();
        a.tags
            .iter()
            .find(|t| t.word == "driving")
            .unwrap()
            .why
            .clone()
    };
    assert!(why(&[48, 52, 55, 58]).contains("inward")); // C E G B♭: E below B♭
    assert!(why(&[46, 48, 52, 55]).contains("outward")); // B♭ C E G: B♭ below E
}

#[test]
fn major_seventh_under_the_root_is_a_clash() {
    let a = analyze(&[47, 48, 52, 55], &Options::default()).unwrap(); // B2 C3 E3 G3
    assert_eq!(a.best().symbol, "Cmaj7/B");
    assert!(a.tags.iter().any(|t| t.word == "piquant"), "{:?}", a.tags);
    let b = analyze(&[48, 52, 55, 59], &Options::default()).unwrap();
    assert!(b.tags.iter().any(|t| t.word == "lush"));
    let _ = PitchClass::C;
}
