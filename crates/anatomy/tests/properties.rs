//! Invariants that must hold for any voicing, under both models.

use anatomy::{analyze, Analysis, Model, Options};
use harmony::ratio::gcd;
use harmony::{PitchClass, RatioSet, SpelledPc};
use proptest::prelude::*;

fn voicing() -> impl Strategy<Value = Vec<u8>> {
    (30u8..70, prop::collection::btree_set(1u8..36, 1..7)).prop_map(|(bass, ups)| {
        std::iter::once(bass)
            .chain(ups.into_iter().map(|u| bass + u))
            .collect()
    })
}

fn models() -> impl Strategy<Value = Options> {
    prop_oneof![Just(Options::prototype()), Just(Options::default())]
}

fn scores(a: &Analysis) -> Vec<f64> {
    a.readings.iter().map(|r| r.score).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(400))]

    #[test]
    fn transposition_moves_the_root_and_keeps_scores(notes in voicing(), k in 1i32..12, opts in models()) {
        let a = analyze(&notes, &opts).unwrap();
        let up: Vec<u8> = notes.iter().map(|&m| (m as i32 + k) as u8).collect();
        let b = analyze(&up, &opts).unwrap();
        for (x, y) in scores(&a).iter().zip(scores(&b)) {
            prop_assert!((x - y).abs() < 1e-9);
        }
        prop_assert_eq!(&a.periodicity.harmonics, &b.periodicity.harmonics);
        // Tied scores keep root order (a stable sort), so the best reading can differ between
        // transpositions. Compare it only when it is unique.
        if a.readings[0].score - a.readings[1].score > 1e-9 {
            prop_assert_eq!(b.best().root, a.best().root + k);
            let words = |x: &Analysis| x.tags.iter().map(|t| t.word).collect::<Vec<_>>();
            prop_assert_eq!(words(&a), words(&b));
        }
    }

    #[test]
    fn doubling_an_upper_pitch_class_keeps_the_readings(notes in voicing(), pick in 0usize..8, opts in models()) {
        prop_assume!(notes.len() >= 2);
        let i = 1 + pick % (notes.len() - 1);
        let bass_pc = notes[0] % 12;
        prop_assume!(notes[i] % 12 != bass_pc);
        let top = *notes.last().unwrap();
        let mut doubled = notes.clone();
        let mut m = notes[i];
        while m <= top { m += 12; }
        prop_assume!(m < 128);
        doubled.push(m);
        let a = analyze(&notes, &opts).unwrap();
        let b = analyze(&doubled, &opts).unwrap();
        if opts.model == Model::Prototype {
            let sa: Vec<(String, u64)> = a.readings.iter().map(|r| (r.symbol.clone(), (r.score * 1e6).round() as u64)).collect();
            let sb: Vec<(String, u64)> = b.readings.iter().map(|r| (r.symbol.clone(), (r.score * 1e6).round() as u64)).collect();
            prop_assert_eq!(sa, sb);
        } else {
            // The doubling can add a new interval above it (Hindemith's term depends on the
            // voicing), so only the set of readings is fixed.
            let mut sa: Vec<&str> = a.readings.iter().map(|r| r.symbol.as_str()).collect();
            let mut sb: Vec<&str> = b.readings.iter().map(|r| r.symbol.as_str()).collect();
            sa.sort_unstable();
            sb.sort_unstable();
            prop_assert_eq!(sa, sb);
        }
    }

    #[test]
    fn input_order_and_duplicates_do_not_matter(notes in voicing(), seed in any::<u64>(), opts in models()) {
        let mut shuffled = notes.clone();
        shuffled.extend_from_slice(&notes[..1]);
        let n = shuffled.len();
        for j in 0..n {
            let r = (seed.rotate_left(j as u32) as usize) % n;
            shuffled.swap(j, r);
        }
        prop_assert_eq!(analyze(&notes, &opts), analyze(&shuffled, &opts));
    }

    #[test]
    fn outputs_are_bounded_and_consistent(notes in voicing(), opts in models(), et in any::<bool>(), sine in any::<bool>(), set in 0usize..3) {
        let opts = Options {
            tuning: if et { anatomy::Tuning::Et } else { anatomy::Tuning::Just },
            timbre: if sine { anatomy::Timbre::Sine } else { anatomy::Timbre::Harmonic6 },
            ratio_set: if opts.model == Model::Prototype { RatioSet::ALL[set % 2] } else { RatioSet::ALL[set] },
            ..opts
        };
        let a = analyze(&notes, &opts).unwrap();
        for (_, v) in a.axes.labelled() {
            prop_assert!((0.0..=1.0).contains(&v));
        }
        prop_assert!((0.0..=1.0).contains(&a.valence) && (0.0..=1.0).contains(&a.arousal));
        for p in &a.pairs {
            prop_assert!((0.0..=1.0).contains(&p.roughness));
        }
        let total: f64 = a.readings.iter().map(|r| r.prob).sum();
        prop_assert!((total - 1.0).abs() < 1e-9);
        prop_assert!(a.readings.windows(2).all(|w| w[0].score >= w[1].score));
        prop_assert_eq!(a.readings.len(), 12);
        prop_assert!(a.tags.windows(2).all(|w| w[0].weight >= w[1].weight));

        // Harmonic numbers: coprime, and in the ratios' proportions.
        let p = &a.periodicity;
        prop_assert_eq!(p.harmonics.iter().copied().fold(0, gcd), 1);
        for (h, r) in p.harmonics.iter().zip(&p.ratios) {
            prop_assert_eq!(h * r.den(), p.harmonics[0] * r.num());
        }
        prop_assert!((p.period * p.bass_freq - p.bass_cycles as f64).abs() < 1e-9);
    }

    #[test]
    fn every_tone_has_a_function_and_a_correct_spelling(notes in voicing()) {
        let a = analyze(&notes, &Options::default()).unwrap();
        for r in &a.readings {
            for &pc in &a.pcs {
                prop_assert!(r.degree_of(pc).is_some(), "{} has no degree for {:?}", r.symbol, pc);
                let name = r.tone_name(pc);
                let spelled = SpelledPc::parse(name).unwrap();
                prop_assert_eq!(spelled.pc(), pc, "{} spells {:?} as {}", r.symbol, pc, name);
                prop_assert!(spelled.accidentals().abs() <= 2, "{}: {}", r.symbol, name);
            }
            prop_assert_eq!(SpelledPc::parse(&r.root_name).unwrap().pc(), r.root);
            prop_assert!(!r.symbol.contains('?'), "{}", r.symbol);
        }
        // Note names agree with MIDI pitch.
        for (&m, name) in a.notes.iter().zip(&a.note_names) {
            let letters = name.trim_end_matches(|c: char| c.is_ascii_digit() || c == '-');
            let s = SpelledPc::parse(letters).unwrap();
            prop_assert_eq!(s.pc(), PitchClass::of_midi(m));
            let octave: i32 = name[letters.len()..].parse().unwrap();
            prop_assert_eq!(s.octave_for_midi(m), octave);
        }
    }
}
