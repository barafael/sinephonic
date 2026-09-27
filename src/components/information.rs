use dioxus::prelude::*;

use super::SectionHead;
use crate::state::use_app;

struct Row {
    label: &'static str,
    value: String,
    fill: f64,
    why: &'static str,
}

/// Entropy and information content, each with its scale and meaning.
#[component]
pub fn Information() -> Element {
    let app = use_app();
    let analysis = app.analysis.read();
    let Some(a) = analysis.as_ref() else {
        return rsx! {};
    };
    let Some(i) = a.information.as_ref() else {
        return rsx! {};
    };
    let rows = [
        Row {
            label: "Harmonic entropy",
            value: format!("{:.2} bits", i.harmonic_entropy),
            fill: i.harmonic_entropy_norm,
            why: "How unsure the ear is which simple ratio each interval stands for (Erlich). Mean over the pairs; low for 3/2 or 5/4, high between them.",
        },
        Row {
            label: "Description length",
            value: format!("{:.1} bits", i.description_bits),
            fill: (i.description_bits / 40.0).min(1.0),
            why: "Bits needed to write the chord as harmonics h₀ : h₁ : … of one fundamental: Σ log₂ hᵢ, the chord's Tenney height.",
        },
        Row {
            label: "Period",
            value: format!("{:.2} bits", i.smoothed_period_bits),
            fill: (i.smoothed_period_bits / 7.0).min(1.0),
            why: "log₂ of the common period in cycles, averaged over every note as reference (Stolzenburg's smoothed periodicity).",
        },
        Row {
            label: "Root entropy",
            value: format!("{:.2} / {:.2} bits", i.root_entropy, 12f64.log2()),
            fill: i.root_entropy_norm,
            why: "Shannon entropy of the 12 root readings: 0 when one root is certain, 3.58 when all are equally likely.",
        },
        Row {
            label: "Interval entropy",
            value: format!("{:.2} / {:.2} bits", i.interval_entropy, 6f64.log2()),
            fill: i.interval_entropy_norm,
            why: "Variety of interval classes in the set: stacked equal intervals are low, all-interval chords reach the maximum.",
        },
        Row {
            label: "Spectral entropy",
            value: format!("{:.2} / {:.2} bits", i.spectral_entropy, i.spectral_entropy_max),
            fill: if i.spectral_entropy_max > 0.0 { i.spectral_entropy / i.spectral_entropy_max } else { 0.0 },
            why: "Entropy of all partials after merging those within 20¢: coinciding partials fuse, which lowers it.",
        },
    ];
    rsx! {
        section { class: "wave-section",
            SectionHead { num: "06", title: "Information", hint: "entropy and complexity, in bits".to_string() }
            div { class: "info",
                div { class: "info-score",
                    span { class: "k", "Complexity" }
                    span { class: "v", "{i.complexity:.2}" }
                    span { class: "muted",
                        "0.3 harmonic entropy + 0.25 period + 0.2 root entropy + 0.15 interval entropy + 0.1 size, each scaled to 0–1."
                    }
                }
                div { class: "info-rows",
                    for r in rows {
                        div { class: "info-row",
                            span { class: "info-label", "{r.label}" }
                            div { class: "axis-track",
                                div { class: "axis-fill", style: "width:{r.fill * 100.0:.1}%" }
                            }
                            span { class: "info-value", "{r.value}" }
                            span { class: "info-why", "{r.why}" }
                        }
                    }
                }
            }
        }
    }
}
