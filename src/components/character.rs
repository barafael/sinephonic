use dioxus::prelude::*;

use super::SectionHead;
use crate::state::{options, use_app, REFERENCES};

fn position(valence: f64, arousal: f64) -> (f64, f64) {
    (8.0 + valence * 84.0, 92.0 - arousal * 84.0)
}

#[component]
pub fn Character() -> Element {
    let app = use_app();
    let chords = REFERENCES;
    // Reference dots only change with the options, not the voicing.
    let refs = use_memo(move || {
        let opts = options(
            &app.settings.read(),
            app.tuning.cloned(),
            app.timbre.cloned(),
        );
        REFERENCES
            .iter()
            .map(|&(label, notes)| {
                let r = anatomy::analyze(notes, &opts).expect("reference chords have 3+ notes");
                (label, position(r.valence, r.arousal))
            })
            .collect::<Vec<_>>()
    });
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let (x, y) = position(a.valence, a.arousal);
    rsx! {
        section { class: "section",
            SectionHead { num: "05", title: "Character" }
            div { class: "tags",
                for t in a.tags.iter().take(5) {
                    div {
                        class: "tag playable",
                        title: "click to hear the notes this is about",
                        onclick: {
                            let idx = t.notes.clone();
                            move |_| crate::audio::subset(app, &idx)
                        },
                        div { class: "w", "{t.word}" }
                        div { class: "why", "{t.why}" }
                    }
                }
            }
            div { class: "character-lower",
                div { class: "axes",
                    for (label, v) in a.axes.labelled() {
                        div { class: "axis",
                            span { "{label}" }
                            div { class: "axis-track",
                                div { class: "axis-fill", style: "width:{v * 100.0:.1}%" }
                            }
                            span { class: "v", "{v:.2}" }
                        }
                    }
                }
                div { class: "va",
                    span { class: "va-label", "tense" }
                    div { class: "va-mid",
                        span { class: "va-label", "dark" }
                        div { class: "va-box",
                            div { class: "va-v" }
                            div { class: "va-h" }
                            for (k, (label, (rx, ry))) in refs().into_iter().enumerate() {
                                div {
                                    class: "ref-dot playable",
                                    style: "left:{rx:.1}%;top:{ry:.1}%",
                                    title: "click to hear",
                                    onclick: move |_| crate::audio::midis(app, chords[k].1),
                                    span { class: "d" }
                                    span { class: "l", "{label}" }
                                }
                            }
                            div {
                                class: "va-dot playable",
                                style: "left:{x:.1}%;top:{y:.1}%",
                                onclick: move |_| crate::audio::chord(app, false),
                            }
                        }
                        span { class: "va-label", "bright" }
                    }
                    span { class: "va-label", "calm" }
                }
            }
        }
    }
}
