use dioxus::prelude::*;
use harmony::SpelledPc;

/// Diatonic step of a spelled note name like "E♭4": C0 = 0, D0 = 1, … (7 per octave).
fn step_of(name: &str) -> Option<(i32, i32)> {
    let split = name.find(|c: char| c.is_ascii_digit() || c == '-')?;
    let (letters, octave) = name.split_at(split);
    let s = SpelledPc::parse(letters)?;
    let octave: i32 = octave.parse().ok()?;
    let letter = "CDEFGAB".find(s.letter())? as i32;
    Some((letter + 7 * octave, s.accidentals()))
}

const GAP: f64 = 10.0;
const TREBLE_TOP: f64 = 36.0; // y of F5
const BASS_TOP: f64 = 126.0; // y of A3
const MIDDLE_C: i32 = 28;

fn y_of(step: i32) -> f64 {
    if step >= MIDDLE_C {
        TREBLE_TOP + (38 - step) as f64 * GAP / 2.0
    } else {
        BASS_TOP + (26 - step) as f64 * GAP / 2.0
    }
}

fn accidental(a: i32) -> &'static str {
    match a {
        -2 => "𝄫",
        -1 => "♭",
        1 => "♯",
        2 => "𝄪",
        _ => "",
    }
}

/// The voicing on a grand staff, spelled as the analysis spells it (whole notes, seconds
/// displaced, accidentals stacked in columns).
#[component]
pub fn Staff(names: Vec<String>, onclick: EventHandler<()>) -> Element {
    let mut notes: Vec<(i32, i32)> = names.iter().filter_map(|n| step_of(n)).collect();
    notes.sort();
    let x0 = 118.0;
    // Seconds on the same staff: displace every other note of a cluster to the right.
    let mut heads: Vec<(f64, f64, i32, i32)> = Vec::new(); // x, y, step, acc
    let mut prev: Option<(i32, bool)> = None;
    for &(step, acc) in &notes {
        let same_staff = prev.is_some_and(|(p, _)| (p >= MIDDLE_C) == (step >= MIDDLE_C));
        let shifted = match prev {
            Some((p, was)) if same_staff && step - p == 1 => !was,
            _ => false,
        };
        heads.push((if shifted { x0 + 13.0 } else { x0 }, y_of(step), step, acc));
        prev = Some((step, shifted));
    }
    // Accidental columns, from the top note down: a column is free if no accidental in it is
    // within a sixth above.
    let mut placed: Vec<(i32, usize)> = Vec::new();
    let mut accs: Vec<(f64, f64, &'static str)> = Vec::new();
    for &(_, y, step, acc) in heads.iter().rev() {
        if acc == 0 {
            continue;
        }
        let col = (0..)
            .find(|&c| {
                !placed
                    .iter()
                    .any(|&(s, pc)| pc == c && (s - step).abs() < 6)
            })
            .unwrap_or(0);
        placed.push((step, col));
        accs.push((x0 - 13.0 - col as f64 * 11.0, y, accidental(acc)));
    }
    // Ledger lines: even steps between the note and its staff (treble E4–F5, bass G2–A3).
    let mut ledgers: Vec<(f64, f64)> = Vec::new();
    for &(x, _, step, _) in &heads {
        let (bottom, top) = if step >= MIDDLE_C { (30, 38) } else { (18, 26) };
        let lines =
            (step.min(bottom)..=step.max(top)).filter(|s| s % 2 == 0 && (*s < bottom || *s > top));
        for l in lines {
            ledgers.push((x, y_of(l)));
        }
    }
    let staff_lines: Vec<f64> = (0..5)
        .map(|k| TREBLE_TOP + k as f64 * GAP)
        .chain((0..5).map(|k| BASS_TOP + k as f64 * GAP))
        .collect();
    rsx! {
        svg {
            class: "staff",
            view_box: "0 0 180 200",
            onclick: move |_| onclick.call(()),
            role: "img",
            "aria-label": "Notation: {names.join(\", \")}",
            for y in staff_lines {
                line { class: "staff-line", x1: "14", x2: "172", y1: "{y}", y2: "{y}" }
            }
            line { class: "staff-line", x1: "14", x2: "14", y1: "{TREBLE_TOP}", y2: "{BASS_TOP + 4.0 * GAP}" }
            text { class: "clef", x: "18", y: "{TREBLE_TOP + 3.0 * GAP + 1.0}", font_size: "46", "𝄞" }
            text { class: "clef", x: "20", y: "{BASS_TOP + GAP + 1.0}", font_size: "34", "𝄢" }
            for (x, y) in ledgers {
                line { class: "staff-line", x1: "{x - 10.0}", x2: "{x + 10.0}", y1: "{y}", y2: "{y}" }
            }
            for (x, y, glyph) in accs {
                text { class: "accidental", x: "{x}", y: "{y + 5.0}", text_anchor: "middle", "{glyph}" }
            }
            for (x, y, _, _) in heads {
                ellipse { class: "notehead", cx: "{x}", cy: "{y}", rx: "6.4", ry: "4.6", transform: "rotate(-20 {x} {y})" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn steps() {
        assert_eq!(step_of("C4"), Some((28, 0)));
        assert_eq!(step_of("E♭4"), Some((30, -1)));
        assert_eq!(step_of("B𝄫3"), Some((27, -2)));
        assert_eq!(step_of("F♯5"), Some((38, 1)));
        assert_eq!(y_of(38), TREBLE_TOP);
        assert_eq!(y_of(26), BASS_TOP);
    }
}
