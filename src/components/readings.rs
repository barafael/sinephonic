use anatomy::improved::metrics::parncutt_support;
use dioxus::prelude::*;
use harmony::{PcSet, PitchClass};

use super::{strip_octave, SectionHead};
use crate::state::use_app;

#[component]
pub fn Readings() -> Element {
    let mut app = use_app();
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let selected = app.selected_root.cloned().unwrap_or(a.best().root);
    let top = a.readings[0].prob;
    let spelling = app.settings.read().spelling;
    let set = PcSet::from_midi(&a.notes);
    let support = parncutt_support(set);
    let max = support.iter().copied().fold(0.0, f64::max).max(1e-9);
    let roots: Vec<(u8, f64, bool, bool)> = (0..12u8)
        .map(|r| {
            (
                r,
                support[r as usize] / max,
                set.has(r),
                r == selected.value(),
            )
        })
        .collect();
    let bass_name = strip_octave(&a.note_names[0]).to_string();
    rsx! {
        section { class: "section",
            SectionHead { num: "02", title: "Readings" }
            div { class: "readings",
                for r in a.readings.iter().take(5) {
                    div {
                        key: "{r.root.value()}",
                        class: if r.root == selected { "reading on" } else { "reading" },
                        onclick: {
                            let root = r.root;
                            move |_| {
                                app.selected_root.set(Some(root));
                                crate::audio::as_reading(app, root);
                            }
                        },
                        div { class: "reading-name", "{r.symbol}" }
                        div { class: "prob",
                            div { class: "prob-track",
                                div { class: "prob-fill", style: "width:{r.prob / top * 100.0:.1}%" }
                            }
                            span { class: "prob-pct", "{(r.prob * 100.0).round()}%" }
                        }
                        div { class: "tones",
                            for pc in a.pcs.iter().copied() {
                                span { class: "tone",
                                    span { class: "dot" }
                                    span { "{r.tone_name(pc)}" }
                                    span { class: "muted", "{r.degree_of(pc).map_or(\"\", |d| d.label())}" }
                                }
                            }
                            span { class: "sub", "{r.bass_relation(a.bass_pc, &bass_name)}" }
                        }
                    }
                }
            }
            div { class: "ps-chart",
                div { class: "ps-title", "Root support for every candidate root (Parncutt) · click to hear" }
                div { class: "bars bars-12",
                    for (r, v, present, chosen) in roots {
                        button {
                            class: if chosen { "bar chosen" } else if present { "bar present" } else { "bar" },
                            title: "hear the chord over this root",
                            onclick: move |_| {
                                let root = PitchClass::new(r as i32);
                                app.selected_root.set(Some(root));
                                crate::audio::as_reading(app, root);
                            },
                            span { class: "bar-track", span { class: "bar-fill", style: "height:{v * 100.0:.0}%" } }
                            span { class: "bar-label", "{spelling.pc_name(PitchClass::new(r as i32))}" }
                        }
                    }
                }
            }
            p { class: "footnote",
                if a.extras.is_some() {
                    "Each of the 12 pitch classes is tried as root. Fit rises when the root is present or in the bass, when a clear 3rd, 5th and 7th exist, with psychoacoustic root support (Parncutt) and when the root is the root of the voicing's strongest interval (Hindemith); it falls with alterations, a missing 3rd or a missing root. Root support adds Parncutt's weights for the tones above each candidate (unison 10, fifth 5, major 3rd 3, minor 7th 2, major 2nd 1). Click a reading to relabel the intervals and hear it over its root."
                } else {
                    "Each of the 12 pitch classes is tried as root. Fit rises when the root is present or in the bass and a clear 3rd, 5th and 7th exist; it falls with alterations, a missing 3rd or a missing root. Click a reading to relabel the intervals and hear it over its root."
                }
            }
        }
    }
}
