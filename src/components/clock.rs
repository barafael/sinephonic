use dioxus::prelude::*;
use harmony::{PitchClass, Spelling};

/// The chord's pitch classes on a circle: chromatic (step 1) or fifths (step 7). The polygon
/// makes symmetry visible (dim7 is a square, aug a triangle); the root is filled, the bass
/// ringed.
#[component]
pub fn PcClock(
    pcs: Vec<u8>,
    root: u8,
    bass: u8,
    step: u8,
    title: &'static str,
    spelling: Spelling,
) -> Element {
    let size = 132.0;
    let c = size / 2.0;
    let r = 40.0;
    let pos = |pc: u8| {
        // Position of pc on the circle: index k with k·step ≡ pc (mod 12); 7 is its own inverse.
        let k = (pc as u32 * step as u32 % 12) as f64;
        let a = k / 12.0 * std::f64::consts::TAU - std::f64::consts::FRAC_PI_2;
        (c + r * a.cos(), c + r * a.sin())
    };
    let mut ordered: Vec<u8> = pcs.clone();
    ordered.sort_by_key(|&p| p as u32 * step as u32 % 12);
    let poly: String = ordered
        .iter()
        .map(|&p| {
            let (x, y) = pos(p);
            format!("{x:.1},{y:.1}")
        })
        .collect::<Vec<_>>()
        .join(" ");
    let labels: Vec<(f64, f64, String, bool)> = (0..12u8)
        .map(|pc| {
            let k = (pc as u32 * step as u32 % 12) as f64;
            let a = k / 12.0 * std::f64::consts::TAU - std::f64::consts::FRAC_PI_2;
            (
                c + (r + 13.0) * a.cos(),
                c + (r + 13.0) * a.sin() + 3.5,
                spelling.pc_name(PitchClass::new(pc as i32)).to_string(),
                pcs.contains(&pc),
            )
        })
        .collect();
    let dots: Vec<(f64, f64, bool, bool)> = pcs
        .iter()
        .map(|&p| {
            let (x, y) = pos(p);
            (x, y, p == root, p == bass)
        })
        .collect();
    rsx! {
        figure { class: "clock",
            svg { view_box: "0 0 {size} {size}", width: "{size}", height: "{size}",
                circle { class: "clock-ring", cx: "{c}", cy: "{c}", r: "{r}" }
                if pcs.len() > 1 {
                    polygon { class: "clock-poly", points: "{poly}" }
                }
                for (x, y, text, on) in labels {
                    text { class: if on { "clock-label on" } else { "clock-label" }, x: "{x}", y: "{y}", text_anchor: "middle", "{text}" }
                }
                for (x, y, is_root, is_bass) in dots {
                    if is_bass {
                        circle { class: "clock-bass", cx: "{x}", cy: "{y}", r: "7" }
                    }
                    circle { class: if is_root { "clock-dot root" } else { "clock-dot" }, cx: "{x}", cy: "{y}", r: "4" }
                }
            }
            figcaption { "{title}" }
        }
    }
}
