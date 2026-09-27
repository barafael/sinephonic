use dioxus::prelude::*;
use harmony::PcSet;

use super::{PcClock, SectionHead};
use crate::state::use_app;

/// One interval-class bar: class index, count, fill 0..1, an example pair to play.
type IcBar = (usize, u8, f64, Option<(usize, usize)>);

const IC_NAMES: [&str; 6] = ["m2", "M2", "m3", "M3", "P4", "TT"];

/// The chord as a pitch-class set: clocks, Parncutt's root support for every candidate root,
/// the interval-class vector and set-class identity.
#[component]
pub fn PitchSpace() -> Element {
    let app = use_app();
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let spelling = app.settings.read().spelling;
    let selected = app.selected_root.cloned().unwrap_or(a.best().root);
    let set = PcSet::from_midi(&a.notes);
    let pcs: Vec<u8> = a.pcs.iter().map(|p| p.value()).collect();

    let iv = set.interval_vector();
    let iv_max = iv.iter().copied().max().unwrap_or(1).max(1) as f64;
    // An example pair for each interval class, to play on click.
    let example = |ic: usize| -> Option<(usize, usize)> {
        a.pairs
            .iter()
            .find(|p| {
                let d = p.semitones % 12;
                d.min(12 - d) as usize == ic + 1
            })
            .map(|p| (p.i, p.j))
    };
    let ics: Vec<IcBar> = (0..6)
        .map(|k| (k, iv[k], iv[k] as f64 / iv_max, example(k)))
        .collect();

    let forte = set.forte().map(|f| f.to_string()).unwrap_or_default();
    let symmetry = set.transpositional_symmetry();
    let inversional = set.inversional_symmetry() > 0;
    let sonority = a.extras.as_ref().and_then(|x| x.sonority);
    let ambiguity = a.extras.as_ref().map(|x| x.root_ambiguity);

    rsx! {
        section { class: "section",
            SectionHead { num: "04", title: "Pitch-class space", hint: "click to hear".to_string() }
            div { class: "identity-row",
                PcClock { pcs: pcs.clone(), root: selected.value(), bass: a.bass_pc.value(), step: 1, title: "chromatic circle", spelling, onplay: move |pc: u8| crate::audio::midis(app, &[60 + pc]) }
                PcClock { pcs: pcs.clone(), root: selected.value(), bass: a.bass_pc.value(), step: 7, title: "circle of fifths", spelling, onplay: move |pc: u8| crate::audio::midis(app, &[60 + pc]) }
            }
            div { class: "ps-facts",
                div { class: "ps-fact", span { class: "k", "Set class" } span { class: "v", "{forte}" } }
                div { class: "ps-fact", span { class: "k", "Prime form" } span { class: "v", "{set.prime_form()}" } }
                div { class: "ps-fact",
                    span { class: "k", "Symmetry" }
                    span { class: "v",
                        match (symmetry > 1, inversional) {
                            (true, true) => format!("T{} · mirror", 12 / symmetry),
                            (true, false) => format!("T{}", 12 / symmetry),
                            (false, true) => "mirror".to_string(),
                            (false, false) => "none".to_string(),
                        }
                    }
                }
                if let Some(x) = ambiguity {
                    div { class: "ps-fact", span { class: "k", "Root ambiguity" } span { class: "v", "{x:.2}" } }
                }
                if let Some(name) = sonority {
                    div { class: "ps-fact", span { class: "k", "Known as" } span { class: "v", "{name}" } }
                }
            }
            div { class: "ps-charts",
                div { class: "ps-chart",
                    div { class: "ps-title", "Interval-class vector ⟨{iv.iter().map(u8::to_string).collect::<String>()}⟩" }
                    div { class: "bars bars-6",
                        for (k, count, v, ex) in ics {
                            button {
                                class: if count > 0 { "bar present" } else { "bar" },
                                title: "hear an example",
                                disabled: ex.is_none(),
                                onclick: move |_| {
                                    if let Some((i, j)) = ex {
                                        crate::audio::subset(app, &[i, j]);
                                    }
                                },
                                span { class: "bar-track", span { class: "bar-fill", style: "height:{v * 100.0:.0}%" } }
                                span { class: "bar-count", "{count}" }
                                span { class: "bar-label", "{IC_NAMES[k]}" }
                            }
                        }
                    }
                }
            }
            p { class: "footnote",
                "Symmetry: Tn means transposing by n semitones maps the set onto itself; mirror means some inversion does. Root ambiguity is Parncutt's √(Σ support / max support). The interval vector counts every pair of pitch classes by interval class; click a bar to hear one."
            }
        }
    }
}
