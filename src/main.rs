//! Chord Anatomy: a one-screen chord-voicing analyser. The analysis lives in the UI-free
//! `anatomy` and `harmony` crates; this crate only renders it.

use dioxus::prelude::*;

mod audio;
mod components;
mod midi_input;
mod state;

use components::{
    Character, DissonanceCurve, Header, Information, InterRelations, Intervals, Neighbours,
    Readings, Spectrum, Voicing, Waveform,
};
use state::AppState;

const MAIN_CSS: Asset = asset!("/assets/main.css");
const FONTS: &str = "https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500&family=Source+Serif+4:ital,opsz,wght@0,8..60,400;0,8..60,600;1,8..60,400&display=swap";

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let app = AppState::provide();
    state::use_url_sync(app);
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
                    Intervals {}
                    InterRelations {}
                    Character {}
                }
                Information {}
                DissonanceCurve {}
                Waveform {}
                Spectrum {}
                Neighbours {}
            } else {
                div { class: "empty", "Select two or more notes to analyze." }
            }
        }
    }
}
