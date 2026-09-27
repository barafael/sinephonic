use dioxus::prelude::*;

use super::{strip_octave, PcClock, SectionHead};
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
    let pcs: Vec<u8> = a.pcs.iter().map(|p| p.value()).collect();
    let spelling = app.settings.read().spelling;
    let bass_name = strip_octave(&a.note_names[0]).to_string();
    let identity = a.extras.as_ref().map(|x| {
        let iv: String = x.interval_vector.iter().map(u8::to_string).collect();
        let mut s = format!(
            "Set class {} · prime form {} · interval vector ⟨{iv}⟩",
            x.forte, x.prime_form
        );
        if let Some(name) = x.sonority {
            s += &format!(" · {name}");
        }
        s
    });
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
                            move |_| app.selected_root.set(Some(root))
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
            div { class: "identity-row",
                PcClock { pcs: pcs.clone(), root: selected.value(), bass: a.bass_pc.value(), step: 1, title: "chromatic", spelling }
                PcClock { pcs: pcs.clone(), root: selected.value(), bass: a.bass_pc.value(), step: 7, title: "fifths", spelling }
                if let Some(line) = identity {
                    p { class: "identity", "{line}" }
                }
            }
            p { class: "footnote",
                if a.extras.is_some() {
                    "Each of the 12 pitch classes is tried as root. Fit rises when the root is present or in the bass, when a clear 3rd, 5th and 7th exist, with psychoacoustic root support (Parncutt) and when the root is the root of the voicing's strongest interval (Hindemith); it falls with alterations, a missing 3rd or a missing root. Click a reading to relabel the intervals."
                } else {
                    "Each of the 12 pitch classes is tried as root. Fit rises when the root is present or in the bass and a clear 3rd, 5th and 7th exist; it falls with alterations, a missing 3rd or a missing root. Click a reading to relabel the intervals."
                }
            }
        }
    }
}
