// Section renderers: each takes the app state and derived data and returns an HTML string.
// Clickable elements carry data-act attributes; app.js dispatches them.

import { esc, clamp, pcName, isBlack, segmented, sectionHead, stripOctave } from "./util.js";
import { clock, vaPosition } from "./plots.js";
import { POOL, pick } from "./presets.js";

const pct = (x) => `${(x * 100).toFixed(1)}%`;

/** The reading shown in section 03: the clicked one, or the best. */
export const selectedReading = (a, root) => (root == null ? a.readings[0] : a.readings.find((r) => r.root === root) ?? a.readings[0]);

const degreeOf = (r, pc) => r.degrees[(pc - r.root + 12) % 12];
const toneOf = (r, pc) => r.toneNames[(pc - r.root + 12) % 12] ?? "?";

// ── 01 Voicing ────────────────────────────────────────────────────────────────────────

/** Keyboard range: C2–C6 (C3–C5 narrow), widened by octaves to include every note. */
export function keyRange(notes, narrow) {
  let [lo, hi] = narrow ? [48, 72] : [36, 84];
  if (notes.length) {
    lo = Math.min(lo, Math.floor(notes[0] / 12) * 12);
    hi = Math.min(127, Math.max(hi, Math.ceil(notes[notes.length - 1] / 12) * 12));
  }
  return [lo, hi];
}

export function keyboard(state, a, names, narrow) {
  const [lo, hi] = keyRange(state.notes, narrow);
  const sel = a ? selectedReading(a, state.selectedRoot) : null;
  const whites = [];
  for (let m = lo; m <= hi; m++) if (!isBlack(m)) whites.push(m);
  const wn = whites.length;
  const label = (m) => {
    const i = state.notes.indexOf(m);
    return i < 0 ? null : names[i];
  };
  const degree = (m) => (sel && state.notes.includes(m) ? degreeOf(sel, m % 12) : null);
  const cls = (base, m) => base + (state.notes.includes(m) ? " on" : "") + (state.sounding.has(m) ? " sounding" : "");
  let html = "";
  for (const m of whites) {
    const d = degree(m);
    const l = label(m);
    html += `<div class="${cls("white-key", m)}" data-act="key" data-midi="${m}">${d ? `<span class="kdeg">${esc(d)}</span>` : ""}${
      l ? `<span>${esc(l)}</span>` : m % 12 === 0 ? `<span>C${m / 12 - 1}</span>` : ""
    }</div>`;
  }
  for (let m = lo; m <= hi; m++) {
    if (!isBlack(m)) continue;
    const idx = whites.indexOf(m - 1);
    const left = ((idx + 1) / wn - 0.3 / wn) * 100;
    const d = degree(m);
    const l = label(m);
    html += `<div class="${cls("black-key", m)}" style="left:${left}%;width:${(0.6 / wn) * 100}%" data-act="key" data-midi="${m}">${
      d ? `<span class="kdeg">${esc(d)}</span>` : ""
    }${l ? `<span class="klabel">${esc(l)}</span>` : ""}</div>`;
  }
  return { html, lo, hi };
}

export function chips(state) {
  const key = state.notes.join(",");
  const presets = pick(state.presetSeed, 10)
    .map((k) => {
      const [name, text] = POOL[k];
      return `<button class="chip${state.presetNotes[k] === key ? " on" : ""}" title="${esc(text)}" data-act="preset" data-k="${k}">${esc(name)}</button>`;
    })
    .join("");
  const midi = state.midiStatus
    ? `<span class="midi-status">${esc(state.midiStatus)}</span>`
    : `<button class="clear" data-act="midi">connect MIDI</button>`;
  return `${presets}<button class="clear" title="show other presets" data-act="shuffle">shuffle</button><button class="clear" data-act="clear">clear</button>${midi}`;
}

// ── 02 Readings ───────────────────────────────────────────────────────────────────────

export function readings(state, a) {
  const sel = selectedReading(a, state.selectedRoot);
  const top = a.readings[0].prob;
  const rows = a.readings
    .slice(0, 5)
    .map((r) => {
      const tones = a.pcs
        .map((pc) => `<span class="tone"><span class="dot"></span><span>${esc(toneOf(r, pc))}</span><span class="muted">${esc(degreeOf(r, pc) ?? "")}</span></span>`)
        .join("");
      return `<div class="reading${r.root === sel.root ? " on" : ""}" data-act="reading" data-root="${r.root}">
        <div class="reading-name">${esc(r.symbol)}</div>
        <div class="prob"><div class="prob-track"><div class="prob-fill" style="width:${pct(r.prob / top)}"></div></div><span class="prob-pct">${Math.round(r.prob * 100)}%</span></div>
        <div class="tones">${tones}<span class="sub">${esc(r.bassRelation)}</span></div>
      </div>`;
    })
    .join("");
  const bars = a.set.rootSupport
    .map((v, r) => {
      const cls = r === sel.root ? "bar chosen" : a.pcs.includes(r) ? "bar present" : "bar";
      return `<button class="${cls}" title="hear the chord over this root" data-act="rootbar" data-root="${r}">
        <span class="bar-track"><span class="bar-fill" style="height:${(v * 100).toFixed(0)}%"></span></span>
        <span class="bar-label">${pcName(r, state.settings.spelling)}</span></button>`;
    })
    .join("");
  const note = a.improved
    ? "Each of the 12 pitch classes is tried as root. Fit rises when the root is present or in the bass, when a clear 3rd, 5th and 7th exist, with psychoacoustic root support (Parncutt) and when the root is the root of the voicing’s strongest interval (Hindemith); it falls with alterations, a missing 3rd or a missing root. Root support adds Parncutt’s weights for the tones above each candidate (unison 10, fifth 5, major 3rd 3, minor 7th 2, major 2nd 1). Click a reading to relabel the intervals and hear it over its root."
    : "Each of the 12 pitch classes is tried as root. Fit rises when the root is present or in the bass and a clear 3rd, 5th and 7th exist; it falls with alterations, a missing 3rd or a missing root. Click a reading to relabel the intervals and hear it over its root.";
  return `${sectionHead("02", "Readings")}
    <div class="readings">${rows}</div>
    <div class="ps-chart"><div class="ps-title">Root support for every candidate root (Parncutt) · click to hear</div><div class="bars bars-12">${bars}</div></div>
    <p class="footnote">${note}</p>`;
}

// ── 03 Intervals ──────────────────────────────────────────────────────────────────────

const cents = (c) => `${c >= 0 ? "+" : "−"}${Math.abs(c).toFixed(1)}¢`;

export function intervals(state, a, freqs) {
  const sel = selectedReading(a, state.selectedRoot);
  const p = a.periodicity;
  let rows = "";
  for (let i = a.notes.length - 1; i >= 0; i--) {
    const d = a.notes[i] - a.notes[0];
    rows += `<div class="iv-row" title="click to hear with the bass" data-act="notes" data-notes="${i === 0 ? "0" : "0," + i}">
      <div class="td note"><span class="dot dot-lg"></span>${esc(a.noteNames[i])}</div>
      <div class="td deg">${esc(degreeOf(sel, a.notes[i] % 12) ?? "")}</div>
      <div class="td">${i === 0 ? "bass" : esc(a.aboveBass[i])}<span class="muted"> · ${d} st</span></div>
      <div class="td r">${esc(p.ratios[i])}</div>
      <div class="td r muted">${i === 0 ? "0.0¢" : cents(p.cents[i])}</div>
      <div class="td r">${freqs[i].toFixed(1)}</div></div>`;
  }
  return `${sectionHead("03", "Intervals", `functions as ${sel.symbol}`)}
    <div class="scroll-x"><div class="iv-table">
      <div class="th">Note</div><div class="th">Degree</div><div class="th">Above bass</div>
      <div class="th r">Just</div><div class="th r">TET−just</div><div class="th r">Hz</div>${rows}
    </div></div>`;
}

// ── 04 Pitch-class space ──────────────────────────────────────────────────────────────

const IC_NAMES = ["m2", "M2", "m3", "M3", "P4", "TT"];

export function pitchSpace(state, a) {
  const sel = selectedReading(a, state.selectedRoot);
  const s = a.set;
  const sp = state.settings.spelling;
  const t = s.transpositionalSymmetry > 1 ? `T${12 / s.transpositionalSymmetry}` : null;
  const mirror = s.inversionalSymmetry > 0 ? "mirror" : null;
  const symmetry = [t, mirror].filter(Boolean).join(" · ") || "none";
  const ivMax = Math.max(1, ...s.intervalVector);
  const bars = s.intervalVector
    .map((count, k) => {
      const ex = a.pairs.find((p) => Math.min(p.semitones % 12, 12 - (p.semitones % 12)) === k + 1);
      return `<button class="bar${count ? " present" : ""}" title="hear an example" ${ex ? `data-act="notes" data-notes="${ex.i},${ex.j}"` : "disabled"}>
        <span class="bar-track"><span class="bar-fill" style="height:${((count / ivMax) * 100).toFixed(0)}%"></span></span>
        <span class="bar-count">${count}</span><span class="bar-label">${IC_NAMES[k]}</span></button>`;
    })
    .join("");
  const fact = (k, v) => `<div class="ps-fact"><span class="k">${k}</span><span class="v">${esc(v)}</span></div>`;
  return `${sectionHead("04", "Pitch-class space", "click to hear")}
    <div class="identity-row">
      ${clock(a.pcs, sel.root, a.bassPc, 1, "chromatic circle", sp)}
      ${clock(a.pcs, sel.root, a.bassPc, 7, "circle of fifths", sp)}
    </div>
    <div class="ps-facts">
      ${fact("Set class", s.forte)}${fact("Prime form", s.primeForm)}${fact("Symmetry", symmetry)}
      ${fact("Root ambiguity", s.rootAmbiguity.toFixed(2))}${s.sonority ? fact("Known as", s.sonority) : ""}
    </div>
    <div class="ps-charts"><div class="ps-chart"><div class="ps-title">Interval-class vector ⟨${s.intervalVector.join("")}⟩</div><div class="bars bars-6">${bars}</div></div></div>
    <p class="footnote">Symmetry: Tn means transposing by n semitones maps the set onto itself; mirror means some inversion does. Root ambiguity is Parncutt’s √(Σ support / max support). The interval vector counts every pair of pitch classes by interval class; click a bar to hear one.</p>`;
}

// ── 05 Inter-relations ────────────────────────────────────────────────────────────────

export function matrix(state, a) {
  const n = a.notes.length;
  let cells = `<div class="cell"></div>`;
  for (let i = 0; i < n; i++) cells += `<div class="cell playable" data-act="notes" data-notes="${i}"><span class="t" style="font-size:13px">${esc(a.noteNames[i])}</span><span class="s"></span></div>`;
  for (let i = 0; i < n; i++) {
    cells += `<div class="cell playable" style="background:var(--sel)" data-act="notes" data-notes="${i}"><span class="t" style="font-size:13px">${esc(a.noteNames[i])}</span><span class="s"></span></div>`;
    for (let j = 0; j < n; j++) {
      if (i === j) {
        cells += `<div class="cell" style="background:var(--diag);color:var(--muted-2)"><span class="t" style="font-size:13px">·</span><span class="s"></span></div>`;
      } else if (j > i) {
        const p = a.pairs.find((q) => q.i === i && q.j === j);
        const r = p.roughness;
        cells += `<div class="cell playable rough${r > 0.55 ? " hot" : ""}" style="--r:${r.toFixed(3)};background:var(--ramp);color:var(--ramp-fg)" data-act="notes" data-notes="${i},${j}">
          <span class="t" style="font-size:15px">${esc(p.interval)}</span><span class="s">r ${r.toFixed(2)}</span></div>`;
      } else {
        const p = a.pairs.find((q) => q.i === j && q.j === i);
        const b = a.beats.find((q) => q.i === j && q.j === i && q.rate > 0.05);
        cells += `<div class="cell playable" style="color:var(--muted)" data-act="notes" data-notes="${j},${i}">
          <span class="t" style="font-size:12px">${esc(p.ratio)}</span><span class="s">${b ? `beats ${b.rate.toFixed(1)} Hz` : ""}</span></div>`;
      }
    }
  }
  const caption = state.tuning === "et" ? "Upper: interval + roughness · Lower: just ratio + 12-TET beating" : "Upper: interval + roughness · Lower: just ratio";
  return `${sectionHead("05", "Inter-relations")}
    <div class="caption-row"><span class="muted">${caption}</span>
      <div class="toggle-label"><span class="muted">tone model</span>${segmented("timbre", [["sine", "sine"], ["harmonic", "6 partials"]], state.timbre)}</div></div>
    <div class="scroll-x"><div class="matrix" style="grid-template-columns:56px repeat(${n},minmax(0,96px))">${cells}</div></div>
    <div class="legend"><span>smooth</span><div class="ramp"></div><span>rough</span><span class="src">Sethares/Plomp–Levelt, relative to C4–D♭4</span></div>`;
}

// ── 06 Character ──────────────────────────────────────────────────────────────────────

const AXES = [
  ["Tension", "tension"], ["Harshness", "harshness"], ["Aggression", "aggression"], ["Complexity", "complexity"],
  ["Brightness", "brightness"], ["Stability", "stability"], ["Ambiguity", "ambiguity"],
];

export function character(state, a, refs) {
  const tags = a.tags
    .slice(0, 5)
    .map((t) => `<div class="tag playable" title="click to hear the notes this is about" data-act="notes" data-notes="${t.notes.join(",")}">
      <div class="w">${esc(t.word)}</div><div class="why">${esc(t.why)}</div></div>`)
    .join("");
  const axes = AXES.map(([label, k]) => {
    const v = a.axes[k];
    return `<div class="axis"><span>${label}</span><div class="axis-track"><div class="axis-fill" style="width:${pct(v)}"></div></div><span class="v">${v.toFixed(2)}</span></div>`;
  }).join("");
  const dots = refs
    .map((r, k) => {
      const [x, y] = vaPosition(r.valence, r.arousal);
      return `<div class="ref-dot playable" style="left:${x.toFixed(1)}%;top:${y.toFixed(1)}%" title="click to hear" data-act="ref" data-k="${k}"><span class="d"></span><span class="l">${esc(r.label)}</span></div>`;
    })
    .join("");
  const [x, y] = vaPosition(a.valence, a.arousal);
  return `${sectionHead("06", "Character")}
    <div class="tags">${tags}</div>
    <div class="character-lower">
      <div class="axes">${axes}</div>
      <div class="va"><div class="va-mid"><div class="va-box">
        <div class="va-v"></div><div class="va-h"></div>
        <span class="va-label in tl">dark · tense</span><span class="va-label in tr">bright · tense</span>
        <span class="va-label in bl">dark · calm</span><span class="va-label in br">bright · calm</span>
        ${dots}<div class="va-dot playable" style="left:${x.toFixed(1)}%;top:${y.toFixed(1)}%" data-act="chord"></div>
      </div></div></div>
    </div>`;
}

// ── 07 Information ────────────────────────────────────────────────────────────────────

export function information(a) {
  const i = a.information;
  const rows = [
    ["Harmonic entropy", `${i.harmonicEntropy.toFixed(2)} bits`, i.harmonicEntropyNorm, "How unsure the ear is which simple ratio each interval stands for (Erlich). Mean over the pairs; low for 3/2 or 5/4, high between them."],
    ["Description length", `${i.descriptionBits.toFixed(1)} bits`, Math.min(1, i.descriptionBits / 40), "Bits needed to write the chord as harmonics h₀ : h₁ : … of one fundamental: Σ log₂ hᵢ, the chord’s Tenney height."],
    ["Period", `${i.smoothedPeriodBits.toFixed(2)} bits`, Math.min(1, i.smoothedPeriodBits / 7), "log₂ of the common period in cycles, averaged over every note as reference (Stolzenburg’s smoothed periodicity)."],
    ["Root entropy", `${i.rootEntropy.toFixed(2)} / ${Math.log2(12).toFixed(2)} bits`, i.rootEntropyNorm, "Shannon entropy of the 12 root readings: 0 when one root is certain, 3.58 when all are equally likely."],
    ["Interval entropy", `${i.intervalEntropy.toFixed(2)} / ${Math.log2(6).toFixed(2)} bits`, i.intervalEntropyNorm, "Variety of interval classes in the set: stacked equal intervals are low, all-interval chords reach the maximum."],
    ["Spectral entropy", `${i.spectralEntropy.toFixed(2)} / ${i.spectralEntropyMax.toFixed(2)} bits`, i.spectralEntropyMax > 0 ? i.spectralEntropy / i.spectralEntropyMax : 0, "Entropy of all partials after merging those within 20¢: coinciding partials fuse, which lowers it."],
  ]
    .map(([label, value, fill, why]) => `<div class="info-row"><span class="info-label">${label}</span>
      <div class="axis-track"><div class="axis-fill" style="width:${pct(clamp(fill, 0, 1))}"></div></div>
      <span class="info-value">${value}</span><span class="info-why">${why}</span></div>`)
    .join("");
  return `${sectionHead("07", "Information", "entropy and complexity, in bits")}
    <div class="info">
      <div class="info-score"><span class="k">Complexity</span><span class="v">${i.complexity.toFixed(2)}</span>
        <span class="muted">0.3 harmonic entropy + 0.25 period + 0.2 root entropy + 0.15 interval entropy + 0.1 size, each scaled to 0–1.</span></div>
      <div class="info-rows">${rows}</div>
    </div>`;
}

// ── 09 Waveform (stats and captions; the plot comes from plots.js) ────────────────────

export function waveformHead(state, a) {
  const p = a.periodicity;
  const approx = state.tuning === "et" ? "≈ " : "";
  const ms = p.period * 1000;
  const stats = [
    ["Harmonic ratio (just)", p.harmonics.join(" : ")],
    ["Common fundamental", `${approx}${p.fundamental.toFixed(2)} Hz`],
    ["Period T", `${approx}${ms < 10 ? ms.toFixed(2) : ms.toFixed(1)} ms`],
    ["Bass cycles per T", `${p.bassCycles} × ${a.noteNames[0]}`],
  ];
  const fastest = fastestBeat(a);
  if (state.tuning === "et") stats.push(["Fastest beat", fastest ? `${fastest.rate.toFixed(1)} Hz · ${a.noteNames[fastest.i]}–${a.noteNames[fastest.j]}` : "none"]);
  return `<div class="stats">${stats.map(([k, v]) => `<div class="stat"><span class="k">${k}</span><span class="v">${esc(v)}</span></div>`).join("")}</div>
    <div class="toggles">${segmented("tuning", [["just", "Just"], ["et", "12-TET"]], state.tuning)}${segmented("window", [["1", "1T"], ["2", "2T"], ["4", "4T"]], String(state.windowPeriods))}</div>`;
}

export function fastestBeat(a) {
  return a.beats.filter((b) => b.rate > 0.05).sort((x, y) => y.rate - x.rate)[0] ?? null;
}

export function waveformCaption(state, a, capped) {
  let text;
  if (state.tuning === "just") {
    text = "Just intonation: every note is an integer multiple of the common fundamental, so the sum repeats exactly at each dashed line.";
  } else {
    const b = fastestBeat(a);
    text = b
      ? `12-TET: ratios are irrational, so the sum never repeats exactly. Coinciding partials beat instead: fastest is ${a.noteNames[b.i]}–${a.noteNames[b.j]} at ${b.rate.toFixed(1)} Hz (partial ${b.lowerPartial} of ${a.noteNames[b.i]} against partial ${b.upperPartial} of ${a.noteNames[b.j]}). Dashed lines mark the just period it approximates; the drift is the cents column in 03.`
      : "12-TET: ratios are irrational, so the sum never repeats exactly. Dashed lines mark the just period it approximates; the drift is the cents column in 03.";
  }
  return text + (capped ? " View capped at 250 ms." : "");
}

export function spectrumCaption(state, a) {
  const hits = a.spectrum.coincidences;
  if (state.tuning === "just") {
    return `Just intonation: ${hits.length} pairs of partials land exactly on shared harmonics of the ${a.periodicity.fundamental.toFixed(2)} Hz fundamental (labelled with the harmonic number), so they fuse instead of beating.`;
  }
  const beating = hits.filter((c) => c.beat > 0.05).length;
  return `12-TET: of ${hits.length} partial pairs that would coincide in just intonation, ${beating} are pulled apart and beat at the rates shown. Slow beats read as warmth or chorus; beats around 15–40 Hz as roughness.`;
}

// ── 11 Neighbours ─────────────────────────────────────────────────────────────────────

export function neighbours(a, found) {
  const here = a.axes.tension;
  let rows = "";
  for (let i = a.notes.length - 1; i >= 0; i--) {
    rows += `<div class="td note"><span class="dot dot-lg"></span>${esc(a.noteNames[i])}</div>`;
    for (const step of [-2, -1, 1, 2]) {
      const k = found.findIndex((n) => n.moved === i && n.step === step);
      if (k < 0) {
        rows += `<div class="nb-cell empty-cell">—</div>`;
        continue;
      }
      const n = found[k];
      const delta = n.tension - here;
      const word = delta > 0.03 ? "tenser" : delta < -0.03 ? "calmer" : "same tension";
      rows += `<button class="nb-cell" title="${word} (${n.tension.toFixed(2)})" data-act="neighbour" data-k="${k}">
        <span class="nb-sym">${esc(n.symbol)}</span><span class="nb-bar"><span class="nb-fill" style="width:${(n.tension * 100).toFixed(0)}%"></span></span>
        <span class="nb-delta">${word}</span></button>`;
    }
  }
  return `${sectionHead("11", "Neighbours", "move one note · click to go there and hear it")}
    <div class="scroll-x"><div class="nb-table">
      <div class="th">Note</div><div class="th c">↓ tone</div><div class="th c">↓ semitone</div><div class="th c">↑ semitone</div><div class="th c">↑ tone</div>${rows}
    </div></div>
    <p class="footnote">Parsimonious voice leading: the smoothest moves between chords change one note by a step (Cohn, Tymoczko). The bar is each neighbour’s tension; the word compares it with the current chord.</p>`;
}

// ── Settings popover ──────────────────────────────────────────────────────────────────

export function settings(state) {
  const s = state.settings;
  return `<div class="settings">
    <label>ratio set</label>${segmented("ratioSet", [["5-limit", "5-limit"], ["7-limit", "7-limit"], ["stolzenburg", "Stolzenburg"]], s.ratioSet)}
    <label>A4 = ${s.a4} Hz</label><input type="range" min="415" max="466" step="1" value="${s.a4}" data-input="a4">
    <label>spelling</label>${segmented("spelling", [["flats", "flats"], ["sharps", "sharps"]], s.spelling)}
    <label>model</label>${segmented("model", [["improved", "improved"], ["prototype", "prototype"]], s.model)}
    <label>theme</label>${segmented("theme", [["auto", "auto"], ["light", "light"], ["dark", "dark"]], state.theme)}
    <p class="note">Improved: spelled chord tones, Parncutt root support, Hindemith’s interval roots and Stolzenburg periodicity. Prototype: the design handoff’s original heuristics.</p>
  </div>`;
}

export { stripOctave };
