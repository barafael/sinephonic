// A large pool of named voicings in many keys; the Voicing section shows a random handful.

export const POOL = [
  ["C–G–A♭–E♭", "C3 G3 Ab3 Eb4"],
  ["C7♯9", "C3 E3 G3 Bb3 Eb4"],
  ["C7♯9♯5", "C3 E3 Ab3 Bb3 Eb4"],
  ["Quartal", "D3 G3 C4 F4"],
  ["Cluster", "C4 Db4 D4"],
  ["E major", "E3 G#3 B3"],
  ["A♭ major", "Ab2 C3 Eb3"],
  ["B major, open", "B1 F#2 D#3 B3"],
  ["D/F♯", "F#2 D3 A3"],
  ["G/D (six-four)", "D3 G3 B3"],
  ["B♭ minor", "Bb2 Db3 F3"],
  ["F♯ minor", "F#3 A3 C#4"],
  ["E♭m/G♭", "Gb2 Bb2 Eb3"],
  ["G♯ minor", "G#2 B2 D#3"],
  ["F♯ diminished", "F#3 A3 C4"],
  ["A♭ augmented", "Ab3 C4 E4"],
  ["Fm/C (six-four)", "C3 F3 Ab3"],
  ["Dsus4", "D3 G3 A3"],
  ["Esus2", "E3 F#3 B3"],
  ["A5 power chord", "A1 E2 A2"],
  ["Dm7", "D3 F3 A3 C4"],
  ["G7", "G2 F3 B3 D4"],
  ["E♭maj7", "Eb3 G3 Bb3 D4"],
  ["A♭maj7/C", "C3 Eb3 G3 Ab3"],
  ["Bø7", "B2 A3 D4 F4"],
  ["C♯°7", "C#3 E3 G3 Bb3"],
  ["Fm(maj7)", "F3 Ab3 C4 E4"],
  ["A7♭5", "A2 C#3 Eb3 G3"],
  ["B♭7/A♭", "Ab2 Bb2 D3 F3"],
  ["Em7/D", "D3 E3 G3 B3"],
  ["F♯7sus4", "F#2 B2 C#3 E3"],
  ["D♭maj7♯5", "Db3 F3 A3 C4"],
  ["Gm6", "G2 D3 E3 Bb3"],
  ["E6", "E2 B2 C#3 G#3"],
  ["Dm9 (rootless)", "F3 A3 C4 E4"],
  ["G13", "G2 F3 B3 E4"],
  ["Cmaj9", "C3 E3 B3 D4 G4"],
  ["Fmaj7♯11", "F2 E3 A3 B3"],
  ["B♭13♯11", "Bb2 Ab3 D4 E4 G4"],
  ["E♭m11", "Eb3 Db4 Gb4 Ab4 Bb4"],
  ["A7alt", "A2 G3 C#4 F4 Bb4"],
  ["D7♭9", "D3 F#3 C4 Eb4 A4"],
  ["E7♯9 (Hendrix)", "E2 G#2 D3 G3"],
  ["G7♭9♭13", "G2 F3 B3 Eb4 Ab4"],
  ["C6/9", "C3 E3 A3 D4 G4"],
  ["F♯m11", "F#2 E3 A3 B3 C#4"],
  ["So What chord", "E3 A3 D4 G4 B4"],
  ["Mu major (Steely Dan)", "F3 G3 A3 C4"],
  ["Upper structure D/C7", "C3 E3 Bb3 D4 F#4 A4"],
  ["B♭9sus4", "Bb2 Eb3 F3 Ab3 C4"],
  ["F/G", "G2 F3 A3 C4"],
  ["A♭maj13", "Ab2 G3 C4 F4 Bb4"],
  ["Dø9", "D3 C4 F4 Ab4 E5"],
  ["Kenny Barron minor", "D3 A3 E4 F4 C5 G5"],
  ["B7♯11", "B2 A3 D#4 E#4 F#4"],
  ["Cm(maj9)", "C3 Eb3 B3 D4 G4"],
  ["Tristan chord", "F3 B3 D#4 G#4"],
  ["Scriabin’s mystic chord", "C3 F#3 Bb3 E4 A4 D5"],
  ["Petrushka chord", "C3 E3 G3 F#4 A#4 C#5"],
  ["Farben chord", "C3 G#3 B3 E4 A4"],
  ["Elektra chord", "E3 G#3 B3 Db4 F4 Ab4"],
  ["Neapolitan sixth", "F3 Ab3 Db4"],
  ["Italian sixth", "Ab2 C3 F#3"],
  ["French sixth", "Ab2 C3 D3 F#3"],
  ["German sixth", "Ab2 C3 Eb3 F#3"],
  ["Cadential six-four", "G2 C3 E3 G3"],
  ["Dream chord (La Monte Young)", "G3 C4 C#4 D4"],
  ["Major-minor (Bartók)", "C3 Eb3 E3 G3"],
  ["Lydian stack", "C3 G3 D4 F#4 B4"],
  ["Stacked fifths", "C3 G3 D4 A4 E5"],
  ["Stacked fourths, five", "E2 A2 D3 G3 C4"],
  ["Whole-tone", "C3 D3 E3 F#3 G#3 A#3"],
  ["Augmented, spread", "E2 C3 G#3"],
  ["Diminished, spread", "A2 Eb3 C4 F#4"],
  ["Octatonic tetrad", "C3 Db3 E3 G3"],
  ["Messiaen-like", "C3 F#3 Bb3 E4 G#4 D5"],
  ["Chromatic cluster", "G3 G#3 A3 A#3 B3"],
  ["Tone cluster", "D4 E4 F#4 G#4"],
  ["Minor second, low", "C2 Db2"],
  ["Perfect fifth, low", "C2 G2"],
  ["Octave", "A2 A3"],
  ["Tritone", "B3 F4"],
  ["Pentatonic cluster", "C4 D4 E4 G4 A4"],
  ["Hexatonic (augmented scale)", "C3 Eb3 E3 G3 Ab3 B3"],
  ["Film-score minor", "A2 E3 C4 B4"],
  ["Sus stack", "D3 A3 E4 G4"],
  ["Gospel 9", "Eb3 Db4 F4 G4 Bb4"],
];

/** xorshift step (32-bit, never 0). */
export function next(seed) {
  let x = seed >>> 0 || 1;
  x ^= x << 13; x >>>= 0;
  x ^= x >>> 17;
  x ^= x << 5; x >>>= 0;
  return x || 1;
}

/** `count` distinct pool indices chosen by `seed` (partial Fisher–Yates). */
export function pick(seed, count) {
  const idx = POOL.map((_, i) => i);
  let s = seed;
  for (let k = 0; k < Math.min(count, idx.length); k++) {
    s = next(s);
    const j = k + (s % (idx.length - k));
    [idx[k], idx[j]] = [idx[j], idx[k]];
  }
  return idx.slice(0, count);
}
