//! Chord-voicing analysis: root readings and chord symbols, just ratios and periodicity,
//! roughness, character axes and tags.
//!
//! Two models share one output type:
//! * [`Model::Prototype`] is an exact port of the design prototype (`reference`), checked
//!   against the prototype's own JavaScript by the `anatomy-oracle` crate.
//! * [`Model::Improved`] adds correct spelling, a structured symbol builder, psychoacoustic
//!   root salience (Parncutt), Stolzenburg periodicity and more (`improved`).
//!
//! Everything here is pure and UI-free.

pub mod corpus;
pub mod degree;
pub mod explore;
pub mod improved;
pub mod information;
pub mod periodicity;
pub mod reference;
pub mod roughness;
pub mod waveform;

pub use degree::{Degree, Seventh, Third};
pub use harmony::{PcSet, PitchClass, RatioSet, SpelledPc, Spelling};
pub use periodicity::{Beat, Periodicity};
pub use roughness::Timbre;

/// Frequencies used for roughness, the Hz column, the waveform and playback.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Tuning {
    Just,
    #[default]
    Et,
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Model {
    Prototype,
    #[default]
    Improved,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Options {
    pub spelling: Spelling,
    pub ratio_set: RatioSet,
    pub a4: f64,
    pub timbre: Timbre,
    pub tuning: Tuning,
    pub model: Model,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            spelling: Spelling::Flats,
            ratio_set: RatioSet::Stolzenburg,
            a4: 440.0,
            timbre: Timbre::Harmonic6,
            tuning: Tuning::Et,
            model: Model::Improved,
        }
    }
}

impl Options {
    /// The prototype's defaults: its model and the 5-limit ratio set.
    pub fn prototype() -> Self {
        Self {
            model: Model::Prototype,
            ratio_set: RatioSet::FiveLimit,
            tuning: Tuning::Just,
            ..Self::default()
        }
    }
}

/// One way of hearing the chord: a root and what every note does above it.
#[derive(Clone, PartialEq, Debug)]
pub struct Reading {
    pub root: PitchClass,
    /// The root as spelled in the symbol.
    pub root_name: String,
    /// Display symbol, e.g. "Cm(♭6)", "A♭maj7/C".
    pub symbol: String,
    /// Harte et al. (2005) label, e.g. "C:min(b6)", "Ab:maj7/3".
    pub harte: String,
    pub score: f64,
    /// Softmax of the scores over all 12 readings.
    pub prob: f64,
    pub root_present: bool,
    pub third: Third,
    pub seventh: Option<Seventh>,
    pub perfect_fifth: bool,
    pub diminished: bool,
    pub augmented: bool,
    /// Number of alterations in the symbol (♭9, ♯9, ♯11, ♭13, ♯5…).
    pub alterations: usize,
    /// Pitch classes relative to the root.
    pub rel: PcSet,
    /// Function of each interval class above the root.
    pub degrees: [Option<Degree>; 12],
    /// Spelled name of the chord tone at each interval class above the root.
    pub tone_names: [Option<String>; 12],
}

impl Reading {
    pub fn degree_of(&self, pc: PitchClass) -> Option<Degree> {
        self.degrees[self.root.up_to(pc) as usize]
    }

    pub fn tone_name(&self, pc: PitchClass) -> &str {
        self.tone_names[self.root.up_to(pc) as usize]
            .as_deref()
            .unwrap_or("?")
    }

    /// "root in bass", "root inside · C in bass" or "rootless · implied F".
    pub fn bass_relation(&self, bass: PitchClass, bass_name: &str) -> String {
        if !self.root_present {
            format!("rootless · implied {}", self.root_name)
        } else if self.root == bass {
            "root in bass".into()
        } else {
            format!("root inside · {bass_name} in bass")
        }
    }
}

/// A pair of notes, i below j.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Pair {
    pub i: usize,
    pub j: usize,
    pub semitones: u8,
    /// Roughness relative to C4–D♭4, 0..1.
    pub roughness: f64,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Axes {
    pub tension: f64,
    pub harshness: f64,
    pub aggression: f64,
    pub complexity: f64,
    pub brightness: f64,
    pub stability: f64,
    pub ambiguity: f64,
}

impl Axes {
    /// In display order.
    pub fn labelled(&self) -> [(&'static str, f64); 7] {
        [
            ("Tension", self.tension),
            ("Harshness", self.harshness),
            ("Aggression", self.aggression),
            ("Complexity", self.complexity),
            ("Brightness", self.brightness),
            ("Stability", self.stability),
            ("Ambiguity", self.ambiguity),
        ]
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Tag {
    pub word: &'static str,
    pub weight: f64,
    pub why: String,
}

/// Metrics only the improved model computes.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Extras {
    /// Forte name of the set class, e.g. "4-27".
    pub forte: String,
    pub prime_form: String,
    pub interval_vector: [u8; 6],
    /// Named sonority, if the voicing is one ("Mystic chord", "So What chord"…).
    pub sonority: Option<&'static str>,
    /// Parncutt root ambiguity √(Σw / max w): 1 = one clear root.
    pub root_ambiguity: f64,
    /// Stolzenburg smoothed log₂ periodicity.
    pub smoothed_periodicity: f64,
    /// Huron (1994) aggregate dyadic consonance.
    pub huron: f64,
    /// Tenney height of the chord h₀:h₁:…, log₂(h₀·h₁·…).
    pub tenney: f64,
    /// Euler's gradus suavitatis of the chord.
    pub gradus: u32,
    /// Cook & Fujisawa (2006) triad tension and modality (major +, minor −).
    pub cook_tension: f64,
    pub cook_modality: f64,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Analysis {
    pub notes: Vec<u8>,
    /// Distinct pitch classes, in order of first appearance from the bass up.
    pub pcs: Vec<PitchClass>,
    pub bass_pc: PitchClass,
    pub periodicity: Periodicity,
    /// All 12 readings, best first.
    pub readings: Vec<Reading>,
    pub pairs: Vec<Pair>,
    /// Beating of coincident partials in the current tuning and timbre (all 0 when just).
    pub beats: Vec<periodicity::Beat>,
    pub axes: Axes,
    pub valence: f64,
    pub arousal: f64,
    /// Best first.
    pub tags: Vec<Tag>,
    /// Display name of each note, e.g. "E♭4".
    pub note_names: Vec<String>,
    pub extras: Option<Extras>,
    /// Entropy and information measures (see [`information`]).
    pub information: Option<information::Information>,
}

impl Analysis {
    pub fn best(&self) -> &Reading {
        &self.readings[0]
    }

    pub fn reading(&self, root: PitchClass) -> Option<&Reading> {
        self.readings.iter().find(|r| r.root == root)
    }

    /// Roughness of the pair (i, j), i < j.
    pub fn pair(&self, i: usize, j: usize) -> Option<&Pair> {
        self.pairs.iter().find(|p| p.i == i && p.j == j)
    }

    /// "Grinding and ambiguous": the first two tags.
    pub fn summary(&self) -> String {
        let words: Vec<&str> = self.tags.iter().take(2).map(|t| t.word).collect();
        let s = words.join(" and ");
        let mut c = s.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    }

    /// The fastest beat, if any pair beats audibly (> 0.05 Hz).
    pub fn fastest_beat(&self) -> Option<&periodicity::Beat> {
        self.beats
            .iter()
            .filter(|b| b.rate > 0.05)
            .max_by(|a, b| a.rate.total_cmp(&b.rate))
    }

    /// Frequencies for the current tuning.
    pub fn freqs(&self, tuning: Tuning) -> &[f64] {
        match tuning {
            Tuning::Just => &self.periodicity.just_freqs,
            Tuning::Et => &self.periodicity.et_freqs,
        }
    }
}

/// Analyses a voicing. `notes` are MIDI numbers in any order (they are sorted and
/// deduplicated). Returns `None` for fewer than two distinct notes.
pub fn analyze(notes: &[u8], opts: &Options) -> Option<Analysis> {
    let notes = harmony::midi::normalize(notes.iter().map(|&m| m as i32));
    if notes.len() < 2 {
        return None;
    }
    Some(match opts.model {
        Model::Prototype => reference::analyze(&notes, opts),
        Model::Improved => improved::analyze(&notes, opts),
    })
}

/// Highest partial number that can beat: the timbre's partial count.
pub(crate) fn max_partial(timbre: Timbre) -> u64 {
    match timbre {
        Timbre::Sine => 1,
        Timbre::Harmonic6 => 6,
    }
}

pub(crate) fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

/// Distinct pitch classes in order of first appearance.
pub(crate) fn distinct_pcs(notes: &[u8]) -> Vec<PitchClass> {
    let mut out: Vec<PitchClass> = Vec::new();
    for &m in notes {
        let pc = PitchClass::of_midi(m);
        if !out.contains(&pc) {
            out.push(pc);
        }
    }
    out
}

/// Interval names by semitone distance: P1…M7, then P8…P15, then "m3+2oct".
pub fn interval_name(d: u8) -> String {
    const SIMPLE: [&str; 12] = [
        "P1", "m2", "M2", "m3", "M3", "P4", "TT", "P5", "m6", "M6", "m7", "M7",
    ];
    const COMPOUND: [&str; 13] = [
        "P8", "m9", "M9", "m10", "M10", "P11", "A11", "P12", "m13", "M13", "m14", "M14", "P15",
    ];
    match d {
        0..=11 => SIMPLE[d as usize].into(),
        12..=24 => COMPOUND[d as usize - 12].into(),
        _ => format!("{}+{}oct", SIMPLE[(d % 12) as usize], d / 12),
    }
}
