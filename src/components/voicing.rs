use dioxus::prelude::*;
use harmony::PitchClass;

use super::SectionHead;
use crate::state::{use_app, PRESETS};

const LOW: u8 = 36;
const HIGH: u8 = 84;

#[component]
pub fn Voicing() -> Element {
    let mut app = use_app();
    let notes = app.notes.read().clone();
    let whites: Vec<u8> = (LOW..=HIGH)
        .filter(|&m| !PitchClass::of_midi(m).is_black_key())
        .collect();
    let wn = whites.len() as f64;
    let blacks: Vec<(u8, f64)> = (LOW..=HIGH)
        .filter(|&m| PitchClass::of_midi(m).is_black_key())
        .map(|m| {
            let idx = whites
                .iter()
                .position(|&w| w == m - 1)
                .expect("white key below") as f64;
            (m, ((idx + 1.0) / wn - 0.3 / wn) * 100.0)
        })
        .collect();
    let width = 0.6 / wn * 100.0;
    rsx! {
        section { class: "section", style: "gap:16px",
            SectionHead { num: "01", title: "Voicing", hint: "click keys to toggle · C2–C6".to_string() }
            div { class: "keyboard",
                for m in whites {
                    div {
                        key: "{m}",
                        class: if notes.contains(&m) { "white-key on" } else { "white-key" },
                        onclick: move |_| app.toggle(m),
                        if m % 12 == 0 { "C{m / 12 - 1}" }
                    }
                }
                for (m, left) in blacks {
                    div {
                        key: "{m}",
                        class: if notes.contains(&m) { "black-key on" } else { "black-key" },
                        style: "left:{left}%;width:{width}%",
                        onclick: move |_| app.toggle(m),
                    }
                }
            }
            div { class: "input-row",
                input {
                    class: "note-input",
                    value: "{app.text}",
                    placeholder: "C3 G3 Ab3 Eb4",
                    spellcheck: "false",
                    oninput: move |e| app.text.set(e.value()),
                    onkeydown: move |e| {
                        if e.key() == Key::Enter {
                            app.commit_text();
                        }
                    },
                    onblur: move |_| app.commit_text(),
                }
                div { class: "chips",
                    for (label, preset) in PRESETS {
                        button {
                            class: if notes == preset { "chip on" } else { "chip" },
                            onclick: move |_| app.set_notes(preset.iter().copied()),
                            "{label}"
                        }
                    }
                    button { class: "clear", onclick: move |_| app.set_notes([]), "clear" }
                }
            }
        }
    }
}
