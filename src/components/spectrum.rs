use anatomy::explore::spectrum;
use anatomy::Tuning;
use dioxus::prelude::*;

use super::SectionHead;
use crate::state::use_app;

const LEFT: f64 = 120.0;
const LANE: f64 = 56.0;
const PAD: f64 = 30.0;
const AXIS: f64 = 30.0;

/// Every partial on a log-frequency axis, one lane per note, with the partials that meet on the
/// chord's harmonic series joined; in 12-TET the joins are labelled with their beat rate.
#[component]
pub fn Spectrum() -> Element {
    let app = use_app();
    let a4 = app.settings.read().a4;
    let mut width = use_signal(|| 1200.0f64);
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let tuning = app.tuning.cloned();
    let timbre = app.timbre.cloned();
    let (parts, hits) = spectrum(a, tuning, timbre);
    let n = a.notes.len();
    let w = width().max(LEFT + 200.0);
    let h = PAD + n as f64 * LANE + AXIS;
    let lo = parts.iter().map(|p| p.freq).fold(f64::INFINITY, f64::min) / 1.06;
    let hi = parts.iter().map(|p| p.freq).fold(0.0, f64::max) * 1.06;
    let x_of = |f: f64| LEFT + (f / lo).log2() / (hi / lo).log2() * (w - LEFT - 12.0);
    let base = |note: usize| PAD + (n - 1 - note) as f64 * LANE + LANE - 6.0;
    let ticks: Vec<(f64, f64, f64)> = parts
        .iter()
        .map(|p| {
            (
                x_of(p.freq),
                base(p.note),
                base(p.note) - p.amp * (LANE - 10.0),
            )
        })
        .collect();
    let joins: Vec<(f64, f64, f64, String)> = hits
        .iter()
        .map(|c| {
            let x = x_of((c.lower.freq + c.upper.freq) / 2.0);
            let label = if tuning == Tuning::Et && c.beat > 0.05 {
                format!("{:.1} Hz", c.beat)
            } else {
                format!("h{}", c.harmonic())
            };
            (
                x,
                base(c.upper.note) - LANE + 8.0,
                base(c.lower.note),
                label,
            )
        })
        .collect();
    // C octave gridlines.
    let grid: Vec<(f64, String)> = (0..10)
        .map(|o| (harmony::Midi(12 * (o + 1)).freq(a4), format!("C{o}")))
        .filter(|(f, _)| *f >= lo && *f <= hi)
        .map(|(f, l)| (x_of(f), l))
        .collect();
    let labels: Vec<(f64, String)> = (0..n)
        .map(|i| (base(i) - 4.0, a.note_names[i].clone()))
        .collect();
    let beating = hits.iter().filter(|c| c.beat > 0.05).count();
    let caption = match tuning {
        Tuning::Just => format!(
            "Just intonation: {} pairs of partials land exactly on shared harmonics of the {:.2} Hz fundamental (labelled with the harmonic number), so they fuse instead of beating.",
            hits.len(),
            a.periodicity.fundamental
        ),
        Tuning::Et => format!(
            "12-TET: of {} partial pairs that would coincide in just intonation, {beating} are pulled apart and beat at the rates shown. Slow beats read as warmth or chorus; beats around 15–40 Hz as roughness.",
            hits.len()
        ),
    };
    rsx! {
        section { class: "wave-section",
            SectionHead { num: "09", title: "Spectrum", hint: "partials on the harmonic series".to_string() }
            div {
                onresize: move |e| {
                    if let Ok(size) = e.get_content_box_size() {
                        if size.width > 0.0 && (size.width - width()).abs() >= 1.0 {
                            width.set(size.width);
                        }
                    }
                },
                svg { class: "plot", view_box: "0 0 {w} {h}", height: "{h}",
                    for (x, l) in grid {
                        line { class: "grid", x1: "{x}", x2: "{x}", y1: "{PAD - 10.0}", y2: "{h - AXIS + 4.0}" }
                        text { class: "mlabel", x: "{x + 3.0}", y: "{h - 10.0}", "{l}" }
                    }
                    for i in 0..n {
                        line { class: "base", x1: "{LEFT}", x2: "{w}", y1: "{base(i)}", y2: "{base(i)}" }
                    }
                    for (x, y1, y2, label) in joins {
                        line { class: "join", x1: "{x}", x2: "{x}", y1: "{y1}", y2: "{y2}" }
                        text { class: "jlabel", x: "{x + 3.0}", y: "{y1 + 2.0}", "{label}" }
                    }
                    for (x, y1, y2) in ticks {
                        line { class: "partial", x1: "{x}", x2: "{x}", y1: "{y1}", y2: "{y2}" }
                    }
                    for (y, l) in labels {
                        text { class: "label", x: "8", y: "{y}", "{l}" }
                    }
                }
            }
            p { class: "wave-caption", "{caption}" }
        }
    }
}
