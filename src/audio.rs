//! Playback through WebAudio. The same script runs in the browser and in the desktop webview.
//! Every play also marks the sounding MIDI notes, so the keyboard can light them up.

use anatomy::Timbre;
use dioxus::prelude::*;

use crate::state::AppState;

/// Envelope 0.0001 → 1 over 20 ms, then exponential decay over `dur`; master gain 0.35/√n;
/// the 6-partial timbre uses harmonics 1–6 at 0.88^(k−1)/k.
const SCRIPT: &str = r#"
(() => {
  const voices = __VOICES__, harmonic = __HARMONIC__, dur = __DUR__;
  const ctx = window.__chordAnatomyCtx || (window.__chordAnatomyCtx = new (window.AudioContext || window.webkitAudioContext)());
  ctx.resume();
  const now = ctx.currentTime + 0.02;
  const master = ctx.createGain();
  master.gain.value = 0.35 / Math.sqrt(Math.max(1, voices.length));
  master.connect(ctx.destination);
  let wave = null;
  if (harmonic) {
    const re = new Float32Array(7), im = new Float32Array(7);
    for (let k = 1; k < 7; k++) im[k] = 0.88 ** (k - 1) / k;
    wave = ctx.createPeriodicWave(re, im);
  }
  voices.forEach(([f, delay, level]) => {
    const o = ctx.createOscillator();
    if (wave) o.setPeriodicWave(wave); else o.type = 'sine';
    o.frequency.value = f;
    const g = ctx.createGain(), t0 = now + delay;
    g.gain.setValueAtTime(0.0001, t0);
    g.gain.exponentialRampToValueAtTime(level, t0 + 0.02);
    g.gain.exponentialRampToValueAtTime(0.001, t0 + dur);
    o.connect(g).connect(master);
    o.start(t0);
    o.stop(t0 + dur + 0.1);
  });
})();
"#;

/// One oscillator: frequency, start delay in seconds, peak level (1 = full).
#[derive(Clone, Copy, Debug)]
pub struct Voice {
    pub freq: f64,
    pub delay: f64,
    pub level: f64,
}

pub fn play_voices(voices: &[Voice], timbre: Timbre, dur: f64) {
    let list = voices
        .iter()
        .map(|v| format!("[{:.6},{:.4},{:.4}]", v.freq, v.delay, v.level))
        .collect::<Vec<_>>()
        .join(",");
    let js = SCRIPT
        .replace("__VOICES__", &format!("[{list}]"))
        .replace(
            "__HARMONIC__",
            if timbre == Timbre::Harmonic6 {
                "true"
            } else {
                "false"
            },
        )
        .replace("__DUR__", &format!("{dur:.3}"));
    let _ = document::eval(&js);
}

/// Plays `freqs` together (or 0.32 s apart) and lights up `midis` on the keyboard meanwhile.
pub fn sound(app: AppState, midis: Vec<u8>, freqs: &[f64], arpeggio: bool, dur: f64) {
    let voices: Vec<Voice> = freqs
        .iter()
        .enumerate()
        .map(|(i, &f)| Voice {
            freq: f,
            delay: if arpeggio { i as f64 * 0.32 } else { 0.0 },
            level: 1.0,
        })
        .collect();
    play_voices(&voices, app.timbre.cloned(), dur);
    let total = dur
        + if arpeggio {
            0.32 * freqs.len() as f64
        } else {
            0.0
        };
    light(app, midis, total * 0.6);
}

/// Marks `midis` as sounding for `secs` seconds.
pub fn light(mut app: AppState, midis: Vec<u8>, secs: f64) {
    let token = app.sound_token.cloned().wrapping_add(1);
    app.sound_token.set(token);
    app.sounding.set(midis);
    spawn(async move {
        let wait = format!(
            "await new Promise(r => setTimeout(r, {}));",
            (secs * 1000.0) as u64
        );
        let _ = document::eval(&wait).await;
        // Only the latest sound clears the highlight.
        if app.sound_token.cloned() == token {
            app.sounding.set(Vec::new());
        }
    });
}

/// Equal-tempered frequency of a MIDI note at the current reference pitch.
pub fn et(app: &AppState, m: u8) -> f64 {
    harmony::Midi(m).freq(app.settings.read().a4)
}

/// The whole voicing in the current tuning.
pub fn chord(app: AppState, arpeggio: bool) {
    let (notes, freqs) = match app.analysis.read().as_ref() {
        Some(a) => (a.notes.clone(), a.freqs(app.tuning.cloned()).to_vec()),
        None => return,
    };
    sound(app, notes, &freqs, arpeggio, 3.0);
}

/// Some notes of the voicing, by index (tuning-aware); an empty list plays the whole chord.
pub fn subset(app: AppState, idx: &[usize]) {
    if idx.is_empty() {
        return chord(app, false);
    }
    let (notes, freqs) = match app.analysis.read().as_ref() {
        Some(a) => {
            let f = a.freqs(app.tuning.cloned());
            (
                idx.iter().map(|&i| a.notes[i]).collect(),
                idx.iter().map(|&i| f[i]).collect::<Vec<_>>(),
            )
        }
        None => return,
    };
    sound(app, notes, &freqs, false, 2.2);
}

/// The voicing heard "as" a reading: its root added softly an octave or so below the bass.
pub fn as_reading(app: AppState, root: harmony::PitchClass) {
    let (notes, freqs) = match app.analysis.read().as_ref() {
        Some(a) => (a.notes.clone(), a.freqs(app.tuning.cloned()).to_vec()),
        None => return,
    };
    let bass = notes[0];
    let below = (1..=24u8)
        .map(|d| bass.saturating_sub(d))
        .find(|&m| m % 12 == root.value() && m >= 12 && bass - m >= 5);
    let mut voices: Vec<Voice> = freqs
        .iter()
        .map(|&f| Voice {
            freq: f,
            delay: 0.0,
            level: 1.0,
        })
        .collect();
    let mut lit = notes.clone();
    if let Some(m) = below {
        voices.push(Voice {
            freq: et(&app, m),
            delay: 0.0,
            level: 0.7,
        });
        lit.push(m);
    }
    play_voices(&voices, app.timbre.cloned(), 3.0);
    light(app, lit, 1.8);
}

/// A tone at an arbitrary frequency over the bass (the consonance landscape).
pub fn over_bass(app: AppState, cents: f64) {
    let (bass, f0) = match app.analysis.read().as_ref() {
        Some(a) => (a.notes[0], a.freqs(app.tuning.cloned())[0]),
        None => return,
    };
    let f = f0 * 2f64.powf(cents / 1200.0);
    let nearest = (bass as f64 + cents / 100.0).round().clamp(0.0, 127.0) as u8;
    sound(app, vec![bass, nearest], &[f0, f], false, 2.2);
}

/// MIDI notes in equal temperament (reference chords, clock notes).
pub fn midis(app: AppState, notes: &[u8]) {
    let freqs: Vec<f64> = notes.iter().map(|&m| et(&app, m)).collect();
    sound(app, notes.to_vec(), &freqs, false, 2.5);
}

/// A single MIDI note (equal-tempered).
pub fn note(app: AppState, m: u8) {
    let f = et(&app, m);
    sound(app, vec![m], &[f], false, 1.4);
}
