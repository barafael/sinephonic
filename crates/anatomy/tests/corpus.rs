//! Every non-disputed corpus entry must get its expected (or an acceptable) best reading and
//! tags from the improved model.

use anatomy::corpus::{self, Verdict};
use anatomy::Options;

#[test]
fn corpus() {
    let entries = corpus::parse(include_str!("corpus.txt")).unwrap();
    assert!(entries.len() > 100, "{}", entries.len());
    let mut failures = Vec::new();
    for (e, o, a) in corpus::run(&entries, &Options::default()) {
        if e.disputed.is_some() || o.passed() {
            continue;
        }
        let rank = match o.verdict {
            Verdict::Miss {
                expected_rank: Some(k),
            } => format!("expected at #{}", k + 1),
            Verdict::Miss {
                expected_rank: None,
            } => "expected not among readings".into(),
            _ => String::new(),
        };
        let top: Vec<String> = a
            .readings
            .iter()
            .take(3)
            .map(|r| format!("{} {:.2}", r.symbol, r.score))
            .collect();
        failures.push(format!(
            "line {}: {} → got {}, want {} ({rank}); missing tags {:?}; top: {}",
            e.line,
            e.text,
            o.best,
            e.expected,
            o.missing_tags,
            top.join(" | ")
        ));
    }
    assert!(
        failures.is_empty(),
        "{} corpus failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
