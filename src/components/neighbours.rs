use anatomy::explore::neighbours;
use dioxus::prelude::*;

use super::SectionHead;
use crate::state::{options, use_app};

/// Chords one voice-leading step away: each note moved by a tone or a semitone.
#[component]
pub fn Neighbours() -> Element {
    let mut app = use_app();
    let found = use_memo(move || {
        let opts = options(
            &app.settings.read(),
            app.tuning.cloned(),
            app.timbre.cloned(),
        );
        neighbours(&app.notes.read(), &opts)
    });
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let here = a.axes.tension;
    let found = found.read();
    let rows: Vec<(String, Vec<Option<anatomy::explore::Neighbour>>)> = (0..a.notes.len())
        .rev()
        .map(|i| {
            let cells = [-2i8, -1, 1, 2]
                .iter()
                .map(|&s| found.iter().find(|n| n.moved == i && n.step == s).cloned())
                .collect();
            (a.note_names[i].clone(), cells)
        })
        .collect();
    rsx! {
        section { class: "section",
            SectionHead { num: "11", title: "Neighbours", hint: "move one note · click to go there and hear it".to_string() }
            div { class: "scroll-x",
                div { class: "nb-table",
                    div { class: "th", "Note" }
                    div { class: "th c", "↓ tone" }
                    div { class: "th c", "↓ semitone" }
                    div { class: "th c", "↑ semitone" }
                    div { class: "th c", "↑ tone" }
                    for (name, cells) in rows {
                        div { class: "td note", span { class: "dot dot-lg" } "{name}" }
                        for cell in cells {
                            match cell {
                                Some(n) => {
                                    let delta = n.tension - here;
                                    let arrow = if delta > 0.03 { "tenser" } else if delta < -0.03 { "calmer" } else { "same tension" };
                                    let notes = n.notes.clone();
                                    rsx! {
                                        button {
                                            class: "nb-cell",
                                            title: "{arrow} ({n.tension:.2})",
                                            onclick: move |_| {
                                                app.set_notes(notes.iter().copied());
                                                crate::audio::chord(app, false);
                                            },
                                            span { class: "nb-sym", "{n.symbol}" }
                                            span { class: "nb-bar",
                                                span { class: "nb-fill", style: "width:{n.tension * 100.0:.0}%" }
                                            }
                                            span { class: "nb-delta", "{arrow}" }
                                        }
                                    }
                                }
                                None => rsx! { div { class: "nb-cell empty-cell", "—" } },
                            }
                        }
                    }
                }
            }
            p { class: "footnote",
                "Parsimonious voice leading: the smoothest moves between chords change one note by a step (Cohn, Tymoczko). The bar is each neighbour's tension; the word compares it with the current chord."
            }
        }
    }
}
