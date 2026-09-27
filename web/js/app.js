// Chord Anatomy: state, rendering and events. The analysis runs in WebAssembly
// (crates/anatomy-wasm); this file only renders its JSON and plays sounds.

import init, * as wasm from "../pkg/anatomy_wasm.js";
import { playFreqs, playVoices } from "./audio.js";
import { POOL, next } from "./presets.js";
import { staff, waveform, spectrum, landscape, LANDSCAPE } from "./plots.js";
import { sectionHead } from "./util.js";
import * as views from "./views.js";

const REFERENCES = [
  ["maj", [48, 52, 55]], ["min", [48, 51, 55]], ["maj7", [48, 52, 55, 59]], ["m9", [48, 51, 55, 58, 62]],
  ["aug", [48, 52, 56]], ["maj7♯11", [48, 52, 59, 66, 67]], ["13♯11", [48, 52, 58, 62, 66, 69]],
  ["7♯9", [48, 52, 55, 58, 63]], ["dim7", [48, 51, 54, 57]], ["7♭9", [36, 40, 46, 49]],
  ["cluster", [60, 61, 62]], ["quartal", [50, 55, 60, 65]],
];

const state = {
  notes: [48, 55, 56, 63],
  selectedRoot: null,
  tuning: "et",
  timbre: "harmonic",
  windowPeriods: 2,
  settings: { spelling: "flats", ratioSet: "stolzenburg", a4: 440, model: "improved" },
  theme: "auto",
  presetSeed: (Math.random() * 4294967295) >>> 0 || 1,
  presetNotes: [],
  sounding: new Set(),
  showSettings: false,
  midiStatus: null,
};

const $ = (id) => document.getElementById(id);

// ── Derived data (cached by input) ────────────────────────────────────────────────────

const optionsJson = () =>
  JSON.stringify({ ...state.settings, timbre: state.timbre, tuning: state.tuning });

const cache = new Map();
function cached(name, key, compute) {
  const hit = cache.get(name);
  if (hit && hit.key === key) return hit.value;
  const value = compute();
  cache.set(name, { key, value });
  return value;
}

const analysis = () => {
  const key = state.notes.join(",") + optionsJson();
  return cached("analysis", key, () => JSON.parse(wasm.analyze(Uint8Array.from(state.notes), optionsJson())));
};

const freqs = (a) => (state.tuning === "just" ? a.periodicity.justFreqs : a.periodicity.etFreqs);
const et = (m) => wasm.midi_freq(m, state.settings.a4);

// ── State changes ─────────────────────────────────────────────────────────────────────

function setNotes(notes, { play = false } = {}) {
  state.notes = [...new Set(notes)].filter((m) => m >= 0 && m < 128).sort((x, y) => x - y);
  state.selectedRoot = null;
  $("note-input").value = wasm.format_notes(Uint8Array.from(state.notes), state.settings.spelling);
  // Flats only: '#' cannot appear inside a URL fragment.
  const hash = wasm.format_notes(Uint8Array.from(state.notes), "flats").replaceAll(" ", ",");
  history.replaceState(null, "", hash ? `#${hash}` : location.pathname + location.search);
  render();
  if (play) playChord();
}

function commitText() {
  const parsed = Array.from(wasm.parse_notes($("note-input").value));
  if (parsed.join(",") !== state.notes.join(",")) setNotes(parsed);
}

// ── Sound ─────────────────────────────────────────────────────────────────────────────

let soundToken = 0;
function light(midis, secs) {
  const token = ++soundToken;
  state.sounding = new Set(midis);
  paintSounding();
  setTimeout(() => {
    if (token === soundToken) {
      state.sounding = new Set();
      paintSounding();
    }
  }, secs * 1000);
}

function paintSounding() {
  for (const el of document.querySelectorAll("#keyboard [data-midi]")) {
    el.classList.toggle("sounding", state.sounding.has(Number(el.dataset.midi)));
  }
}

function sound(midis, fs, { arpeggio = false, dur = 3 } = {}) {
  const total = playFreqs(fs, state.timbre, { arpeggio, dur });
  light(midis, total * 0.6);
}

function playChord(arpeggio = false) {
  const a = analysis();
  if (a) sound(a.notes, freqs(a), { arpeggio });
  else if (state.notes.length === 1) sound(state.notes, [et(state.notes[0])], { dur: 1.4 });
}

function playIndices(idx) {
  const a = analysis();
  if (!a) return;
  if (!idx.length) return playChord();
  sound(idx.map((i) => a.notes[i]), idx.map((i) => freqs(a)[i]), { dur: 2.2 });
}

/** The voicing heard "as" a reading: its root added softly below the bass. */
function playAsReading(root) {
  const a = analysis();
  if (!a) return;
  const bass = a.notes[0];
  let below = null;
  for (let d = 5; d <= 24; d++) if (bass - d >= 12 && (bass - d) % 12 === root) { below = bass - d; break; }
  const voices = freqs(a).map((freq) => ({ freq }));
  const lit = [...a.notes];
  if (below != null) {
    voices.push({ freq: et(below), level: 0.7 });
    lit.push(below);
  }
  playVoices(voices, state.timbre, 3);
  light(lit, 1.8);
}

function playOverBass(cents) {
  const a = analysis();
  if (!a) return;
  const f0 = freqs(a)[0];
  const nearest = Math.max(0, Math.min(127, Math.round(a.notes[0] + cents / 100)));
  sound([a.notes[0], nearest], [f0, f0 * 2 ** (cents / 1200)], { dur: 2.2 });
}

function playMidis(midis) {
  sound(midis, midis.map(et), { dur: 2.5 });
}

// ── Mood tint and theme ───────────────────────────────────────────────────────────────

/** Faint Oklab tint: cool for dark chords, warm for bright, a touch of red as tension rises. */
function moodBackgrounds(valence, arousal) {
  const ab = (c, h) => [c * Math.cos((h * Math.PI) / 180), c * Math.sin((h * Math.PI) / 180)];
  const cool = ab(0.032, 255), warm = ab(0.03, 80), tense = ab(0.02, 25);
  const v = Math.min(1, Math.max(0, valence));
  const t = Math.min(1, Math.max(0, (arousal - 0.35) / 0.65));
  const a = cool[0] + (warm[0] - cool[0]) * v + tense[0] * t;
  const b = cool[1] + (warm[1] - cool[1]) * v + tense[1] * t;
  return [
    `oklab(0.963 ${(0.001 + 0.4 * a).toFixed(4)} ${(0.006 + 0.4 * b).toFixed(4)})`,
    `oklab(0.18 ${(0.002 + 0.7 * a).toFixed(4)} ${(0.004 + 0.7 * b).toFixed(4)})`,
  ];
}

// ── Rendering ─────────────────────────────────────────────────────────────────────────

function renderHeader(a) {
  const names = a ? a.noteNames : state.notes.map((m) => wasm.note_name(m, state.settings.spelling));
  $("chord-name").textContent = a ? a.readings[0].symbol : names[0] ?? "—";
  $("notes-line").textContent = a ? names.join(" – ") : state.notes.length ? "one note" : "no notes";
  $("summary").textContent = a ? a.summary : "";
  $("staff").innerHTML = staff(names);
  $("settings").innerHTML = state.showSettings ? views.settings(state) : "";
  $("settings-toggle").textContent = state.showSettings ? "close settings" : "settings";
  document.documentElement.dataset.theme = state.theme;
  if (a) {
    const [light, dark] = moodBackgrounds(a.valence, a.arousal);
    document.documentElement.style.setProperty("--mood-light", light);
    document.documentElement.style.setProperty("--mood-dark", dark);
  }
}

function renderVoicing(a) {
  const names = a ? a.noteNames : state.notes.map((m) => wasm.note_name(m, state.settings.spelling));
  const narrow = $("keyboard").clientWidth > 0 && $("keyboard").clientWidth < 640;
  const kb = views.keyboard(state, a, names, narrow);
  $("keyboard").innerHTML = kb.html;
  $("voicing-head").innerHTML = sectionHead("01", "Voicing", `click keys to toggle and hear · C${kb.lo / 12 - 1}–C${kb.hi / 12 - 1}`);
  $("chips").innerHTML = views.chips(state);
}

function renderPlots(a) {
  const fs = freqs(a);
  const wave = waveform(a, fs, state.windowPeriods, $("waveform").clientWidth || 1200);
  $("waveform").innerHTML = wave.svg;
  $("waveform-caption").textContent = views.waveformCaption(state, a, wave.capped);
  $("spectrum").innerHTML = spectrum(a, state.tuning, state.settings.a4, $("spectrum").clientWidth || 1200);

  const span = 1200 * Math.log2(fs[fs.length - 1] / fs[0]);
  const maxC = span <= 1250 ? 1250 : Math.min(span + 100, 3700);
  const curve = cached("landscape", `${fs[0]}|${maxC}|${optionsJson()}`, () =>
    JSON.parse(wasm.consonance_landscape(fs[0], maxC, 900, optionsJson())));
  $("landscape").innerHTML = landscape(a, curve, fs, maxC, $("landscape").clientWidth || 1200);
}

function render() {
  const a = analysis();
  renderHeader(a);
  renderVoicing(a);
  $("empty").hidden = !!a;
  $("analysis").hidden = !a;
  if (!a) return;

  const refs = cached("refs", optionsJson(), () =>
    REFERENCES.map(([label, notes]) => ({ label, notes, ...JSON.parse(wasm.mood(Uint8Array.from(notes), optionsJson())) })));
  const found = cached("neighbours", state.notes.join(",") + optionsJson(), () =>
    JSON.parse(wasm.voice_leading_neighbours(Uint8Array.from(state.notes), optionsJson())));
  state.neighbours = found;
  state.references = refs;

  $("readings").innerHTML = views.readings(state, a);
  $("intervals").innerHTML = views.intervals(state, a, freqs(a));
  $("pitch-space").innerHTML = views.pitchSpace(state, a);
  $("matrix").innerHTML = views.matrix(state, a);
  $("character").innerHTML = views.character(state, a, refs);
  $("information").innerHTML = views.information(a);
  $("landscape-head").innerHTML = sectionHead("08", "Consonance landscape", `every interval above ${a.noteNames[0]} · click to hear any of them`);
  $("waveform-headline").innerHTML = sectionHead("09", "Waveform", "click a lane to hear it");
  $("waveform-head").innerHTML = views.waveformHead(state, a);
  $("spectrum-headline").innerHTML = sectionHead("10", "Spectrum", "click a lane or a beat to hear it");
  $("spectrum-caption").textContent = views.spectrumCaption(state, a);
  $("neighbours").innerHTML = views.neighbours(a, found);
  renderPlots(a);
}

// ── Events ────────────────────────────────────────────────────────────────────────────

const indices = (s) => (s ? s.split(",").filter(Boolean).map(Number) : []);

const actions = {
  key(el) {
    const m = Number(el.dataset.midi);
    const adding = !state.notes.includes(m);
    setNotes(adding ? [...state.notes, m] : state.notes.filter((x) => x !== m));
    if (adding) sound([m], [et(m)], { dur: 1.4 });
  },
  preset(el) {
    setNotes(Array.from(wasm.parse_notes(POOL[Number(el.dataset.k)][1])), { play: true });
  },
  shuffle() {
    state.presetSeed = next(state.presetSeed);
    renderVoicing(analysis());
  },
  clear() {
    setNotes([]);
  },
  midi() {
    connectMidi();
  },
  play() {
    playChord(false);
  },
  arpeggiate() {
    playChord(true);
  },
  chord() {
    playChord(false);
  },
  reading(el) {
    state.selectedRoot = Number(el.dataset.root);
    render();
    playAsReading(state.selectedRoot);
  },
  rootbar(el) {
    actions.reading(el);
  },
  notes(el) {
    playIndices(indices(el.dataset.notes));
  },
  pc(el) {
    playMidis([60 + Number(el.dataset.pc)]);
  },
  ref(el) {
    playMidis(REFERENCES[Number(el.dataset.k)][1]);
  },
  neighbour(el) {
    setNotes(state.neighbours[Number(el.dataset.k)].notes, { play: true });
  },
  settings() {
    state.showSettings = !state.showSettings;
    renderHeader(analysis());
  },
  seg(el) {
    const { name, value } = el.dataset;
    if (name === "timbre") state.timbre = value;
    else if (name === "tuning") state.tuning = value;
    else if (name === "window") state.windowPeriods = Number(value);
    else if (name === "theme") state.theme = value;
    else if (name === "spelling") {
      state.settings.spelling = value;
      $("note-input").value = wasm.format_notes(Uint8Array.from(state.notes), value);
    } else state.settings[name] = value;
    render();
  },
  landscape(el, event) {
    const rect = el.getBoundingClientRect();
    const w = Number(el.dataset.width);
    const x = ((event.clientX - rect.left) / rect.width) * w;
    const { LEFT, RIGHT } = LANDSCAPE;
    const maxC = Number(el.dataset.max);
    playOverBass(Math.min(maxC, Math.max(0, ((x - LEFT) / (w - LEFT - RIGHT)) * maxC)));
  },
};

document.addEventListener("click", (event) => {
  const el = event.target.closest("[data-act]");
  if (!el || el.disabled) return;
  actions[el.dataset.act]?.(el, event);
});

document.addEventListener("input", (event) => {
  if (event.target.dataset.input === "a4") {
    state.settings.a4 = Number(event.target.value);
    render();
  }
});

// ── MIDI input: held keys set the voicing; releasing keeps the chord ──────────────────

async function connectMidi() {
  const status = (s) => {
    state.midiStatus = s;
    $("chips").innerHTML = views.chips(state);
  };
  if (!navigator.requestMIDIAccess) return status("Web MIDI is not available in this browser");
  try {
    status("MIDI: waiting for browser permission…");
    const access = await navigator.requestMIDIAccess();
    const held = new Set();
    const names = [];
    const hook = (input) => {
      names.push(input.name);
      input.onmidimessage = (e) => {
        const [st, note, vel] = e.data;
        const cmd = st & 0xf0;
        if (cmd === 0x90 && vel > 0) {
          held.add(note);
          setNotes([...held]);
        } else if (cmd === 0x80 || (cmd === 0x90 && vel === 0)) {
          held.delete(note);
        }
      };
    };
    access.inputs.forEach(hook);
    access.onstatechange = (e) => {
      if (e.port.type === "input" && e.port.state === "connected" && !e.port.onmidimessage) {
        hook(e.port);
        status(`MIDI: ${names.join(", ")}`);
      }
    };
    status(names.length ? `MIDI: ${names.join(", ")}` : "MIDI: no input devices yet");
  } catch {
    status("MIDI access denied");
  }
}

// ── Start ─────────────────────────────────────────────────────────────────────────────

function notesFromHash() {
  const text = decodeURIComponent(location.hash.slice(1)).replaceAll(",", " ");
  return text ? Array.from(wasm.parse_notes(text)) : [];
}

await init();
state.presetNotes = POOL.map(([, text]) => Array.from(wasm.parse_notes(text)).join(","));

const input = $("note-input");
input.addEventListener("keydown", (e) => {
  if (e.key === "Enter") commitText();
});
input.addEventListener("blur", commitText);

window.addEventListener("hashchange", () => {
  const notes = notesFromHash();
  if (notes.length && notes.join(",") !== state.notes.join(",")) setNotes(notes);
});

let resizeTimer = 0;
new ResizeObserver(() => {
  clearTimeout(resizeTimer);
  resizeTimer = setTimeout(() => {
    const a = analysis();
    renderVoicing(a);
    if (a) renderPlots(a);
  }, 120);
}).observe(document.body);

const initial = notesFromHash();
setNotes(initial.length ? initial : state.notes);
