//! Playback through WebAudio. The same script runs in the browser and in the desktop webview.

use anatomy::Timbre;
use dioxus::prelude::*;

/// Envelope 0.0001 → 1 over 20 ms, → 0.001 by 3 s; master gain 0.35/√n; the 6-partial timbre
/// uses harmonics 1–6 at 0.88^(k−1)/k; arpeggio steps are 0.32 s.
const SCRIPT: &str = r#"
(() => {
  const fr = __FREQS__, arp = __ARP__, harmonic = __HARMONIC__;
  const ctx = window.__chordAnatomyCtx || (window.__chordAnatomyCtx = new (window.AudioContext || window.webkitAudioContext)());
  ctx.resume();
  const now = ctx.currentTime + 0.02;
  const master = ctx.createGain();
  master.gain.value = 0.35 / Math.sqrt(fr.length);
  master.connect(ctx.destination);
  let wave = null;
  if (harmonic) {
    const re = new Float32Array(7), im = new Float32Array(7);
    for (let k = 1; k < 7; k++) im[k] = 0.88 ** (k - 1) / k;
    wave = ctx.createPeriodicWave(re, im);
  }
  fr.forEach((f, i) => {
    const o = ctx.createOscillator();
    if (wave) o.setPeriodicWave(wave); else o.type = 'sine';
    o.frequency.value = f;
    const g = ctx.createGain(), t0 = now + (arp ? i * 0.32 : 0);
    g.gain.setValueAtTime(0.0001, t0);
    g.gain.exponentialRampToValueAtTime(1, t0 + 0.02);
    g.gain.exponentialRampToValueAtTime(0.001, t0 + 3);
    o.connect(g).connect(master);
    o.start(t0);
    o.stop(t0 + 3.1);
  });
})();
"#;

pub fn play(freqs: &[f64], arpeggio: bool, timbre: Timbre) {
    let list = freqs
        .iter()
        .map(|f| format!("{f:.6}"))
        .collect::<Vec<_>>()
        .join(",");
    let js = SCRIPT
        .replace("__FREQS__", &format!("[{list}]"))
        .replace("__ARP__", if arpeggio { "true" } else { "false" })
        .replace(
            "__HARMONIC__",
            if timbre == Timbre::Harmonic6 {
                "true"
            } else {
                "false"
            },
        );
    let _ = document::eval(&js);
}
