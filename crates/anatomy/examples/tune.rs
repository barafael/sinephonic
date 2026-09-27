//! Grid search over the improved model's root weights, scored on `tests/corpus.txt`.
//! Prints the best weight sets and the entries they still miss.
//!
//!     cargo run --release -p anatomy --example tune

use anatomy::corpus;
use anatomy::improved::reading::{readings, Weights, WEIGHTS};
use harmony::Spelling;

fn evaluate(entries: &[corpus::Entry], w: &Weights) -> (usize, f64, Vec<String>) {
    let mut pass = 0;
    let mut margin = 0.0;
    let mut misses = Vec::new();
    for e in entries.iter().filter(|e| e.disputed.is_none()) {
        let all = readings(&e.notes, Spelling::Flats, w);
        let ok = |s: &str| s == e.expected || e.alternates.iter().any(|a| a == s);
        if ok(&all[0].symbol) {
            pass += 1;
            let rival = all.iter().find(|r| !ok(&r.symbol)).map_or(0.0, |r| r.score);
            margin += (all[0].score - rival).min(2.0);
        } else {
            misses.push(format!(
                "{} → {} (want {})",
                e.text, all[0].symbol, e.expected
            ));
        }
    }
    (pass, margin, misses)
}

fn main() {
    let entries = corpus::parse(include_str!("../tests/corpus.txt")).unwrap();
    let total = entries.iter().filter(|e| e.disputed.is_none()).count();
    let (p, m, misses) = evaluate(&entries, &WEIGHTS);
    println!("current: {p}/{total} (margin {m:.1})");
    for x in &misses {
        println!("  miss {x}");
    }
    let mut results: Vec<(usize, f64, Weights)> = Vec::new();
    for bass in [1.5, 2.0, 2.5, 3.0, 3.5] {
        for parncutt in [0.0, 1.0, 2.0, 3.0] {
            for hindemith in [0.0, 0.5, 1.0] {
                for dominant in [0.0, 0.3, 0.6, 1.0] {
                    for dominant_alteration in [0.3, 0.5] {
                        for incomplete_added in [1.5, 2.0, 2.5] {
                            for full_triad in [0.6, 1.0] {
                                for triad_alteration in [0.9, 1.2, 1.5] {
                                    for foreign_bass in [0.0, 0.5, 1.0] {
                                        let w = Weights {
                                            bass,
                                            parncutt,
                                            hindemith,
                                            dominant,
                                            dominant_alteration,
                                            incomplete_added,
                                            full_triad,
                                            triad_alteration,
                                            foreign_bass,
                                            temperature: WEIGHTS.temperature,
                                        };
                                        let (p, m, _) = evaluate(&entries, &w);
                                        results.push((p, m, w));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    results.sort_by(|a, b| b.0.cmp(&a.0).then(b.1.total_cmp(&a.1)));
    for (p, m, w) in results.iter().take(8) {
        println!("{p}/{total} margin {m:.1}: {w:?}");
    }
    let (_, _, best) = results[0];
    for x in evaluate(&entries, &best).2 {
        println!("  best still misses {x}");
    }
}
