use dioxus::prelude::*;
use harmony::Midi;

use super::{SettingsPanel, Staff};
use crate::audio;
use crate::state::use_app;

#[component]
pub fn Header() -> Element {
    let app = use_app();
    let mut show_settings = use_signal(|| false);
    let analysis = app.analysis.read();
    let notes = app.notes.read();
    let spelling = app.settings.read().spelling;
    let (name, line, summary) = match analysis.as_ref() {
        Some(a) => (
            a.best().symbol.clone(),
            a.note_names.join(" – "),
            a.summary(),
        ),
        None => match notes.first() {
            Some(&m) => (
                Midi(m).name(spelling),
                "one note".to_string(),
                String::new(),
            ),
            None => ("—".to_string(), "no notes".to_string(), String::new()),
        },
    };
    let play = move |arp: bool| audio::chord(app, arp);
    let staff_names: Vec<String> = match analysis.as_ref() {
        Some(a) => a.note_names.clone(),
        None => notes.iter().map(|&m| Midi(m).name(spelling)).collect(),
    };
    rsx! {
        header { class: "header",
            div { class: "header-left",
                div { class: "eyebrow-row",
                    span { class: "eyebrow", "Chord anatomy — voicing analysis" }
                    button {
                        class: "link-button",
                        onclick: move |_| show_settings.toggle(),
                        if show_settings() { "close settings" } else { "settings" }
                    }
                    if show_settings() {
                        SettingsPanel {}
                    }
                }
                div { class: "chord-name", "{name}" }
                div { class: "muted", "{line}" }
            }
            div { class: "header-staff", title: "click to play",
                Staff { names: staff_names, onclick: move |_| audio::chord(app, false) }
            }
            div { class: "header-right",
                div { class: "summary", "{summary}" }
                div { class: "buttons",
                    button { class: "btn btn-accent", onclick: move |_| play(false), "Play" }
                    button { class: "btn btn-outline", onclick: move |_| play(true), "Arpeggiate" }
                }
            }
        }
    }
}
