//! Calibration of the improved model's tension: orderings any listener would agree with.

use anatomy::{analyze, Options};

fn tension(notes: &[u8]) -> f64 {
    analyze(notes, &Options::default()).unwrap().axes.tension
}

#[test]
fn tension_orderings() {
    let octave = tension(&[48, 60]);
    let fifth = tension(&[60, 67]);
    let major = tension(&[60, 64, 67]);
    let maj7 = tension(&[60, 64, 67, 71]);
    let sharp9 = tension(&[48, 52, 55, 58, 63]);
    let dim7 = tension(&[48, 51, 54, 57]);
    let cluster = tension(&[60, 61, 62]);
    assert!(
        octave < fifth && fifth < major && major < maj7 && maj7 < sharp9,
        "{octave} {fifth} {major} {maj7} {sharp9}"
    );
    assert!(sharp9 < cluster && dim7 < cluster);
    assert!(major < 0.35, "a close C4 major triad is not tense: {major}");
    assert!(cluster > 0.8, "{cluster}");
    // Lower registers are rougher, but a C3 triad is still calm.
    let low = tension(&[48, 52, 55]);
    assert!(low > major && low < 0.5, "{low}");
    // Major and minor are equally tense to within a little.
    assert!((tension(&[60, 63, 67]) - major).abs() < 0.1);
    // Not saturated: distinct five-note chords stay distinguishable.
    assert!(sharp9 < 0.8);
}

#[test]
fn six_four_is_less_stable_but_not_zero() {
    let six_four = analyze(&[48, 53, 56], &Options::default())
        .unwrap()
        .axes
        .stability;
    let root = analyze(&[53, 56, 60], &Options::default())
        .unwrap()
        .axes
        .stability;
    assert!(six_four < root && six_four > 0.05, "{six_four} {root}");
}
