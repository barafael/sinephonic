use anatomy::interval_name;
use dioxus::prelude::*;
use harmony::PitchClass;

use super::SectionHead;
use crate::state::use_app;

/// "+1.9¢" / "−13.7¢" with a true minus sign.
fn cents(c: f64) -> String {
    let sign = if c >= 0.0 { "+" } else { "−" };
    format!("{sign}{:.1}¢", c.abs())
}

#[component]
pub fn Intervals() -> Element {
    let app = use_app();
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let sel = app
        .selected_root
        .cloned()
        .and_then(|r| a.reading(r))
        .unwrap_or(a.best());
    let fr = a.freqs(app.tuning.cloned());
    let p = &a.periodicity;
    let rows: Vec<_> = (0..a.notes.len())
        .rev()
        .map(|i| {
            let d = a.notes[i] - a.notes[0];
            let pc = PitchClass::of_midi(a.notes[i]);
            (
                i,
                a.note_names[i].clone(),
                sel.degree_of(pc).map_or("", |d| d.label()),
                if i == 0 {
                    "bass".to_string()
                } else {
                    interval_name(d)
                },
                format!("{d} st"),
                p.ratios[i].to_string(),
                if i == 0 {
                    "0.0¢".to_string()
                } else {
                    cents(p.cents[i])
                },
                format!("{:.1}", fr[i]),
            )
        })
        .collect();
    rsx! {
        section { class: "section",
            SectionHead { num: "03", title: "Intervals", hint: format!("functions as {}", sel.symbol) }
            div { class: "scroll-x",
                div { class: "iv-table",
                    div { class: "th", "Note" }
                    div { class: "th", "Degree" }
                    div { class: "th", "Above bass" }
                    div { class: "th r", "Just" }
                    div { class: "th r", "TET−just" }
                    div { class: "th r", "Hz" }
                    for (i, note, deg, iv, st, ratio, c, hz) in rows {
                        // A row plays its note with the bass (the bass row plays alone).
                        div {
                            class: "iv-row",
                            title: "click to hear with the bass",
                            onclick: move |_| crate::audio::subset(app, &if i == 0 { vec![0] } else { vec![0, i] }),
                            div { class: "td note", span { class: "dot dot-lg" } "{note}" }
                            div { class: "td deg", "{deg}" }
                            div { class: "td", "{iv}" span { class: "muted", " · {st}" } }
                            div { class: "td r", "{ratio}" }
                            div { class: "td r muted", "{c}" }
                            div { class: "td r", "{hz}" }
                        }
                    }
                }
            }
        }
    }
}
