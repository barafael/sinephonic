// SVG builders: staff notation, pitch-class clocks, waveform, spectrum, consonance landscape.
// Each returns an SVG string; clickable parts carry data-act attributes handled in app.js.

import { esc, clamp, intervalName, pcName } from "./util.js";

// ── Staff ─────────────────────────────────────────────────────────────────────────────

const GAP = 10;
const TREBLE_TOP = 36; // y of F5
const BASS_TOP = 126; // y of A3
const MIDDLE_C = 28;
const ACC = { "-2": "𝄫", "-1": "♭", 1: "♯", 2: "𝄪" };

/** Diatonic step (C0 = 0, 7 per octave) and accidentals of "E♭4", "B𝄫3", "F♯5". */
export function stepOf(name) {
  const m = /^([A-G])(.*?)(-?\d+)$/u.exec(name);
  if (!m) return null;
  let acc = 0;
  for (const ch of m[2]) acc += { "♭": -1, "♯": 1, "𝄫": -2, "𝄪": 2, b: -1, "#": 1 }[ch] ?? 0;
  return { step: "CDEFGAB".indexOf(m[1]) + 7 * Number(m[3]), acc };
}

const yOf = (step) => (step >= MIDDLE_C ? TREBLE_TOP + ((38 - step) * GAP) / 2 : BASS_TOP + ((26 - step) * GAP) / 2);

/** The voicing on a grand staff: whole notes, seconds displaced, accidentals in columns. */
export function staff(names) {
  const notes = names.map(stepOf).filter(Boolean).sort((a, b) => a.step - b.step);
  const x0 = 118;
  const heads = [];
  let prev = null;
  for (const n of notes) {
    const sameStaff = prev && prev.step >= MIDDLE_C === n.step >= MIDDLE_C;
    const shifted = sameStaff && n.step - prev.step === 1 ? !prev.shifted : false;
    heads.push({ x: shifted ? x0 + 13 : x0, y: yOf(n.step), ...n });
    prev = { step: n.step, shifted };
  }
  // Accidental columns from the top note down: a column is free if nothing in it is within a sixth.
  const placed = [];
  const accs = [];
  for (const h of [...heads].reverse()) {
    if (!h.acc) continue;
    let col = 0;
    while (placed.some((p) => p.col === col && Math.abs(p.step - h.step) < 6)) col++;
    placed.push({ step: h.step, col });
    accs.push({ x: x0 - 13 - col * 11, y: h.y, glyph: ACC[h.acc] ?? "" });
  }
  // Ledger lines: even steps between the note and its staff (treble E4–F5, bass G2–A3).
  const ledgers = [];
  for (const h of heads) {
    const [bottom, top] = h.step >= MIDDLE_C ? [30, 38] : [18, 26];
    for (let s = Math.min(h.step, bottom); s <= Math.max(h.step, top); s++) {
      if (s % 2 === 0 && (s < bottom || s > top)) ledgers.push({ x: h.x, y: yOf(s) });
    }
  }
  const lines = [0, 1, 2, 3, 4].flatMap((k) => [TREBLE_TOP + k * GAP, BASS_TOP + k * GAP]);
  return `<svg class="staff" viewBox="0 0 180 200" role="img" data-act="chord" aria-label="Notation: ${esc(names.join(", "))}">
    ${lines.map((y) => `<line class="staff-line" x1="14" x2="172" y1="${y}" y2="${y}"/>`).join("")}
    <line class="staff-line" x1="14" x2="14" y1="${TREBLE_TOP}" y2="${BASS_TOP + 4 * GAP}"/>
    <text class="clef" x="18" y="${TREBLE_TOP + 3 * GAP + 1}" font-size="46">𝄞</text>
    <text class="clef" x="20" y="${BASS_TOP + GAP + 1}" font-size="34">𝄢</text>
    ${ledgers.map((l) => `<line class="staff-line" x1="${l.x - 10}" x2="${l.x + 10}" y1="${l.y}" y2="${l.y}"/>`).join("")}
    ${accs.map((a) => `<text class="accidental" x="${a.x}" y="${a.y + 5}" text-anchor="middle">${a.glyph}</text>`).join("")}
    ${heads.map((h) => `<ellipse class="notehead" cx="${h.x}" cy="${h.y}" rx="6.4" ry="4.6" transform="rotate(-20 ${h.x} ${h.y})"/>`).join("")}
  </svg>`;
}

// ── Pitch-class clock ─────────────────────────────────────────────────────────────────

/** Pitch classes on a circle, chromatic (step 1) or fifths (step 7); root filled, bass ringed. */
export function clock(pcs, root, bass, step, title, spelling) {
  const size = 260;
  const c = size / 2;
  const r = 88;
  const angle = (pc) => (((pc * step) % 12) / 12) * 2 * Math.PI - Math.PI / 2;
  const pos = (pc, rad = r) => [c + rad * Math.cos(angle(pc)), c + rad * Math.sin(angle(pc))];
  const ordered = [...pcs].sort((a, b) => ((a * step) % 12) - ((b * step) % 12));
  const poly = ordered.map((p) => pos(p).map((v) => v.toFixed(1)).join(",")).join(" ");
  let labels = "";
  for (let pc = 0; pc < 12; pc++) {
    const [x, y] = pos(pc, r + 24);
    labels += `<text class="clock-label${pcs.includes(pc) ? " on" : ""}" x="${x.toFixed(1)}" y="${(y + 5).toFixed(1)}" text-anchor="middle" data-act="pc" data-pc="${pc}">${pcName(pc, spelling)}</text>`;
  }
  const dots = pcs
    .map((p) => {
      const [x, y] = pos(p).map((v) => v.toFixed(1));
      return (p === bass ? `<circle class="clock-bass" cx="${x}" cy="${y}" r="13"/>` : "") +
        `<circle class="clock-dot${p === root ? " root" : ""}" cx="${x}" cy="${y}" r="8" data-act="pc" data-pc="${p}"/>`;
    })
    .join("");
  return `<figure class="clock"><svg viewBox="0 0 ${size} ${size}">
      <circle class="clock-ring" cx="${c}" cy="${c}" r="${r}"/>
      ${pcs.length > 1 ? `<polygon class="clock-poly" points="${poly}"/>` : ""}
      ${labels}${dots}
    </svg><figcaption>${esc(title)}</figcaption></figure>`;
}

// ── Waveform ──────────────────────────────────────────────────────────────────────────

function path(x0, width, step, span, mid, amp, f) {
  let d = "";
  const n = Math.ceil(width / step);
  for (let i = 0; i <= n; i++) {
    const x = Math.min(i * step, width);
    const y = mid - amp * f((x / width) * span);
    d += `${i ? "L" : "M"}${(x0 + x).toFixed(2)} ${y.toFixed(2)}`;
  }
  return d;
}

/** One lane per note (highest on top) and the summed wave; dashed lines at each period T. */
export function waveform(a, freqs, periods, width) {
  const LEFT = 120, LANE = 28, SUM_H = 160, PAD = 22, STEP = 0.5;
  const n = a.notes.length;
  const w = Math.max(width, LEFT + 100);
  const h = n * LANE + SUM_H + PAD * 2 + 10;
  const pw = w - LEFT;
  const T = a.periodicity.period;
  const capped = T * periods > 0.25;
  const span = capped ? 0.25 : T * periods;
  const xOf = (t) => LEFT + (t / span) * pw;
  let marks = "";
  for (let k = 0; k * T <= span + 1e-9 && k < 10000; k++) {
    const x = Math.round(xOf(k * T)) + 0.5;
    const label = k === 0 ? "0" : `${k === 1 ? "T" : k + "T"} = ${(k * T * 1000).toFixed(1)} ms`;
    marks += `<line class="marker" x1="${x}" x2="${x}" y1="${PAD - 6}" y2="${h - PAD}"/><text class="mlabel" x="${Math.min(x + 4, w - 110)}" y="${h - 8}">${label}</text>`;
  }
  let lanes = "";
  for (let i = 0; i < n; i++) {
    const mid = PAD + (n - 1 - i) * LANE + LANE / 2;
    const f = freqs[i];
    lanes += `<rect class="hit" x="0" y="${mid - LANE / 2}" width="${w}" height="${LANE}" data-act="notes" data-notes="${i}"/>
      <line class="base" x1="${LEFT}" x2="${w}" y1="${mid}" y2="${mid}"/>
      <path class="lane" d="${path(LEFT, pw, STEP, span, mid, LANE * 0.36, (t) => Math.sin(2 * Math.PI * f * t))}"/>
      <text class="label" x="8" y="${mid + 3.5}">${esc(a.noteNames[i])}  ${f.toFixed(1)} Hz</text>`;
  }
  const smid = PAD + n * LANE + 10 + SUM_H / 2;
  const sum = (t) => freqs.reduce((s, f) => s + Math.sin(2 * Math.PI * f * t), 0) / n;
  const svg = `<svg class="plot" viewBox="0 0 ${w} ${h}" height="${h}">
    ${marks}${lanes}
    <rect class="hit" x="0" y="${smid - SUM_H / 2}" width="${w}" height="${SUM_H}" data-act="chord"/>
    <line class="sum-base" x1="${LEFT}" x2="${w}" y1="${smid}" y2="${smid}"/>
    <path class="sum" d="${path(LEFT, pw, STEP, span, smid, SUM_H * 0.46, sum)}"/>
    <text class="label" x="8" y="${smid + 3.5}">sum</text>
  </svg>`;
  return { svg, capped };
}

// ── Spectrum ──────────────────────────────────────────────────────────────────────────

/** Partials on a log-frequency axis, one lane per note; coinciding partials joined. */
export function spectrum(a, tuning, a4, width) {
  const LEFT = 120, LANE = 56, PAD = 30, AXIS = 30;
  const { partials, coincidences } = a.spectrum;
  const n = a.notes.length;
  const w = Math.max(width, LEFT + 200);
  const h = PAD + n * LANE + AXIS;
  const lo = Math.min(...partials.map((p) => p.freq)) / 1.06;
  const hi = Math.max(...partials.map((p) => p.freq)) * 1.06;
  const xOf = (f) => LEFT + (Math.log2(f / lo) / Math.log2(hi / lo)) * (w - LEFT - 12);
  const base = (note) => PAD + (n - 1 - note) * LANE + LANE - 6;
  let grid = "";
  for (let o = 0; o < 10; o++) {
    const f = a4 * 2 ** ((12 * (o + 1) - 69) / 12);
    if (f < lo || f > hi) continue;
    const x = xOf(f);
    grid += `<line class="grid" x1="${x}" x2="${x}" y1="${PAD - 10}" y2="${h - AXIS + 4}"/><text class="mlabel" x="${x + 3}" y="${h - 10}">C${o}</text>`;
  }
  let lanes = "";
  for (let i = 0; i < n; i++) {
    lanes += `<rect class="hit" x="0" y="${base(i) - LANE + 6}" width="${w}" height="${LANE}" data-act="notes" data-notes="${i}"/>
      <line class="base" x1="${LEFT}" x2="${w}" y1="${base(i)}" y2="${base(i)}"/>
      <text class="label" x="8" y="${base(i) - 4}">${esc(a.noteNames[i])}</text>`;
  }
  const joins = coincidences
    .map((c) => {
      const x = xOf((c.lower.freq + c.upper.freq) / 2);
      const y1 = base(c.upper.note) - LANE + 8;
      const y2 = base(c.lower.note);
      const label = tuning === "et" && c.beat > 0.05 ? `${c.beat.toFixed(1)} Hz` : `h${c.harmonic}`;
      return `<g class="join-group" data-act="notes" data-notes="${c.lower.note},${c.upper.note}">
        <line class="join-hit" x1="${x}" x2="${x}" y1="${y1}" y2="${y2}"/>
        <line class="join" x1="${x}" x2="${x}" y1="${y1}" y2="${y2}"/>
        <text class="jlabel" x="${x + 3}" y="${y1 + 2}">${label}</text></g>`;
    })
    .join("");
  const ticks = partials
    .map((p) => {
      const x = xOf(p.freq);
      return `<line class="partial" x1="${x}" x2="${x}" y1="${base(p.note)}" y2="${base(p.note) - p.amp * (LANE - 10)}"/>`;
    })
    .join("");
  return `<svg class="plot" viewBox="0 0 ${w} ${h}" height="${h}">${grid}${lanes}${joins}${ticks}</svg>`;
}

// ── Consonance landscape ──────────────────────────────────────────────────────────────

const LANDMARKS = [
  ["16/15", 111.73], ["9/8", 203.91], ["6/5", 315.64], ["5/4", 386.31], ["4/3", 498.04], ["7/5", 582.51],
  ["3/2", 701.96], ["8/5", 813.69], ["5/3", 884.36], ["7/4", 968.83], ["15/8", 1088.27],
];

export const LANDSCAPE = { LEFT: 16, RIGHT: 16, TOP: 34, BOTTOM: 46, HEIGHT: 320 };

/** Roughness and harmonic entropy over every interval above the bass, voicing marked. */
export function landscape(a, curve, freqs, maxC, width) {
  const { LEFT, RIGHT, TOP, BOTTOM, HEIGHT } = LANDSCAPE;
  const w = Math.max(width, 400);
  const plotH = HEIGHT - TOP - BOTTOM;
  const peak = Math.max(0.4, ...curve.roughness);
  const cx = (c) => LEFT + (c / maxC) * (w - LEFT - RIGHT);
  const cy = (v) => TOP + (1 - clamp(v, 0, 1)) * plotH;
  const line = (values, scale) =>
    curve.cents.map((c, k) => `${k ? "L" : "M"}${cx(c).toFixed(1)} ${cy(values[k] / scale).toFixed(1)}`).join("");
  const rough = line(curve.roughness, peak);
  const fill = `${rough}L${cx(maxC).toFixed(1)} ${cy(0)}L${cx(0).toFixed(1)} ${cy(0)}Z`;
  let grid = "";
  for (let k = 1; k * 100 <= maxC; k++) {
    grid += `<line class="grid${k % 12 === 0 ? " strong" : ""}" x1="${cx(k * 100)}" x2="${cx(k * 100)}" y1="${TOP}" y2="${HEIGHT - BOTTOM}"/>`;
  }
  let labels = "";
  for (let o = 0; o * 1200 < maxC; o++) {
    for (const [l, c] of LANDMARKS) {
      const cc = c + 1200 * o;
      if (cc < maxC) labels += `<text class="mlabel" x="${cx(cc)}" y="${HEIGHT - BOTTOM + 14}" text-anchor="middle">${l}</text>`;
    }
    if (o >= 1) labels += `<text class="label" x="${cx(1200 * o)}" y="${HEIGHT - BOTTOM + 28}" text-anchor="middle">${o === 1 ? "P8" : o + " oct"}</text>`;
  }
  const steps = curve.cents.length - 1;
  let marks = "";
  for (let i = 1; i < a.notes.length; i++) {
    const c = 1200 * Math.log2(freqs[i] / freqs[0]);
    const k = Math.min(steps, Math.round((c / maxC) * steps));
    const r = curve.roughness[k];
    const he = curve.entropy[k];
    const x = cx(c);
    const name = `${a.noteNames[i]} ${intervalName(a.notes[i] - a.notes[0])}`;
    const values = `r ${r.toFixed(2)} · HE ${he.toFixed(2)}`;
    const right = x > w - 170;
    const tx = right ? x - 5 : x + 5;
    const anchor = right ? ' text-anchor="end"' : "";
    marks += `<line class="marker" x1="${x}" x2="${x}" y1="${TOP - 6}" y2="${HEIGHT - BOTTOM}"/>
      <circle class="he-dot" cx="${x}" cy="${cy(he)}" r="3.5"/>
      <circle class="curve-dot" cx="${x}" cy="${cy(r / peak)}" r="6" data-act="notes" data-notes="0,${i}"/>
      <text class="label" x="${tx}" y="${TOP - 18}"${anchor}>${esc(name)}</text>
      <text class="mlabel" x="${tx}" y="${TOP - 6}"${anchor}>${values}</text>`;
  }
  return `<svg class="plot curve-plot playable" viewBox="0 0 ${w} ${HEIGHT}" height="${HEIGHT}" data-act="landscape" data-max="${maxC}" data-width="${w}">
    ${grid}<path class="rough-fill" d="${fill}"/><path class="he-line" d="${line(curve.entropy, 1)}"/><path class="curve-line" d="${rough}"/>
    <line class="base" x1="${LEFT}" x2="${w - RIGHT}" y1="${HEIGHT - BOTTOM}" y2="${HEIGHT - BOTTOM}"/>
    ${labels}${marks}
  </svg>`;
}

/** Mood-map position in percent, stretched around the centre lines (gain 1.6). */
export function vaPosition(valence, arousal) {
  const stretch = (v) => clamp(50 + (v - 0.5) * 1.6 * 84, 6, 94);
  return [stretch(valence), 100 - stretch(arousal)];
}
