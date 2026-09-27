use anatomy::{interval_name, Timbre};
use dioxus::prelude::*;

use super::{SectionHead, Segmented};
use crate::state::use_app;

struct Cell {
    top: String,
    sub: String,
    bg: String,
    fg: &'static str,
    size: &'static str,
}

impl Cell {
    fn plain(top: String, bg: &str, fg: &'static str, size: &'static str) -> Self {
        Self {
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
    let mut cells = vec![Cell::plain(String::new(), "transparent", "#1d1b18", "13px")];
    for name in &a.note_names {
        cells.push(Cell::plain(name.clone(), "transparent", "#1d1b18", "13px"));
    }
    for i in 0..n {
        cells.push(Cell::plain(
            a.note_names[i].clone(),
            "#ebe6dc",
            "#1d1b18",
            "13px",
        ));
        for j in 0..n {
            cells.push(if i == j {
                Cell::plain("·".into(), "#ece8e0", "#9a9488", "13px")
            } else if j > i {
                let p = a.pair(i, j).expect("pair exists");
                let r = p.roughness;
                Cell {
                    top: interval_name(p.semitones),
                    sub: format!("r {r:.2}"),
                    bg: format!("oklch({:.3} {:.3} 35)", 0.97 - 0.35 * r, 0.02 + 0.12 * r),
                    fg: if r > 0.55 { "#fff" } else { "#1d1b18" },
                    size: "15px",
                }
            } else {
                let mut c = Cell::plain(
                    set.ratio((a.notes[i] - a.notes[j]) as u32).to_string(),
                    "transparent",
                    "#6b665e",
                    "12px",
                );
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
            SectionHead { num: "04", title: "Inter-relations" }
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
                    for c in cells {
                        div { class: "cell", style: "background:{c.bg};color:{c.fg}",
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
