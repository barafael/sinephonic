//! `anatomy`: analyse chord voicings in the terminal.
//!
//! ```text
//! anatomy C3 G3 Ab3 Eb4            full report (improved model)
//! anatomy --compare "C E G Bb D#"  prototype and improved side by side
//! anatomy corpus [FILE]            judge every corpus entry
//! ```

use std::process::ExitCode;

use anatomy::corpus::{self, Verdict};
use anatomy::{analyze, interval_name, Analysis, Model, Options, Timbre, Tuning};
use harmony::{RatioSet, Spelling};

const USAGE: &str = "\
usage: anatomy [OPTIONS] NOTES...
       anatomy corpus [FILE] [OPTIONS]

Notes are text like \"C3 G3 Ab3 Eb4\" or \"C G Ab Eb\" (octave 3 upward), one or many
arguments.

options:
  --proto         use the prototype model (default: improved)
  --compare       show prototype and improved side by side
  --all           list all 12 readings
  --sharps        spell black keys with sharps by default
  --ratio R       ratio set: 5, 7 or s (Stolzenburg; improved default)
  --just          just frequencies for roughness and beats (default: 12-TET)
  --sine          sine tones for roughness (default: 6 partials)
  --a4 HZ         reference pitch (default 440)
  --failures      corpus: print only entries that miss";

struct Args {
    opts: Options,
    compare: bool,
    all: bool,
    failures: bool,
    corpus: Option<Option<String>>,
    notes: String,
}

fn parse_args() -> Result<Args, String> {
    let mut opts = Options::default();
    let mut ratio: Option<RatioSet> = None;
    let mut a = Args {
        opts,
        compare: false,
        all: false,
        failures: false,
        corpus: None,
        notes: String::new(),
    };
    let mut words: Vec<String> = Vec::new();
    let mut it = std::env::args().skip(1);
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "-h" | "--help" => return Err(String::new()),
            "--proto" => opts.model = Model::Prototype,
            "--compare" => a.compare = true,
            "--all" => a.all = true,
            "--failures" => a.failures = true,
            "--sharps" => opts.spelling = Spelling::Sharps,
            "--et" => opts.tuning = Tuning::Et,
            "--just" => opts.tuning = Tuning::Just,
            "--sine" => opts.timbre = Timbre::Sine,
            "--ratio" => {
                ratio = Some(match it.next().as_deref() {
                    Some("5") => RatioSet::FiveLimit,
                    Some("7") => RatioSet::SevenLimit,
                    Some("s") => RatioSet::Stolzenburg,
                    other => return Err(format!("--ratio takes 5, 7 or s, not {other:?}")),
                })
            }
            "--a4" => {
                opts.a4 = it
                    .next()
                    .and_then(|s| s.parse().ok())
                    .ok_or("--a4 takes a number")?
            }
            _ => words.push(arg),
        }
    }
    opts.ratio_set = ratio.unwrap_or(match opts.model {
        Model::Prototype => RatioSet::FiveLimit,
        Model::Improved => RatioSet::Stolzenburg,
    });
    a.opts = opts;
    if words.first().map(String::as_str) == Some("corpus") {
        a.corpus = Some(words.get(1).cloned());
    } else {
        a.notes = words.join(" ");
    }
    Ok(a)
}

fn main() -> ExitCode {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("{e}\n");
            }
            eprintln!("{USAGE}");
            return ExitCode::FAILURE;
        }
    };
    if let Some(file) = &args.corpus {
        return run_corpus(file.as_deref(), &args);
    }
    let notes = harmony::midi::parse_notes(&args.notes);
    if notes.len() < 2 {
        eprintln!("need two or more notes\n\n{USAGE}");
        return ExitCode::FAILURE;
    }
    if args.compare {
        compare(&notes, &args);
    } else {
        let a = analyze(&notes, &args.opts).expect("two notes");
        report(&a, &args);
    }
    ExitCode::SUCCESS
}

fn bar(v: f64, width: usize) -> String {
    let n = (v.clamp(0.0, 1.0) * width as f64).round() as usize;
    format!("{}{}", "█".repeat(n), "·".repeat(width - n))
}

fn pad(s: &str, w: usize) -> String {
    let n = s.chars().count();
    if n >= w {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(w - n))
    }
}

fn report(a: &Analysis, args: &Args) {
    let opts = &args.opts;
    let best = a.best();
    println!();
    println!("  {}    {}", best.symbol, a.note_names.join(" – "));
    println!("  {}", a.summary());
    println!(
        "  model: {:?} · ratios: {} · {:?} · {:?} · A4 = {}",
        opts.model,
        opts.ratio_set.label(),
        opts.tuning,
        opts.timbre,
        opts.a4
    );

    println!("\n  READINGS");
    let top = a.readings[0].prob;
    let k = if args.all { 12 } else { 5 };
    for r in a.readings.iter().take(k) {
        let tones: Vec<String> = a
            .pcs
            .iter()
            .map(|&pc| {
                format!(
                    "{} {}",
                    r.tone_name(pc),
                    r.degree_of(pc).map_or("?", |d| d.label())
                )
            })
            .collect();
        let harte = if r.harte.is_empty() {
            String::new()
        } else {
            format!("[{}]", r.harte)
        };
        println!(
            "  {} {} {:>3.0}% {:>6.2}  {}  {}",
            pad(&r.symbol, 22),
            bar(r.prob / top, 10),
            r.prob * 100.0,
            r.score,
            pad(&harte, 20),
            tones.join(" · ")
        );
        println!(
            "  {}{}",
            " ".repeat(24),
            r.bass_relation(
                a.bass_pc,
                a.note_names[0].trim_end_matches(|c: char| c.is_ascii_digit() || c == '-')
            )
        );
    }

    println!("\n  INTERVALS (functions as {})", best.symbol);
    println!(
        "  {}{}{}{}{}Hz",
        pad("note", 8),
        pad("degree", 8),
        pad("above bass", 18),
        pad("just", 9),
        pad("TET−just", 10)
    );
    let fr = a.freqs(opts.tuning);
    let p = &a.periodicity;
    for i in (0..a.notes.len()).rev() {
        let d = a.notes[i] - a.notes[0];
        let above = if i == 0 {
            "bass".to_string()
        } else {
            format!("{} · {d} st", interval_name(d))
        };
        let deg = best
            .degree_of(harmony::PitchClass::of_midi(a.notes[i]))
            .map_or("?", |d| d.label());
        println!(
            "  {}{}{}{}{}{:.1}",
            pad(&a.note_names[i], 8),
            pad(deg, 8),
            pad(&above, 18),
            pad(&p.ratios[i].to_string(), 9),
            pad(&format!("{:+.1}¢", p.cents[i]), 10),
            fr[i]
        );
    }

    println!("\n  PAIRS (upper: interval, roughness · lower: just ratio)");
    let n = a.notes.len();
    let mut header = pad("", 7);
    for name in &a.note_names {
        header += &pad(name, 11);
    }
    println!("  {header}");
    for i in 0..n {
        let mut row = pad(&a.note_names[i], 7);
        for j in 0..n {
            let cell = if i == j {
                "·".to_string()
            } else if j > i {
                let pr = a.pair(i, j).expect("pair");
                format!("{} r{:.2}", interval_name(pr.semitones), pr.roughness)
            } else {
                opts.ratio_set
                    .ratio((a.notes[i] - a.notes[j]) as u32)
                    .to_string()
            };
            row += &pad(&cell, 11);
        }
        println!("  {row}");
    }

    println!("\n  CHARACTER");
    for t in a.tags.iter().take(5) {
        println!("  {} {}", pad(t.word, 12), t.why);
    }
    println!();
    for (label, v) in a.axes.labelled() {
        println!("  {} {} {v:.2}", pad(label, 12), bar(v, 20));
    }
    println!(
        "  {} valence {:.2} · arousal {:.2}",
        pad("", 12),
        a.valence,
        a.arousal
    );

    println!("\n  WAVEFORM");
    let ms = p.period * 1000.0;
    println!(
        "  harmonics {} · fundamental {:.2} Hz · period {} ms · {} × {}",
        p.harmonics_text(" : "),
        p.fundamental,
        if ms < 10.0 {
            format!("{ms:.2}")
        } else {
            format!("{ms:.1}")
        },
        p.bass_cycles,
        a.note_names[0]
    );

    let audible: Vec<String> = a
        .beats
        .iter()
        .filter(|b| b.rate > 0.05)
        .map(|b| {
            format!(
                "{}–{} {:.1} Hz ({}:{})",
                a.note_names[b.i], a.note_names[b.j], b.rate, b.lower_partial, b.upper_partial
            )
        })
        .collect();
    if !audible.is_empty() {
        println!("  beats: {}", audible.join(" · "));
    }

    if let Some(x) = &a.extras {
        println!("\n  SET THEORY & CONSONANCE");
        println!(
            "  set class {} · prime form {} · interval vector ⟨{}⟩{}",
            x.forte,
            x.prime_form,
            x.interval_vector
                .iter()
                .map(u8::to_string)
                .collect::<Vec<_>>()
                .join(""),
            x.sonority.map(|s| format!(" · {s}")).unwrap_or_default()
        );
        println!(
            "  Parncutt root ambiguity {:.2} · Stolzenburg smoothed log₂ periodicity {:.2} · Huron consonance {:+.2}",
            x.root_ambiguity, x.smoothed_periodicity, x.huron
        );
        println!(
            "  Tenney height {:.2} · Euler gradus {} · Cook–Fujisawa tension {:.3}, modality {:+.3}",
            x.tenney, x.gradus, x.cook_tension, x.cook_modality
        );
    }
    println!();
}

fn compare(notes: &[u8], args: &Args) {
    let proto = analyze(
        notes,
        &Options {
            model: Model::Prototype,
            ratio_set: RatioSet::FiveLimit,
            ..args.opts
        },
    )
    .unwrap();
    let imp = analyze(
        notes,
        &Options {
            model: Model::Improved,
            ..args.opts
        },
    )
    .unwrap();
    println!("\n  {}", imp.note_names.join(" – "));
    println!("  {}IMPROVED", pad("PROTOTYPE", 44));
    let k = if args.all { 12 } else { 6 };
    for i in 0..k {
        let p = &proto.readings[i];
        let q = &imp.readings[i];
        println!(
            "  {}{:>4.0}% {:>6.2}      {}{:>4.0}% {:>6.2}",
            pad(&p.symbol, 22),
            p.prob * 100.0,
            p.score,
            pad(&q.symbol, 22),
            q.prob * 100.0,
            q.score
        );
    }
    let words = |a: &Analysis| {
        a.tags
            .iter()
            .take(5)
            .map(|t| t.word)
            .collect::<Vec<_>>()
            .join(", ")
    };
    println!("  {}{}", pad(&words(&proto), 44), words(&imp));
    println!();
}

fn run_corpus(file: Option<&str>, args: &Args) -> ExitCode {
    let text = match file {
        Some(f) => match std::fs::read_to_string(f) {
            Ok(t) => t,
            Err(e) => {
                eprintln!("{f}: {e}");
                return ExitCode::FAILURE;
            }
        },
        None => include_str!("../../anatomy/tests/corpus.txt").to_string(),
    };
    let entries = match corpus::parse(&text) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let results = corpus::run(&entries, &args.opts);
    let mut section = String::new();
    let (mut pass, mut fail, mut disputed) = (0, 0, 0);
    for (e, o, a) in &results {
        let mark = match (&o.verdict, e.disputed.is_some(), o.missing_tags.is_empty()) {
            (_, true, _) => {
                disputed += 1;
                "?"
            }
            (Verdict::Match, _, true) => {
                pass += 1;
                "✓"
            }
            (Verdict::Alternate, _, true) => {
                pass += 1;
                "≈"
            }
            _ => {
                fail += 1;
                "✗"
            }
        };
        if args.failures && mark != "✗" {
            continue;
        }
        if e.section != section {
            section = e.section.clone();
            println!("\n  {}", section.to_uppercase());
        }
        let second = a.readings.get(1).map_or(String::new(), |r| {
            format!("{} {:.0}%", r.symbol, r.prob * 100.0)
        });
        let want = if o.best == e.expected {
            String::new()
        } else {
            format!("want {}", e.expected)
        };
        println!(
            "  {mark} {} {} {:>3.0}%  {}  {}",
            pad(&e.text, 24),
            pad(&o.best, 18),
            a.best().prob * 100.0,
            pad(&format!("(2nd {second})"), 26),
            want
        );
        if !o.missing_tags.is_empty() {
            println!("      missing tags: {}", o.missing_tags.join(", "));
        }
        if let Some(d) = &e.disputed {
            println!("      disputed: {d}");
        }
    }
    println!("\n  {pass} pass · {fail} fail · {disputed} disputed");
    if fail == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
