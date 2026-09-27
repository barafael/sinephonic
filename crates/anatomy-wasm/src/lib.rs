//! WebAssembly bindings for the web app. Every export takes plain values (MIDI numbers, an
//! options JSON string) and returns a JSON string, so the analysis crates stay free of serde
//! and the JavaScript side works with ordinary objects (`JSON.parse`).
//!
//! The JSON shapes are built explicitly below; they are the contract with `web/js`.

use anatomy::explore::{dissonance_curve, neighbours, spectrum};
use anatomy::improved::metrics::{parncutt_ambiguity, parncutt_support};
use anatomy::improved::sonority;
use anatomy::information::harmonic_entropy_curve;
use anatomy::{interval_name, Analysis, Model, Options, RatioSet, Spelling, Timbre, Tuning};
use harmony::{Midi, PcSet, PitchClass};
use serde::Deserialize;
use serde_json::{json, Value};
use wasm_bindgen::prelude::*;

/// Options as the web app sends them.
#[derive(Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
struct JsOptions {
    spelling: Option<String>,
    ratio_set: Option<String>,
    a4: Option<f64>,
    timbre: Option<String>,
    tuning: Option<String>,
    model: Option<String>,
}

fn options(json: &str) -> Options {
    let o: JsOptions = serde_json::from_str(json).unwrap_or_default();
    let d = Options::default();
    Options {
        spelling: match o.spelling.as_deref() {
            Some("sharps") => Spelling::Sharps,
            Some("flats") => Spelling::Flats,
            _ => d.spelling,
        },
        ratio_set: match o.ratio_set.as_deref() {
            Some("5-limit") => RatioSet::FiveLimit,
            Some("7-limit") => RatioSet::SevenLimit,
            Some("stolzenburg") => RatioSet::Stolzenburg,
            _ => d.ratio_set,
        },
        a4: o.a4.filter(|a| (380.0..=500.0).contains(a)).unwrap_or(d.a4),
        timbre: match o.timbre.as_deref() {
            Some("sine") => Timbre::Sine,
            Some("harmonic") => Timbre::Harmonic6,
            _ => d.timbre,
        },
        tuning: match o.tuning.as_deref() {
            Some("just") => Tuning::Just,
            Some("et") => Tuning::Et,
            _ => d.tuning,
        },
        model: match o.model.as_deref() {
            Some("prototype") => Model::Prototype,
            Some("improved") => Model::Improved,
            _ => d.model,
        },
    }
}

fn spelling(s: &str) -> Spelling {
    if s == "sharps" {
        Spelling::Sharps
    } else {
        Spelling::Flats
    }
}

/// Full analysis of a voicing, or `null` for fewer than two distinct notes.
#[wasm_bindgen]
pub fn analyze(notes: &[u8], options_json: &str) -> String {
    let opts = options(options_json);
    match anatomy::analyze(notes, &opts) {
        Some(a) => analysis_json(&a, &opts).to_string(),
        None => "null".into(),
    }
}

fn analysis_json(a: &Analysis, opts: &Options) -> Value {
    let p = &a.periodicity;
    let bass_name = a.note_names[0].trim_end_matches(|c: char| c.is_ascii_digit() || c == '-');
    let readings: Vec<Value> = a
        .readings
        .iter()
        .map(|r| {
            json!({
                "root": r.root.value(),
                "rootName": r.root_name,
                "symbol": r.symbol,
                "harte": r.harte,
                "score": r.score,
                "prob": r.prob,
                "rootPresent": r.root_present,
                "degrees": r.degrees.iter().map(|d| d.map(|d| d.label())).collect::<Vec<_>>(),
                "toneNames": r.tone_names,
                "bassRelation": r.bass_relation(a.bass_pc, bass_name),
            })
        })
        .collect();
    let pairs: Vec<Value> = a
        .pairs
        .iter()
        .map(|pr| {
            json!({
                "i": pr.i, "j": pr.j, "semitones": pr.semitones,
                "interval": interval_name(pr.semitones),
                "ratio": opts.ratio_set.ratio(pr.semitones as u32).to_string(),
                "roughness": pr.roughness,
            })
        })
        .collect();
    let beats: Vec<Value> = a
        .beats
        .iter()
        .map(|b| json!({ "i": b.i, "j": b.j, "lowerPartial": b.lower_partial, "upperPartial": b.upper_partial, "rate": b.rate }))
        .collect();
    let tags: Vec<Value> = a
        .tags
        .iter()
        .map(|t| json!({ "word": t.word, "weight": t.weight, "why": t.why, "notes": t.notes }))
        .collect();
    let (parts, hits) = spectrum(a, opts.tuning, opts.timbre);
    let partial = |x: &anatomy::explore::Partial| json!({ "note": x.note, "k": x.k, "freq": x.freq, "amp": x.amp, "harmonic": x.harmonic });

    let set = PcSet::from_midi(&a.notes);
    let support = parncutt_support(set);
    let max_support = support.iter().copied().fold(0.0, f64::max).max(1e-9);
    let ax = &a.axes;

    json!({
        "notes": a.notes,
        "noteNames": a.note_names,
        "pcs": a.pcs.iter().map(|p| p.value()).collect::<Vec<_>>(),
        "bassPc": a.bass_pc.value(),
        "aboveBass": a.notes.iter().map(|&m| interval_name(m - a.notes[0])).collect::<Vec<_>>(),
        "readings": readings,
        "periodicity": {
            "ratios": p.ratios.iter().map(|r| r.to_string()).collect::<Vec<_>>(),
            "harmonics": p.harmonics,
            "bassCycles": p.bass_cycles,
            "bassFreq": p.bass_freq,
            "fundamental": p.fundamental,
            "period": p.period,
            "justFreqs": p.just_freqs,
            "etFreqs": p.et_freqs,
            "cents": p.cents,
        },
        "pairs": pairs,
        "beats": beats,
        "axes": {
            "tension": ax.tension, "harshness": ax.harshness, "aggression": ax.aggression,
            "complexity": ax.complexity, "brightness": ax.brightness, "stability": ax.stability,
            "ambiguity": ax.ambiguity,
        },
        "valence": a.valence,
        "arousal": a.arousal,
        "tags": tags,
        "summary": a.summary(),
        "set": {
            "forte": set.forte().map(|f| f.to_string()).unwrap_or_default(),
            "primeForm": set.prime_form().to_string(),
            "intervalVector": set.interval_vector(),
            "transpositionalSymmetry": set.transpositional_symmetry(),
            "inversionalSymmetry": set.inversional_symmetry(),
            "rootSupport": support.iter().map(|s| s / max_support).collect::<Vec<_>>(),
            "rootAmbiguity": parncutt_ambiguity(&support),
            "sonority": sonority::named(&a.notes),
        },
        "information": a.information.as_ref().map(|i| json!({
            "harmonicEntropy": i.harmonic_entropy,
            "harmonicEntropyMax": i.harmonic_entropy_max,
            "harmonicEntropyNorm": i.harmonic_entropy_norm,
            "descriptionBits": i.description_bits,
            "periodBits": i.period_bits,
            "smoothedPeriodBits": i.smoothed_period_bits,
            "rootEntropy": i.root_entropy,
            "rootEntropyNorm": i.root_entropy_norm,
            "intervalEntropy": i.interval_entropy,
            "intervalEntropyNorm": i.interval_entropy_norm,
            "spectralEntropy": i.spectral_entropy,
            "spectralEntropyMax": i.spectral_entropy_max,
            "complexity": i.complexity,
        })),
        "improved": a.extras.is_some(),
        "spectrum": {
            "partials": parts.iter().map(partial).collect::<Vec<_>>(),
            "coincidences": hits.iter().map(|c| json!({
                "lower": partial(&c.lower), "upper": partial(&c.upper),
                "harmonic": c.harmonic(), "beat": c.beat,
            })).collect::<Vec<_>>(),
        },
    })
}

/// Chords one voice-leading step away.
#[wasm_bindgen]
pub fn voice_leading_neighbours(notes: &[u8], options_json: &str) -> String {
    let opts = options(options_json);
    let n: Vec<Value> = neighbours(notes, &opts)
        .into_iter()
        .map(|x| json!({ "notes": x.notes, "moved": x.moved, "step": x.step, "symbol": x.symbol, "tension": x.tension }))
        .collect();
    Value::Array(n).to_string()
}

/// Valence and arousal only (for the reference dots on the mood map).
#[wasm_bindgen]
pub fn mood(notes: &[u8], options_json: &str) -> String {
    match anatomy::analyze(notes, &options(options_json)) {
        Some(a) => json!({ "valence": a.valence, "arousal": a.arousal }).to_string(),
        None => "null".into(),
    }
}

/// Roughness (relative to C4–D♭4) and normalised harmonic entropy for intervals over `low` Hz,
/// sampled at `steps + 1` points from 0 to `max_cents`: `{cents: [...], roughness: [...],
/// entropy: [...]}`.
#[wasm_bindgen]
pub fn consonance_landscape(low: f64, max_cents: f64, steps: usize, options_json: &str) -> String {
    let opts = options(options_json);
    let rough = dissonance_curve(low, opts.timbre, opts.a4, max_cents, steps);
    let he = harmonic_entropy_curve(max_cents, steps);
    json!({
        "cents": rough.iter().map(|c| c.0).collect::<Vec<_>>(),
        "roughness": rough.iter().map(|c| c.1).collect::<Vec<_>>(),
        "entropy": he.iter().map(|c| c.1).collect::<Vec<_>>(),
    })
    .to_string()
}

/// Note text such as "C3 G3 Ab3 Eb4" or "C G Ab Eb" to sorted, unique MIDI numbers.
#[wasm_bindgen]
pub fn parse_notes(text: &str) -> Vec<u8> {
    harmony::midi::parse_notes(text)
}

/// MIDI numbers to the text-field form "C3 G3 Ab3 Eb4".
#[wasm_bindgen]
pub fn format_notes(notes: &[u8], spelling_name: &str) -> String {
    harmony::midi::format_notes(notes, spelling(spelling_name))
}

/// "E♭4".
#[wasm_bindgen]
pub fn note_name(midi: u8, spelling_name: &str) -> String {
    Midi(midi).name(spelling(spelling_name))
}

/// "E♭" for a pitch class.
#[wasm_bindgen]
pub fn pc_name(pc: u8, spelling_name: &str) -> String {
    spelling(spelling_name)
        .pc_name(PitchClass::new(pc as i32))
        .to_string()
}

/// Equal-tempered frequency.
#[wasm_bindgen]
pub fn midi_freq(midi: u8, a4: f64) -> f64 {
    Midi(midi).freq(a4)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_json_shape() {
        let v: Value = serde_json::from_str(&analyze(
            &[48, 55, 56, 63],
            r#"{"model":"prototype","ratioSet":"5-limit","tuning":"just"}"#,
        ))
        .unwrap();
        assert_eq!(v["readings"][0]["symbol"], "Cm(♭6)");
        assert_eq!(v["periodicity"]["harmonics"], json!([10, 15, 16, 24]));
        assert_eq!(v["summary"], "Grinding and ambiguous");
        assert_eq!(v["readings"].as_array().unwrap().len(), 12);
        assert_eq!(v["readings"][0]["degrees"].as_array().unwrap().len(), 12);
        assert_eq!(v["set"]["forte"], "4-20");
        assert!(v["improved"] == false);
        assert_eq!(analyze(&[60], "{}"), "null");
    }

    #[test]
    fn defaults_and_helpers() {
        let v: Value = serde_json::from_str(&analyze(&[48, 52, 55], "garbage")).unwrap();
        assert_eq!(v["readings"][0]["symbol"], "C");
        assert!(v["improved"] == true);
        assert_eq!(parse_notes("C G Ab Eb"), [48, 55, 56, 63]);
        assert_eq!(format_notes(&[48, 56], "flats"), "C3 Ab3");
        assert_eq!(note_name(63, "sharps"), "D♯4");
        let n: Value =
            serde_json::from_str(&voice_leading_neighbours(&[48, 52, 55], "{}")).unwrap();
        assert_eq!(n.as_array().unwrap().len(), 12);
        let l: Value =
            serde_json::from_str(&consonance_landscape(130.8, 1200.0, 100, "{}")).unwrap();
        assert_eq!(l["cents"].as_array().unwrap().len(), 101);
        let m: Value = serde_json::from_str(&mood(&[48, 52, 55], "{}")).unwrap();
        assert!(m["valence"].as_f64().unwrap() > 0.5);
    }
}
