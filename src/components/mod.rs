mod character;
mod clock;
mod curve;
mod header;
mod information;
mod intervals;
mod matrix;
mod neighbours;
mod readings;
mod segmented;
mod settings;
mod spectrum;
mod staff;
mod voicing;
mod waveform;

pub use character::Character;
pub use clock::PcClock;
pub use curve::DissonanceCurve;
pub use header::Header;
pub use information::Information;
pub use intervals::Intervals;
pub use matrix::InterRelations;
pub use neighbours::Neighbours;
pub use readings::Readings;
pub use segmented::Segmented;
pub use settings::SettingsPanel;
pub use spectrum::Spectrum;
pub use staff::Staff;
pub use voicing::Voicing;
pub use waveform::Waveform;

use dioxus::prelude::*;

/// Numbered section heading: "02  READINGS ─────── hint".
#[component]
pub fn SectionHead(num: &'static str, title: &'static str, hint: Option<String>) -> Element {
    rsx! {
        div { class: "section-head",
            span { class: "num", "{num}" }
            span { "{title}" }
            span { class: "rule" }
            if let Some(h) = hint {
                span { class: "hint", "{h}" }
            }
        }
    }
}

/// "E♭4" → "E♭".
pub fn strip_octave(s: &str) -> &str {
    s.trim_end_matches(|c: char| c.is_ascii_digit() || c == '-')
}
