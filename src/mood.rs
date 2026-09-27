//! A faint page tint that follows the chord's mood: cool for dark chords, warm for bright
//! ones, a touch of red as tension rises. Mixed in Oklab so blends never swing through
//! unrelated hues.

/// (a, b) in Oklab for a chroma and hue in degrees.
fn ab(chroma: f64, hue: f64) -> (f64, f64) {
    let h = hue.to_radians();
    (chroma * h.cos(), chroma * h.sin())
}

/// The tint offset for a valence and arousal in 0..1.
fn offset(valence: f64, arousal: f64) -> (f64, f64) {
    let cool = ab(0.032, 255.0);
    let warm = ab(0.03, 80.0);
    let tense = ab(0.02, 25.0);
    let v = valence.clamp(0.0, 1.0);
    let t = ((arousal - 0.35) / 0.65).clamp(0.0, 1.0);
    (
        cool.0 + (warm.0 - cool.0) * v + tense.0 * t,
        cool.1 + (warm.1 - cool.1) * v + tense.1 * t,
    )
}

/// CSS colours for the light and dark page backgrounds.
pub fn backgrounds(valence: f64, arousal: f64) -> (String, String) {
    let (a, b) = offset(valence, arousal);
    // The neutral papers (#f5f3ee, #161412) are slightly warm already.
    let light = format!("oklab(0.963 {:.4} {:.4})", 0.001 + 0.4 * a, 0.006 + 0.4 * b);
    let dark = format!("oklab(0.18 {:.4} {:.4})", 0.002 + 0.7 * a, 0.004 + 0.7 * b);
    (light, dark)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tints_stay_faint_and_move_with_mood() {
        let (dark_mood, _) = offset(0.0, 0.0);
        let (bright_mood, _) = offset(1.0, 0.0);
        assert!(dark_mood < 0.0 && bright_mood > 0.0, "a axis: cool → warm");
        for v in [0.0, 0.5, 1.0] {
            for a in [0.0, 0.5, 1.0] {
                let (x, y) = offset(v, a);
                // Light-mode chroma stays under 0.02: primarily bright paper.
                assert!((0.4 * x).hypot(0.4 * y) < 0.02);
            }
        }
        assert!(backgrounds(0.5, 0.5).0.starts_with("oklab(0.963"));
    }
}
