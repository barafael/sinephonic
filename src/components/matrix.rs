use anatomy::explore::dissonance_curve;
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
    // Dissonance curve over the bass, with the voicing's intervals marked.
    let timbre = app.timbre.cloned();
    let fr = a.freqs(app.tuning.cloned());
    let span = 1200.0 * (fr[n - 1] / fr[0]).log2();
    let max_c = if span <= 1250.0 {
        1250.0
    } else {
        (span + 100.0).min(3700.0)
    };
    let curve = dissonance_curve(fr[0], timbre, app.settings.read().a4, max_c, 600);
    let peak = curve.iter().map(|c| c.1).fold(0.0, f64::max).max(0.4);
    let (cw, ch, top, bottom) = (600.0, 150.0, 10.0, 22.0);
    let cx = |c: f64| 6.0 + c / max_c * (cw - 12.0);
    let cy = |r: f64| top + (1.0 - (r / peak).min(1.0)) * (ch - top - bottom);
    let curve_d: String = curve
        .iter()
        .enumerate()
        .map(|(k, &(c, r))| {
            format!(
                "{}{:.1} {:.1}",
                if k == 0 { 'M' } else { 'L' },
                cx(c),
                cy(r)
            )
        })
        .collect();
    let marks: Vec<(f64, f64, String)> = (1..n)
        .map(|i| {
            let c = 1200.0 * (fr[i] / fr[0]).log2();
            let k = ((c / max_c) * 600.0).round() as usize;
            (
                cx(c),
                cy(curve[k.min(600)].1),
                interval_name(a.notes[i] - a.notes[0]),
            )
        })
        .collect();
    let octaves: Vec<f64> = (1..=(max_c / 1200.0) as usize)
        .map(|o| cx(o as f64 * 1200.0))
        .collect();
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
            div { class: "curve-head",
                span { class: "muted", "Dissonance curve over {a.note_names[0]}" }
                span { class: "muted", "dots: this voicing" }
            }
            svg { class: "curve", view_box: "0 0 {cw} {ch}",
                for x in octaves {
                    line { class: "grid", x1: "{x}", x2: "{x}", y1: "{top}", y2: "{ch - bottom}" }
                }
                line { class: "base", x1: "0", x2: "{cw}", y1: "{ch - bottom}", y2: "{ch - bottom}" }
                path { class: "curve-line", d: "{curve_d}" }
                for (x, y, label) in marks {
                    line { class: "marker", x1: "{x}", x2: "{x}", y1: "{y}", y2: "{ch - bottom}" }
                    circle { class: "curve-dot", cx: "{x}", cy: "{y}", r: "3.5" }
                    text { class: "mlabel", x: "{x}", y: "{ch - 6.0}", text_anchor: "middle", "{label}" }
                }
            }
        }
    }
}
