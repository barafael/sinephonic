# sinephonic: Chord Anatomy

A one-screen chord-voicing analyser, built from the design handoff in
`Chord Anatomy_ Interactive Analysis.zip`, in Dioxus 0.7. It shows:

- the root readings and chord symbols
- the interval of each note above the bass, with its function
- just ratios, harmonic numbers and the chord period
- pairwise roughness
- character tags and axes on a valence × arousal map
- each note's waveform and the summed waveform, with playback
- the partial spectrum on the chord's harmonic series, with 12-TET beat rates
- a consonance landscape: Sethares' roughness and Erlich's harmonic entropy for every interval above the bass, with the voicing's intervals marked
- information measures in bits (harmonic entropy, description length, period, root, interval and spectral entropy) and the complexity score built from them
- voice-leading neighbours: every chord one semitone or tone away, clickable
- pitch-class space: chromatic and circle-of-fifths clocks, set class, symmetry, the interval-class vector, and Parncutt's root support for every candidate root

Almost everything plays when clicked: keys, presets, readings (over their root), interval rows,
matrix cells, tags (the notes they're about), spectrum beats, any point of the consonance
landscape, waveform lanes, clock notes and the reference chords on the mood map. Sounding keys
light up. The header shows the voicing on a grand staff, spelled as analysed. The page tint
follows the chord's mood (valence, arousal), faintly, in light and dark themes. Presets are a
random handful from a pool of about 90 voicings in all keys (**shuffle** for more). The layout
uses the full width of large screens and becomes one long page on phones.

12-TET is the default tuning; just intonation is a toggle. The voicing is kept in the URL
fragment (`#C3,G3,Ab3,Eb4`), so you can share a chord or edit it in the address bar.
**connect MIDI** takes input from a MIDI keyboard through Web MIDI; I haven't tested it with
hardware.

## Layout

```
src/                    Dioxus app (web; desktop via the `desktop` feature)
assets/main.css         design tokens and styles from the handoff
crates/harmony/         musical algebra, no dependencies
crates/anatomy/         analysis engine, depends only on harmony
crates/anatomy-cli/     `anatomy` terminal tool for judging voicings
crates/anatomy-oracle/  test-only: runs the prototype's JavaScript in Boa
```

The analysis crates contain no Dioxus code.

### harmony

- `pc`: pitch classes (ℤ/12) and pitch-class sets as bitmasks, with Tₙ/TₙI, prime form (Rahn), interval vectors, Forte names (with Z) and symmetry.
- `spelled`: spelled pitches and intervals as line-of-fifths coordinates (fifths, octaves) ∈ ℤ², after Temperley and DCMLab `pitchtypes`. Qualities d/m/P/M/A and chord-tone spelling (C°7 = C E♭ G♭ B𝄫).
- `ratio`: exact rationals, the 5-limit, 7-limit and Stolzenburg ratio sets, common harmonics, Tenney height and Euler's gradus.
- `midi`: frequencies, note names and the note-text parser from the handoff.

### anatomy

It has two models with one output type:

- **Prototype** (`reference.rs`): an exact port of the handoff's JS. `anatomy-oracle` checks it field by field against the original script over thousands of voicings.
- **Improved** (default, `improved/`):
  - chord tones spelled on the line of fifths
  - a structured symbol builder: 7♭5, 13, 9sus4, ø9, power chords, hybrid slash chords like F/G, and no more `A°(maj7)♭7/C`
  - Harte labels
  - root scores combining template fit with Parncutt's root support, the bass, Hindemith's interval roots and a dominant-chord prior
  - complexity from Stolzenburg's smoothed periodicity
  - roughness measured as the excess over the unison baseline, and tension that doesn't saturate (Huron dissonance plus mean roughness)
  - brightness from the chord tones' line-of-fifths positions (the axis that orders the modes from Locrian to Lydian); valence and arousal computed independently, so all four mood quadrants are populated (`cargo run -p anatomy --example quadrants`)
  - tag explanations checked against the voicing: inversion, tritone direction, register and tuning (`tests/claims.rs`)
  - set class, named sonorities, Huron consonance and Cook–Fujisawa tension and modality
  - *symmetric* and *bittersweet* tags

  The weights in `improved/reading.rs` are fitted to `crates/anatomy/tests/corpus.txt` by `cargo run --release -p anatomy --example tune`.

## Commands

```sh
dx serve --platform web                              # the app
cargo test --workspace --exclude sinephonic
cargo run -p anatomy-cli -- C3 G3 Ab3 Eb4            # full report
cargo run -p anatomy-cli -- --compare "C E G Bb D#"  # prototype vs improved
cargo run -p anatomy-cli -- corpus                   # judge the corpus
```

The desktop build (`dx serve --platform desktop`) needs webkit2gtk-4.1.

## Adding voicings to the corpus

Each line of `crates/anatomy/tests/corpus.txt` is one voicing:

```
E3 G3 C4          = C/E
Eb3 G3 Bb3 C4     = E♭6   ~ Cm7/E♭   ! both are standard
C3 E3 G3          = C     @ fused
```

- `=` gives the expected best reading.
- `~` lists other acceptable best readings.
- `@` lists tags that must appear.
- `!` marks the entry as disputed: it is reported but never fails.
