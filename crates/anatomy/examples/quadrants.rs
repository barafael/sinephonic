//! Where chords land on the valence × arousal map, and how many fall in each quadrant.
//!
//!     cargo run -p anatomy --example quadrants

use anatomy::{analyze, corpus, Options};

const PROBES: &[(&str, &str)] = &[
    ("C major", "C4 E4 G4"),
    ("C major low", "C3 E3 G3"),
    ("Cmaj7", "C3 E3 G3 B3"),
    ("C6/9", "C3 E3 A3 D4 G4"),
    ("Cadd9 high", "C5 D5 E5 G5"),
    ("Cmaj7♯11", "C3 E3 B3 F#4 G4"),
    ("C lydian stack", "C3 G3 D4 F#4 B4"),
    ("C13♯11", "C3 E3 Bb3 D4 F#4 A4"),
    ("C7♯11", "C3 E3 G3 Bb3 F#4"),
    ("C augmented", "C4 E4 G#4"),
    ("Cmaj7♯5", "C3 E3 G#3 B3"),
    ("D/C upper structure", "C3 D4 F#4 A4"),
    ("whole-tone high", "C5 D5 E5 F#5"),
    ("C minor", "C3 Eb3 G3"),
    ("Cm9", "C3 Eb3 G3 Bb3 D4"),
    ("Am low", "A2 E3 C4"),
    ("Cm(♭6)", "C3 G3 Ab3 Eb4"),
    ("C°7", "C3 Eb3 Gb3 A3"),
    ("C7♭9 low", "C2 E2 Bb2 Db3"),
    ("cluster low", "C2 Db2 D2"),
    ("Cm(maj7)", "C3 Eb3 G3 B3"),
    ("tritone low", "C2 F#2"),
    ("C7♯9", "C3 E3 G3 Bb3 Eb4"),
];

fn quadrant(v: f64, a: f64) -> usize {
    (if v >= 0.5 { 1 } else { 0 }) + (if a >= 0.5 { 2 } else { 0 })
}

const NAMES: [&str; 4] = [
    "dark · calm",
    "bright · calm",
    "dark · tense",
    "bright · tense",
];

fn main() {
    let opts = Options::default();
    let mut counts = [0usize; 4];
    println!("{:24} {:>5} {:>5}  quadrant", "probe", "val", "aro");
    for (name, text) in PROBES {
        let a = analyze(&harmony::midi::parse_notes(text), &opts).unwrap();
        println!(
            "{name:24} {:5.2} {:5.2}  {}",
            a.valence,
            a.arousal,
            NAMES[quadrant(a.valence, a.arousal)]
        );
    }
    let entries = corpus::parse(include_str!("../tests/corpus.txt")).unwrap();
    for e in &entries {
        let a = analyze(&e.notes, &opts).unwrap();
        counts[quadrant(a.valence, a.arousal)] += 1;
    }
    println!("\ncorpus ({} voicings):", entries.len());
    for (k, c) in counts.iter().enumerate() {
        println!("  {:16} {c}", NAMES[k]);
    }
}
