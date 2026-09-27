use anatomy::{Model, RatioSet, Spelling};
use dioxus::prelude::*;

use super::Segmented;
use crate::state::use_app;

/// Ratio set, reference pitch, spelling and model.
#[component]
pub fn SettingsPanel() -> Element {
    let mut app = use_app();
    let s = app.settings.cloned();
    rsx! {
        div { class: "settings",
            label { "ratio set" }
            Segmented {
                options: RatioSet::ALL.iter().map(|&r| (r, r.label().to_string())).collect::<Vec<_>>(),
                value: s.ratio_set,
                onchange: move |r| app.settings.write().ratio_set = r,
            }
            label { "A4 = {s.a4:.0} Hz" }
            input {
                r#type: "range",
                min: "415",
                max: "466",
                step: "1",
                value: "{s.a4}",
                oninput: move |e| {
                    if let Ok(v) = e.value().parse::<f64>() {
                        app.settings.write().a4 = v;
                    }
                },
            }
            label { "spelling" }
            Segmented {
                options: vec![(Spelling::Flats, "flats".to_string()), (Spelling::Sharps, "sharps".to_string())],
                value: s.spelling,
                onchange: move |sp| {
                    app.settings.write().spelling = sp;
                    let notes = app.notes.peek().clone();
                    app.text.set(harmony::midi::format_notes(&notes, sp));
                },
            }
            label { "model" }
            Segmented {
                options: vec![(Model::Improved, "improved".to_string()), (Model::Prototype, "prototype".to_string())],
                value: s.model,
                onchange: move |m| app.settings.write().model = m,
            }
            p { class: "note",
                "Improved: spelled chord tones, Parncutt root support, Hindemith's interval roots and Stolzenburg periodicity. Prototype: the design handoff's original heuristics."
            }
        }
    }
}
