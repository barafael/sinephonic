use anatomy::{interval_name, Timbre};
use dioxus::prelude::*;

use super::{SectionHead, Segmented};
use crate::state::use_app;

struct Cell {
    /// Notes played on click.
    play: Vec<usize>,
    top: String,
    sub: String,
    bg: String,
    fg: &'static str,
    size: &'static str,
}

impl Cell {
    fn plain(top: String, bg: &str, fg: &'static str, size: &'static str) -> Self {
        Self {
            play: Vec::new(),
            top,
            sub: String::new(),
            bg: bg.to_string(),
            fg,
            size,
        }
    }
}

#[component]
pub fn InterRelations() -> Element {
    let mut app = use_app();
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let set = app.settings.read().ratio_set;
    let n = a.notes.len();
    let mut cells = vec![Cell::plain(
        String::new(),
        "transparent",
        "var(--ink)",
        "13px",
    )];
    for (i, name) in a.note_names.iter().enumerate() {
        let mut c = Cell::plain(name.clone(), "transparent", "var(--ink)", "13px");
        c.play = vec![i];
        cells.push(c);
    }
    for i in 0..n {
        let mut head = Cell::plain(a.note_names[i].clone(), "var(--sel)", "var(--ink)", "13px");
        head.play = vec![i];
        cells.push(head);
        for j in 0..n {
            cells.push(if i == j {
                Cell::plain("·".into(), "var(--diag)", "var(--muted-2)", "13px")
            } else if j > i {
                let p = a.pair(i, j).expect("pair exists");
                let r = p.roughness;
                Cell {
                    play: vec![i, j],
                    top: interval_name(p.semitones),
                    sub: format!("r {r:.2}"),
                    // The colour ramp lives in CSS (.cell.rough) so it can follow the theme.
                    bg: format!("--r:{r:.3}"),
                    fg: if r > 0.55 { "hot" } else { "" },
                    size: "15px",
                }
            } else {
                let mut c = Cell::plain(
                    set.ratio((a.notes[i] - a.notes[j]) as u32).to_string(),
                    "transparent",
                    "var(--muted)",
                    "12px",
                );
                c.play = vec![j, i];
                if let Some(b) = a
                    .beats
                    .iter()
                    .find(|b| (b.i, b.j) == (j, i) && b.rate > 0.05)
                {
                    c.sub = format!("beats {:.1} Hz", b.rate);
                }
                c
            });
        }
    }
    rsx! {
        section { class: "section",
            SectionHead { num: "05", title: "Inter-relations" }
            div { class: "caption-row",
                span { class: "muted",
                    if app.tuning.cloned() == anatomy::Tuning::Et {
                        "Upper: interval + roughness · Lower: just ratio + 12-TET beating"
                    } else {
                        "Upper: interval + roughness · Lower: just ratio"
                    }
                }
                div { class: "toggle-label",
                    span { class: "muted", "tone model" }
                    Segmented {
                        options: vec![(Timbre::Sine, "sine".to_string()), (Timbre::Harmonic6, "6 partials".to_string())],
                        value: app.timbre.cloned(),
                        onchange: move |t| app.timbre.set(t),
                    }
                }
            }
            div { class: "scroll-x",
                div { class: "matrix", style: "grid-template-columns:56px repeat({n},minmax(0,96px))",
                    // Keyed by size and position: a cell's kind (header, rough, ratio) depends
                    // only on those, so a new size gets fresh elements instead of patched styles.
                    for (k, c) in cells.into_iter().enumerate() {
                        div {
                            key: "{n}-{k}",
                            class: if c.bg.starts_with("--r") { format!("cell playable rough {}", c.fg) } else if c.play.is_empty() { "cell".to_string() } else { "cell playable".to_string() },
                            // Every cell sets both properties so no stale inline style survives a re-render.
                            style: if c.bg.starts_with("--r") { format!("{};background:var(--ramp);color:var(--ramp-fg)", c.bg) } else { format!("background:{};color:{}", c.bg, c.fg) },
                            onclick: {
                                let idx = c.play.clone();
                                move |_| {
                                    if !idx.is_empty() {
                                        crate::audio::subset(app, &idx);
                                    }
                                }
                            },
                            span { class: "t", style: "font-size:{c.size}", "{c.top}" }
                            span { class: "s", "{c.sub}" }
                        }
                    }
                }
            }
            div { class: "legend",
                span { "smooth" }
                div { class: "ramp" }
                span { "rough" }
                span { class: "src", "Sethares/Plomp–Levelt, relative to C4–D♭4" }
            }
        }
    }
}
