// Small shared helpers.

export const esc = (s) =>
  String(s).replace(/[&<>"']/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;", "'": "&#39;" })[c]);

export const clamp = (x, lo, hi) => Math.min(hi, Math.max(lo, x));

/** "E♭4" → "E♭". */
export const stripOctave = (name) => name.replace(/-?\d+$/, "");

export const PC_FLATS = ["C", "D♭", "D", "E♭", "E", "F", "G♭", "G", "A♭", "A", "B♭", "B"];
export const PC_SHARPS = ["C", "C♯", "D", "D♯", "E", "F", "F♯", "G", "G♯", "A", "A♯", "B"];
export const pcName = (pc, spelling) => (spelling === "sharps" ? PC_SHARPS : PC_FLATS)[((pc % 12) + 12) % 12];
export const isBlack = (m) => [1, 3, 6, 8, 10].includes(m % 12);

/** A segmented toggle: options are [value, label] pairs; clicks dispatch data-act="seg". */
export function segmented(name, options, value) {
  const buttons = options
    .map(([v, label]) => `<button class="${v === value ? "on" : ""}" data-act="seg" data-name="${name}" data-value="${esc(v)}">${esc(label)}</button>`)
    .join("");
  return `<div class="segmented">${buttons}</div>`;
}

/** Numbered section heading. */
export function sectionHead(num, title, hint = "") {
  return `<div class="section-head"><span class="num">${num}</span><span>${esc(title)}</span><span class="rule"></span>${
    hint ? `<span class="hint">${esc(hint)}</span>` : ""
  }</div>`;
}

export function intervalName(d) {
  const simple = ["P1", "m2", "M2", "m3", "M3", "P4", "TT", "P5", "m6", "M6", "m7", "M7"];
  const compound = ["P8", "m9", "M9", "m10", "M10", "P11", "A11", "P12", "m13", "M13", "m14", "M14", "P15"];
  if (d < 12) return simple[d];
  if (d <= 24) return compound[d - 12];
  return `${simple[d % 12]}+${Math.floor(d / 12)}oct`;
}
