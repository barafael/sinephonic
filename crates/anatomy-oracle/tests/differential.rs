//! The Rust port (`anatomy::reference`) against the prototype's JavaScript, field by field.

use anatomy::{Analysis, Options, Timbre, Tuning};
use anatomy_oracle::Oracle;
use harmony::{RatioSet, Spelling};
use serde_json::Value;

fn close(a: f64, b: f64, what: &str, ctx: &str) {
    let tol = 1e-9 * a.abs().max(b.abs()).max(1.0);
    assert!((a - b).abs() <= tol, "{ctx}: {what}: rust {a} vs js {b}");
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
}

fn compare(r: &Analysis, js: &Value, ctx: &str) {
    let p = &r.periodicity;
    let ints: Vec<u64> = js["ints"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| f(x) as u64)
        .collect();
    assert_eq!(p.harmonics, ints, "{ctx}: harmonics");
    assert_eq!(p.bass_cycles, f(&js["P"]) as u64, "{ctx}: P");
    close(p.fundamental, f(&js["f0"]), "f0", ctx);
    close(p.period, f(&js["period"]), "period", ctx);
    let rats: Vec<String> = p.ratios.iter().map(|r| r.to_string()).collect();
    let js_rats: Vec<&str> = js["rats"]
        .as_array()
        .unwrap()
        .iter()
        .map(|x| x.as_str().unwrap())
        .collect();
    assert_eq!(rats, js_rats, "{ctx}: ratios");
    for (k, (ours, key)) in [
        (&p.just_freqs, "justF"),
        (&p.et_freqs, "etF"),
        (&p.cents, "cents"),
    ]
    .into_iter()
    .enumerate()
    {
        let theirs = js[key].as_array().unwrap();
        assert_eq!(ours.len(), theirs.len());
        for (a, b) in ours.iter().zip(theirs) {
            let _ = k;
            // cents are differences of logs; compare absolutely.
            assert!(
                (a - f(b)).abs() <= 1e-9 * a.abs().max(1.0),
                "{ctx}: {key}: {a} vs {b}"
            );
        }
    }

    let all = js["all"].as_array().unwrap();
    assert_eq!(r.readings.len(), all.len());
    for (k, (ours, theirs)) in r.readings.iter().zip(all).enumerate() {
        let c = format!("{ctx}: reading #{k}");
        assert_eq!(ours.root.value() as f64, f(&theirs["root"]), "{c}: root");
        assert_eq!(ours.symbol, theirs["name"].as_str().unwrap(), "{c}: name");
        close(ours.score, f(&theirs["score"]), "score", &c);
        close(ours.prob, f(&theirs["p"]), "prob", &c);
        assert_eq!(
            ours.root_present,
            theirs["rootPresent"].as_bool().unwrap(),
            "{c}"
        );
        assert_eq!(ours.alterations as f64, f(&theirs["alts"]), "{c}: alts");
        let deg: Vec<Option<&str>> = theirs["deg"]
            .as_array()
            .unwrap()
            .iter()
            .map(|d| d.as_str())
            .collect();
        let our_deg: Vec<Option<&str>> =
            ours.degrees.iter().map(|d| d.map(|d| d.label())).collect();
        assert_eq!(our_deg, deg, "{c}: degrees");
    }

    let pairs = js["pairs"].as_array().unwrap();
    assert_eq!(r.pairs.len(), pairs.len());
    for (ours, theirs) in r.pairs.iter().zip(pairs) {
        let t = theirs.as_array().unwrap();
        assert_eq!(
            (ours.i, ours.j, ours.semitones as usize),
            (f(&t[0]) as usize, f(&t[1]) as usize, f(&t[2]) as usize)
        );
        close(ours.roughness, f(&t[3]), "roughness", ctx);
    }

    let tags = js["tags"].as_array().unwrap();
    let ours: Vec<(&str, String)> = r.tags.iter().map(|t| (t.word, t.why.clone())).collect();
    let theirs: Vec<(&str, String)> = tags
        .iter()
        .map(|t| (t[0].as_str().unwrap(), t[2].as_str().unwrap().to_string()))
        .collect();
    assert_eq!(ours, theirs, "{ctx}: tags");

    let ax = &js["axes"];
    close(r.axes.tension, f(&ax["tension"]), "tension", ctx);
    close(r.axes.harshness, f(&ax["harsh"]), "harshness", ctx);
    close(r.axes.aggression, f(&ax["aggression"]), "aggression", ctx);
    close(r.axes.complexity, f(&ax["complexity"]), "complexity", ctx);
    close(r.axes.brightness, f(&ax["brightness"]), "brightness", ctx);
    close(r.axes.stability, f(&ax["stability"]), "stability", ctx);
    close(r.axes.ambiguity, f(&ax["ambiguity"]), "ambiguity", ctx);
    close(r.valence, f(&js["valence"]), "valence", ctx);
    close(r.arousal, f(&js["arousal"]), "arousal", ctx);
}

fn check(oracle: &mut Oracle, notes: &[u8], opts: &Options) {
    let ctx = format!("{notes:?} {opts:?}");
    let r = anatomy::analyze(notes, opts).unwrap();
    let js = oracle.analyze(notes, opts);
    compare(&r, &js, &ctx);
}

fn proto() -> Options {
    Options::prototype()
}

/// Tiny deterministic PRNG so failures reproduce.
struct Lcg(u64);
impl Lcg {
    fn next(&mut self) -> u64 {
        self.0 = self
            .0
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        self.0 >> 33
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[test]
fn handoff_presets_under_every_option() {
    let presets: [&[u8]; 9] = [
        &[48, 55, 56, 63],
        &[48, 52, 55, 58, 63],
        &[48, 52, 56, 58, 63],
        &[48, 52, 55, 59],
        &[48, 52, 55],
        &[48, 51, 55],
        &[48, 52, 55, 58, 61],
        &[50, 55, 60, 65],
        &[60, 61, 62],
    ];
    let mut o = Oracle::new();
    for notes in presets {
        for spelling in [Spelling::Flats, Spelling::Sharps] {
            for ratio_set in [RatioSet::FiveLimit, RatioSet::SevenLimit] {
                for timbre in [Timbre::Sine, Timbre::Harmonic6] {
                    for tuning in [Tuning::Just, Tuning::Et] {
                        for a4 in [415.0, 440.0, 466.0] {
                            let opts = Options {
                                spelling,
                                ratio_set,
                                timbre,
                                tuning,
                                a4,
                                ..proto()
                            };
                            check(&mut o, notes, &opts);
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn every_two_to_four_note_voicing_in_two_octaves() {
    let mut o = Oracle::new();
    let lo = 48u8;
    let hi = 72u8;
    let opts = proto();
    let mut count = 0;
    for a in lo..=hi {
        for b in a + 1..=hi {
            check(&mut o, &[a, b], &opts);
            count += 1;
            for c in b + 1..=hi {
                check(&mut o, &[a, b, c], &opts);
                count += 1;
                for d in c + 1..=hi {
                    // All tetrads with a fixed bass keep this fast; transposition is covered by
                    // the random test.
                    if a == lo {
                        check(&mut o, &[a, b, c, d], &opts);
                        count += 1;
                    }
                }
            }
        }
    }
    assert!(count > 4000, "{count}");
}

#[test]
fn random_voicings_and_options() {
    let mut o = Oracle::new();
    let mut rng = Lcg(0x5eed_c0de);
    for _ in 0..3000 {
        let n = 2 + rng.below(6) as usize; // 2..=7 notes
        let bass = 28 + rng.below(40) as u8;
        let span = 12 + rng.below(36) as u8;
        let mut notes: Vec<u8> = vec![bass];
        while notes.len() < n {
            let m = bass + 1 + rng.below(span as u64) as u8;
            if !notes.contains(&m) {
                notes.push(m);
            }
        }
        notes.sort_unstable();
        let opts = Options {
            spelling: if rng.below(2) == 0 {
                Spelling::Flats
            } else {
                Spelling::Sharps
            },
            ratio_set: if rng.below(2) == 0 {
                RatioSet::FiveLimit
            } else {
                RatioSet::SevenLimit
            },
            timbre: if rng.below(2) == 0 {
                Timbre::Sine
            } else {
                Timbre::Harmonic6
            },
            tuning: if rng.below(2) == 0 {
                Tuning::Just
            } else {
                Tuning::Et
            },
            a4: 415.0 + rng.below(52) as f64,
            ..proto()
        };
        check(&mut o, &notes, &opts);
    }
}
