use anatomy::waveform::{marker_label, markers, path, sine, span, sum};
use anatomy::Tuning;
use dioxus::prelude::*;

use super::{SectionHead, Segmented};
use crate::state::use_app;

const LEFT: f64 = 120.0;
const LANE: f64 = 28.0;
const SUM_H: f64 = 160.0;
const PAD: f64 = 22.0;
const STEP: f64 = 0.5;

#[component]
pub fn Waveform() -> Element {
    let mut app = use_app();
    let mut width = use_signal(|| 1200.0f64);
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let tuning = app.tuning.cloned();
    let periods = app.window_periods.cloned();
    let p = &a.periodicity;
    let fr = a.freqs(tuning).to_vec();
    let n = a.notes.len();
    let w = width().max(LEFT + 100.0);
    let h = n as f64 * LANE + SUM_H + PAD * 2.0 + 10.0;
    let pw = w - LEFT;
    let (span_s, capped) = span(p.period, periods);
    let x_of = |t: f64| LEFT + t / span_s * pw;

    let lanes: Vec<(f64, String, String)> = (0..n)
        .map(|i| {
            let mid = PAD + (n - 1 - i) as f64 * LANE + LANE / 2.0;
            let label = format!("{}  {:.1} Hz", a.note_names[i], fr[i]);
            (
                mid,
                label,
                path(LEFT, pw, STEP, span_s, mid, LANE * 0.36, sine(fr[i])),
            )
        })
        .collect();
    let smid = PAD + n as f64 * LANE + 10.0 + SUM_H / 2.0;
    let sum_d = path(LEFT, pw, STEP, span_s, smid, SUM_H * 0.46, sum(&fr));
    let marks: Vec<(f64, String, f64)> = markers(p.period, span_s)
        .into_iter()
        .enumerate()
        .map(|(k, t)| {
            let x = x_of(t).round() + 0.5;
            (x, marker_label(k, t), (x + 4.0).min(w - 110.0))
        })
        .collect();

    // In 12-TET the period and fundamental belong to the just chord it approximates.
    let approx = if tuning == Tuning::Et { "≈ " } else { "" };
    let ms = p.period * 1000.0;
    let period = if ms < 10.0 {
        format!("{approx}{ms:.2} ms")
    } else {
        format!("{approx}{ms:.1} ms")
    };
    let plain = |i: usize| a.note_names[i].clone();
    let fastest = a.fastest_beat().map(|b| {
        (
            b.rate,
            plain(b.i),
            plain(b.j),
            b.lower_partial,
            b.upper_partial,
        )
    });
    let mut stats = vec![
        ("Harmonic ratio (just)", p.harmonics_text(" : ")),
        (
            "Common fundamental",
            format!("{approx}{:.2} Hz", p.fundamental),
        ),
        ("Period T", period),
        (
            "Bass cycles per T",
            format!("{} × {}", p.bass_cycles, a.note_names[0]),
        ),
    ];
    if tuning == Tuning::Et {
        let v = fastest
            .as_ref()
            .map_or("none".to_string(), |(r, lo, hi, _, _)| {
                format!("{r:.1} Hz · {lo}–{hi}")
            });
        stats.push(("Fastest beat", v));
    }
    let mut caption = match (tuning, &fastest) {
        (Tuning::Just, _) => "Just intonation: every note is an integer multiple of the common fundamental, so the sum repeats exactly at each dashed line.".to_string(),
        (Tuning::Et, Some((r, lo, hi, pl, pu))) => format!(
            "12-TET: ratios are irrational, so the sum never repeats exactly. Coinciding partials beat instead: fastest is {lo}–{hi} at {r:.1} Hz (partial {pl} of {lo} against partial {pu} of {hi}). Dashed lines mark the just period it approximates; the drift is the cents column in 03."
        ),
        (Tuning::Et, None) => "12-TET: ratios are irrational, so the sum never repeats exactly. Dashed lines mark the just period it approximates; the drift is the cents column in 03.".to_string(),
    };
    if capped {
        caption += " View capped at 250 ms.";
    }
    rsx! {
        section { class: "wave-section",
            SectionHead { num: "08", title: "Waveform" }
            div { class: "wave-top",
                div { class: "stats",
                    for (k, v) in stats {
                        div { class: "stat",
                            span { class: "k", "{k}" }
                            span { class: "v", "{v}" }
                        }
                    }
                }
                div { class: "toggles",
                    Segmented {
                        options: vec![(Tuning::Just, "Just".to_string()), (Tuning::Et, "12-TET".to_string())],
                        value: tuning,
                        onchange: move |t| app.tuning.set(t),
                    }
                    Segmented {
                        options: vec![(1u32, "1T".to_string()), (2, "2T".to_string()), (4, "4T".to_string())],
                        value: periods,
                        onchange: move |k| app.window_periods.set(k),
                    }
                }
            }
            div {
                onresize: move |e| {
                    if let Ok(size) = e.get_content_box_size() {
                        if size.width > 0.0 && (size.width - width()).abs() >= 1.0 {
                            width.set(size.width);
                        }
                    }
                },
                svg {
                    class: "plot",
                    view_box: "0 0 {w} {h}",
                    height: "{h}",
                    for (x, label, lx) in marks {
                        line { class: "marker", x1: "{x}", x2: "{x}", y1: "{PAD - 6.0}", y2: "{h - PAD}" }
                        text { class: "mlabel", x: "{lx}", y: "{h - 8.0}", "{label}" }
                    }
                    for (mid, label, d) in lanes {
                        line { class: "base", x1: "{LEFT}", x2: "{w}", y1: "{mid}", y2: "{mid}" }
                        path { class: "lane", d: "{d}" }
                        text { class: "label", x: "8", y: "{mid + 3.5}", "{label}" }
                    }
                    line { class: "sum-base", x1: "{LEFT}", x2: "{w}", y1: "{smid}", y2: "{smid}" }
                    path { class: "sum", d: "{sum_d}" }
                    text { class: "label", x: "8", y: "{smid + 3.5}", "sum" }
                }
            }
            p { class: "wave-caption", "{caption}" }
        }
    }
}
