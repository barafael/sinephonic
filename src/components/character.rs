use dioxus::prelude::*;

use super::SectionHead;
use crate::state::{options, use_app, REFERENCES};

/// Map position in percent. Most chords score between 0.25 and 0.8, so the display stretches
/// around the centre lines (gain 1.6) to separate them; the bars show the true values.
fn position(valence: f64, arousal: f64) -> (f64, f64) {
    let stretch = |v: f64| (50.0 + (v - 0.5) * 1.6 * 84.0).clamp(6.0, 94.0);
    (stretch(valence), 100.0 - stretch(arousal))
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
            SectionHead { num: "06", title: "Character" }
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
                    div { class: "va-mid",
                        div { class: "va-box",
                            div { class: "va-v" }
                            div { class: "va-h" }
                            span { class: "va-label in tl", "dark · tense" }
                            span { class: "va-label in tr", "bright · tense" }
                            span { class: "va-label in bl", "dark · calm" }
                            span { class: "va-label in br", "bright · calm" }
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
                    }
                }
            }
        }
    }
}
