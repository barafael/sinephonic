//! Musical algebra for chord analysis.
//!
//! * [`pc`]: pitch classes (ℤ/12) and pitch-class sets with Tₙ/TₙI, prime form, interval
//!   vectors and Forte names.
//! * [`spelled`]: spelled pitches and intervals as line-of-fifths coordinates (after
//!   Temperley and DCMLab's `pitchtypes`), so C°7 is C E♭ G♭ B𝄫 rather than C E♭ G♭ A.
//! * [`ratio`]: exact rationals for just intonation, with Tenney height and Euler's gradus.
//! * [`midi`]: MIDI numbers, frequencies, note names and the note-text parser.
//!
//! No dependencies and no UI: it compiles anywhere, including wasm.

mod forte_table;
pub mod midi;
pub mod pc;
pub mod ratio;
pub mod spelled;

pub use midi::{Midi, Spelling};
pub use pc::{PcSet, PitchClass};
pub use ratio::{Ratio, RatioSet};
pub use spelled::{IntervalQuality, SpelledInterval, SpelledPc};
