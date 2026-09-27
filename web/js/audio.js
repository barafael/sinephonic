// WebAudio playback. Envelope 0.0001 → 1 over 20 ms, exponential decay over `dur`; master gain
// 0.35/√n; the 6-partial timbre uses harmonics 1–6 at 0.88^(k−1)/k.

let ctx = null;
let harmonicWave = null;

function context() {
  if (!ctx) ctx = new (window.AudioContext || window.webkitAudioContext)();
  ctx.resume();
  return ctx;
}

function wave(c) {
  if (!harmonicWave) {
    const re = new Float32Array(7);
    const im = new Float32Array(7);
    for (let k = 1; k < 7; k++) im[k] = 0.88 ** (k - 1) / k;
    harmonicWave = c.createPeriodicWave(re, im);
  }
  return harmonicWave;
}

/**
 * Plays voices `{freq, delay, level}`; `timbre` is "sine" or "harmonic".
 * Returns the total duration in seconds.
 */
export function playVoices(voices, timbre, dur = 3) {
  const c = context();
  const now = c.currentTime + 0.02;
  const master = c.createGain();
  master.gain.value = 0.35 / Math.sqrt(Math.max(1, voices.length));
  master.connect(c.destination);
  let last = 0;
  for (const { freq, delay = 0, level = 1 } of voices) {
    const o = c.createOscillator();
    if (timbre === "harmonic") o.setPeriodicWave(wave(c));
    else o.type = "sine";
    o.frequency.value = freq;
    const g = c.createGain();
    const t0 = now + delay;
    g.gain.setValueAtTime(0.0001, t0);
    g.gain.exponentialRampToValueAtTime(level, t0 + 0.02);
    g.gain.exponentialRampToValueAtTime(0.001, t0 + dur);
    o.connect(g).connect(master);
    o.start(t0);
    o.stop(t0 + dur + 0.1);
    last = Math.max(last, delay);
  }
  return last + dur;
}

/** Frequencies together, or 0.32 s apart when `arpeggio`. */
export function playFreqs(freqs, timbre, { arpeggio = false, dur = 3 } = {}) {
  return playVoices(
    freqs.map((freq, i) => ({ freq, delay: arpeggio ? i * 0.32 : 0 })),
    timbre,
    dur,
  );
}
