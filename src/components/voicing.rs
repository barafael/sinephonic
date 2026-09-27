use dioxus::prelude::*;
use harmony::PitchClass;

use super::SectionHead;
use crate::presets::{next, pick, POOL};
use crate::state::use_app;

/// The keyboard spans C2–C6 (C3–C5 on narrow screens), widened by whole octaves to include
/// every note of the voicing.
fn key_range(notes: &[u8], narrow: bool) -> (u8, u8) {
    let (mut lo, mut hi) = if narrow { (48u8, 72u8) } else { (36, 84) };
    if let (Some(&first), Some(&last)) = (notes.first(), notes.last()) {
        lo = lo.min(first / 12 * 12);
        hi = hi.max(last.div_ceil(12) * 12).min(127);
    }
    (lo, hi)
}

#[component]
pub fn Voicing() -> Element {
    let mut app = use_app();
    let midi = use_signal(|| None::<String>);
    let notes = app.notes.read().clone();
    let mut kb_width = use_signal(|| 1200.0f64);
    let (low, high) = key_range(&notes, kb_width() < 640.0);
    let spelling = app.settings.read().spelling;
    // Active keys show the note's spelled name from the analysis.
    let label = |m: u8| -> Option<String> {
        let a = app.analysis.read();
        let i = notes.iter().position(|&x| x == m)?;
        Some(match a.as_ref() {
            Some(a) => a.note_names[i].clone(),
            None => harmony::Midi(m).name(spelling),
        })
    };
    let white_labels: Vec<Option<String>> = (low..=high).map(label).collect();
    // Degree of each active key under the selected reading.
    let degrees: Vec<Option<&'static str>> = {
        let a = app.analysis.read();
        (low..=high)
            .map(|m| {
                let a = a.as_ref()?;
                notes.contains(&m).then_some(())?;
                let r = app
                    .selected_root
                    .cloned()
                    .and_then(|r| a.reading(r))
                    .unwrap_or(a.best());
                r.degree_of(PitchClass::of_midi(m)).map(|d| d.label())
            })
            .collect()
    };
    let sounding = app.sounding.read().clone();
    let key_class = |base: &str, m: u8| {
        let mut c = base.to_string();
        if notes.contains(&m) {
            c += " on";
        }
        if sounding.contains(&m) {
            c += " sounding";
        }
        c
    };
    // Clicking an unlit key adds it and plays it; clicking a lit key removes it.
    let press = move |m: u8| {
        let mut app = app;
        let adding = !app.notes.peek().contains(&m);
        app.toggle(m);
        if adding {
            crate::audio::note(app, m);
        }
    };
    let whites: Vec<u8> = (low..=high)
        .filter(|&m| !PitchClass::of_midi(m).is_black_key())
        .collect();
    let wn = whites.len() as f64;
    let blacks: Vec<(u8, f64)> = (low..=high)
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
            SectionHead { num: "01", title: "Voicing", hint: format!("click keys to toggle and hear · C{}–C{}", low / 12 - 1, high / 12 - 1) }
            div {
                class: "keyboard",
                onresize: move |e| {
                    if let Ok(size) = e.get_content_box_size() {
                        if size.width > 0.0 && (size.width - kb_width()).abs() >= 1.0 {
                            kb_width.set(size.width);
                        }
                    }
                },
                for m in whites {
                    div {
                        key: "{m}",
                        class: key_class("white-key", m),
                        onclick: move |_| press(m),
                        if let Some(d) = degrees[(m - low) as usize] {
                            span { class: "kdeg", "{d}" }
                        }
                        if let Some(l) = &white_labels[(m - low) as usize] {
                            span { "{l}" }
                        } else if m % 12 == 0 {
                            span { "C{m / 12 - 1}" }
                        }
                    }
                }
                for (m, left) in blacks {
                    div {
                        key: "{m}",
                        class: key_class("black-key", m),
                        style: "left:{left}%;width:{width}%",
                        onclick: move |_| press(m),
                        if let Some(d) = degrees[(m - low) as usize] {
                            span { class: "kdeg", "{d}" }
                        }
                        if let Some(l) = &white_labels[(m - low) as usize] {
                            span { class: "klabel", "{l}" }
                        }
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
                    for k in pick(app.preset_seed.cloned(), 10) {
                        {
                            let (label, text) = POOL[k];
                            let preset = harmony::midi::parse_notes(text);
                            let on = notes == preset;
                            rsx! {
                                button {
                                    key: "{k}",
                                    class: if on { "chip on" } else { "chip" },
                                    title: "{text}",
                                    onclick: move |_| {
                                        app.set_notes(preset.iter().copied());
                                        crate::audio::chord(app, false);
                                    },
                                    "{label}"
                                }
                            }
                        }
                    }
                    button {
                        class: "clear",
                        title: "show other presets",
                        onclick: move |_| {
                            let s = next(app.preset_seed.cloned());
                            app.preset_seed.set(s);
                        },
                        "shuffle"
                    }
                    button { class: "clear", onclick: move |_| app.set_notes([]), "clear" }
                    match midi() {
                        None => rsx! {
                            button { class: "clear", onclick: move |_| crate::midi_input::connect(app, midi), "connect MIDI" }
                        },
                        Some(s) => rsx! { span { class: "midi-status", "{s}" } },
                    }
                }
            }
        }
    }
}
