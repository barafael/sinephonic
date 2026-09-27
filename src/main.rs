//! Chord Anatomy: a one-screen chord-voicing analyser. The analysis lives in the UI-free
//! `anatomy` and `harmony` crates; this crate only renders it.

use dioxus::prelude::*;

mod audio;
mod components;
mod midi_input;
mod mood;
mod presets;
mod state;

use components::{
    Character, DissonanceCurve, Header, Information, InterRelations, Intervals, Neighbours,
    PitchSpace, Readings, Spectrum, Voicing, Waveform,
};
use state::AppState;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const FONTS: &str = "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=Noto+Music&family=Source+Serif+4:ital,opsz,wght@0,8..60,400;0,8..60,600;1,8..60,400&display=swap";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let app = AppState::provide();
    state::use_url_sync(app);
    // The page background leans with the chord's valence and arousal.
    use_effect(move || {
        let Some((v, a)) = app.analysis.read().as_ref().map(|x| (x.valence, x.arousal)) else {
            return;
        };
        let (light, dark) = mood::backgrounds(v, a);
        let _ = document::eval(&format!(
            "const s = document.documentElement.style; s.setProperty('--mood-light', '{light}'); s.setProperty('--mood-dark', '{dark}');"
        ));
    });
    // A different handful of presets on every visit.
    let mut seed = app.preset_seed;
    use_future(move || async move {
        if let Ok(v) = document::eval("return Math.floor(Math.random() * 4294967295) + 1").await {
            if let Some(x) = v.as_u64() {
                seed.set(x);
            }
        }
    });
    // The theme attribute sits on <html> so the page background follows it too.
    use_effect(move || {
        let theme = app.theme.cloned().attr();
        let _ = document::eval(&format!(
            "document.documentElement.dataset.theme = '{theme}'"
        ));
    });
    let has = app.notes.read().len() >= 2;
    rsx! {
        document::Title { "Chord Anatomy" }
        document::Link { rel: "preconnect", href: "https://fonts.googleapis.com" }
        document::Stylesheet { href: FONTS }
        document::Stylesheet { href: MAIN_CSS }
        div { class: "page",
            Header {}
            Voicing {}
            if has {
                div { class: "grid",
                    Readings {}
                    // Intervals and pitch-class space share a column so it matches Readings' height.
                    div { class: "stack",
                        Intervals {}
                        PitchSpace {}
                    }
                    InterRelations {}
                    Character {}
                }
                div { class: "wide-grid",
                    Information {}
                    DissonanceCurve {}
                    Waveform {}
                    Spectrum {}
                    Neighbours {}
                }
            } else {
                div { class: "empty", "Select two or more notes to analyze." }
            }
        }
    }
}
