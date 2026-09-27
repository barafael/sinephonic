use anatomy::explore::dissonance_curve;
use anatomy::information::{harmonic_entropy_curve, harmonic_entropy_norm};
use anatomy::interval_name;
use dioxus::prelude::*;

use super::SectionHead;
use crate::state::use_app;

const LEFT: f64 = 16.0;
const RIGHT: f64 = 16.0;
const TOP: f64 = 34.0;
const BOTTOM: f64 = 46.0;
const HEIGHT: f64 = 320.0;
const STEPS: usize = 900;

/// Just ratios labelled along the axis, where both curves dip.
const LANDMARKS: [(&str, f64); 11] = [
    ("16/15", 111.73),
    ("9/8", 203.91),
    ("6/5", 315.64),
    ("5/4", 386.31),
    ("4/3", 498.04),
    ("7/5", 582.51),
    ("3/2", 701.96),
    ("8/5", 813.69),
    ("5/3", 884.36),
    ("7/4", 968.83),
    ("15/8", 1088.27),
];

/// Sensory roughness and harmonic entropy of every interval above the bass, with the
/// voicing's intervals placed on both curves.
#[component]
pub fn DissonanceCurve() -> Element {
    let app = use_app();
    let mut width = use_signal(|| 1200.0f64);
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let n = a.notes.len();
    let fr = a.freqs(app.tuning.cloned());
    let span = 1200.0 * (fr[n - 1] / fr[0]).log2();
    let max_c = if span <= 1250.0 {
        1250.0
    } else {
        (span + 100.0).min(3700.0)
    };
    let w = width().max(400.0);
    let plot_h = HEIGHT - TOP - BOTTOM;
    let curve = dissonance_curve(
        fr[0],
        app.timbre.cloned(),
        app.settings.read().a4,
        max_c,
        STEPS,
    );
    let peak = curve.iter().map(|c| c.1).fold(0.0, f64::max).max(0.4);
    let cx = |c: f64| LEFT + c / max_c * (w - LEFT - RIGHT);
    let cy = |v: f64| TOP + (1.0 - v.clamp(0.0, 1.0)) * plot_h;
    let path = |pts: &[(f64, f64)], scale: f64| -> String {
        pts.iter()
            .enumerate()
            .map(|(k, &(c, v))| {
                format!(
                    "{}{:.1} {:.1}",
                    if k == 0 { 'M' } else { 'L' },
                    cx(c),
                    cy(v / scale)
                )
            })
            .collect()
    };
    let rough_d = path(&curve, peak);
    let rough_fill = format!(
        "{rough_d}L{:.1} {:.1}L{:.1} {:.1}Z",
        cx(max_c),
        cy(0.0),
        cx(0.0),
        cy(0.0)
    );
    let he_d = path(&harmonic_entropy_curve(max_c, STEPS), 1.0);
    let semis: Vec<(f64, bool)> = (1..=(max_c / 100.0) as usize)
        .map(|k| (cx(k as f64 * 100.0), k % 12 == 0))
        .collect();
    let landmarks: Vec<(f64, &str)> = (0..=(max_c / 1200.0) as usize)
        .flat_map(|o| {
            LANDMARKS
                .iter()
                .map(move |&(l, c)| (c + 1200.0 * o as f64, l))
        })
        .filter(|&(c, _)| c < max_c)
        .map(|(c, l)| (cx(c), l))
        .collect();
    let octave_labels: Vec<(f64, String)> = (1..=(max_c / 1200.0) as usize)
        .map(|o| {
            (
                cx(1200.0 * o as f64),
                if o == 1 {
                    "P8".into()
                } else {
                    format!("{o} oct")
                },
            )
        })
        .collect();
    let marks: Vec<(f64, f64, f64, String, String)> = (1..n)
        .map(|i| {
            let c = 1200.0 * (fr[i] / fr[0]).log2();
            let k = ((c / max_c) * STEPS as f64).round() as usize;
            let r = curve[k.min(STEPS)].1;
            let he = harmonic_entropy_norm(c);
            (
                cx(c),
                cy(r / peak),
                cy(he),
                format!(
                    "{} {}",
                    a.note_names[i],
                    interval_name(a.notes[i] - a.notes[0])
                ),
                format!("r {:.2} · HE {:.2}", r, he),
            )
        })
        .collect();
    rsx! {
        section { class: "wave-section",
            SectionHead { num: "07", title: "Consonance landscape", hint: format!("every interval above {}", a.note_names[0]) }
            div {
                onresize: move |e| {
                    if let Ok(size) = e.get_content_box_size() {
                        if size.width > 0.0 && (size.width - width()).abs() >= 1.0 {
                            width.set(size.width);
                        }
                    }
                },
                svg { class: "plot curve-plot", view_box: "0 0 {w} {HEIGHT}", height: "{HEIGHT}",
                    for (x, octave) in semis {
                        line { class: if octave { "grid strong" } else { "grid" }, x1: "{x}", x2: "{x}", y1: "{TOP}", y2: "{HEIGHT - BOTTOM}" }
                    }
                    path { class: "rough-fill", d: "{rough_fill}" }
                    path { class: "he-line", d: "{he_d}" }
                    path { class: "curve-line", d: "{rough_d}" }
                    line { class: "base", x1: "{LEFT}", x2: "{w - RIGHT}", y1: "{HEIGHT - BOTTOM}", y2: "{HEIGHT - BOTTOM}" }
                    for (x, l) in landmarks {
                        text { class: "mlabel", x: "{x}", y: "{HEIGHT - BOTTOM + 14.0}", text_anchor: "middle", "{l}" }
                    }
                    for (x, l) in octave_labels {
                        text { class: "label", x: "{x}", y: "{HEIGHT - BOTTOM + 28.0}", text_anchor: "middle", "{l}" }
                    }
                    for (x, yr, yh, name, values) in marks {
                        line { class: "marker", x1: "{x}", x2: "{x}", y1: "{TOP - 6.0}", y2: "{HEIGHT - BOTTOM}" }
                        circle { class: "he-dot", cx: "{x}", cy: "{yh}", r: "3.5" }
                        circle { class: "curve-dot", cx: "{x}", cy: "{yr}", r: "5" }
                        // Labels near the right edge hang to the left of their marker.
                        if x > w - 170.0 {
                            text { class: "label", x: "{x - 5.0}", y: "{TOP - 18.0}", text_anchor: "end", "{name}" }
                            text { class: "mlabel", x: "{x - 5.0}", y: "{TOP - 6.0}", text_anchor: "end", "{values}" }
                        } else {
                            text { class: "label", x: "{x + 5.0}", y: "{TOP - 18.0}", "{name}" }
                            text { class: "mlabel", x: "{x + 5.0}", y: "{TOP - 6.0}", "{values}" }
                        }
                    }
                }
            }
            div { class: "legend",
                span { class: "swatch solid" }
                span { "sensory roughness (Sethares, excess over unison, current tone model and bass register)" }
                span { class: "swatch dashed" }
                span { "harmonic entropy (Erlich, register-free)" }
            }
            p { class: "wave-caption",
                "Both curves dip at simple ratios. Roughness comes from beating partials and grows in low registers; harmonic entropy measures how clearly an interval implies one ratio. The voicing's notes sit where the dots are: labels show the interval above the bass, its roughness r and its normalised harmonic entropy."
            }
        }
    }
}
