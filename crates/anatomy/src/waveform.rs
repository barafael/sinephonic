//! Waveform geometry: time span, period markers and SVG path data. The UI draws the paths.

/// Longest time window shown.
pub const MAX_SPAN: f64 = 0.25;

/// The visible time span for `periods` chord periods, and whether the cap applied.
pub fn span(period: f64, periods: u32) -> (f64, bool) {
    let s = period * periods as f64;
    if s > MAX_SPAN {
        (MAX_SPAN, true)
    } else {
        (s, false)
    }
}

/// Times k·T that fall inside the span.
pub fn markers(period: f64, span: f64) -> Vec<f64> {
    let mut out = Vec::new();
    let mut k = 0;
    while k as f64 * period <= span + 1e-9 {
        out.push(k as f64 * period);
        k += 1;
        if k > 10_000 {
            break;
        }
    }
    out
}

/// Marker label: "0", "T = 76.5 ms", "2T = 152.9 ms".
pub fn marker_label(k: usize, t: f64) -> String {
    match k {
        0 => "0".into(),
        1 => format!("T = {:.1} ms", t * 1000.0),
        k => format!("{k}T = {:.1} ms", t * 1000.0),
    }
}

/// Path data for `y = mid − amp · f(t)` across `width` user units from `x0`, sampled every
/// `step` units, with t running from 0 to `span`.
pub fn path(
    x0: f64,
    width: f64,
    step: f64,
    span: f64,
    mid: f64,
    amp: f64,
    f: impl Fn(f64) -> f64,
) -> String {
    use std::fmt::Write;
    let mut d = String::with_capacity((width / step) as usize * 16);
    let n = (width / step).ceil() as usize;
    for i in 0..=n {
        let x = (i as f64 * step).min(width);
        let t = x / width * span;
        let y = mid - amp * f(t);
        let cmd = if i == 0 { 'M' } else { 'L' };
        let _ = write!(d, "{cmd}{:.2} {:.2}", x0 + x, y);
    }
    d
}

/// A pure sine at `freq` with zero phase at t = 0.
pub fn sine(freq: f64) -> impl Fn(f64) -> f64 {
    move |t| (std::f64::consts::TAU * freq * t).sin()
}

/// Mean of sines at `freqs`.
pub fn sum(freqs: &[f64]) -> impl Fn(f64) -> f64 + '_ {
    move |t| {
        freqs
            .iter()
            .map(|f| (std::f64::consts::TAU * f * t).sin())
            .sum::<f64>()
            / freqs.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spans_and_markers() {
        assert_eq!(span(0.0764, 2), (0.0764 * 2.0, false));
        assert_eq!(span(0.1529, 2), (MAX_SPAN, true));
        assert_eq!(markers(0.1, 0.25).len(), 3);
        assert_eq!(markers(0.1, 0.2).len(), 3);
        assert_eq!(marker_label(2, 0.15291), "2T = 152.9 ms");
    }

    #[test]
    fn path_shape() {
        let d = path(120.0, 10.0, 5.0, 1.0, 50.0, 10.0, |_| 1.0);
        assert_eq!(d, "M120.00 40.00L125.00 40.00L130.00 40.00");
        // A sum of in-phase sines repeats at the common period.
        let f = sum(&[100.0, 150.0]);
        assert!((f(0.02) - f(0.0)).abs() < 1e-9);
    }
}
