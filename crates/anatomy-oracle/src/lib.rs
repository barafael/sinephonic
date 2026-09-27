//! Runs the design prototype's JavaScript (`js/prototype.js`, copied verbatim from the
//! `<script data-dc-script>` block of `Chord Anatomy.dc.html`) in the Boa engine, so tests can
//! compare `anatomy::reference` with the original on any voicing.

use anatomy::{Options, Timbre, Tuning};
use boa_engine::{js_string, Context, JsValue, Source};
use harmony::{RatioSet, Spelling};
use serde_json::Value;

const PROTOTYPE: &str = include_str!("../js/prototype.js");

/// Serialises everything the UI reads from `analyze()`. `Set`s and sparse objects become arrays.
const WRAPPER: &str = r#"
function oracle(notes, accidentals, lim, a4, timbre, tuning) {
  const N = accidentals === 'sharps' ? SH : FL;
  const a = analyze(notes, {N, lim, A4: a4, timbre, tuning});
  const keys = [...Array(12).keys()];
  return JSON.stringify({
    ints: a.ints, P: a.P, f0: a.f0, period: a.period,
    justF: a.justF, etF: a.etF, cents: a.cents,
    rats: a.rats.map(r => r.join('/')),
    all: a.all.map(r => ({root: r.root, name: r.name, score: r.score, p: r.p,
      rootPresent: r.rootPresent, alts: r.alts, deg: keys.map(i => r.deg[i] ?? null)})),
    pairs: a.pairs.map(p => [p.i, p.j, p.d, p.r]),
    tags: a.tags.map(t => [t.w, t.k, t.why]),
    axes: a.axes, valence: a.valence, arousal: a.arousal,
  });
}
"#;

pub struct Oracle {
    ctx: Context,
}

impl Default for Oracle {
    fn default() -> Self {
        Self::new()
    }
}

impl Oracle {
    pub fn new() -> Self {
        let mut ctx = Context::default();
        ctx.eval(Source::from_bytes(PROTOTYPE))
            .expect("prototype.js evaluates");
        ctx.eval(Source::from_bytes(WRAPPER))
            .expect("wrapper evaluates");
        Self { ctx }
    }

    /// The prototype's `analyze(notes, opts)` as JSON. `notes` must be sorted and unique.
    /// The prototype only knows the 5- and 7-limit ratio sets.
    pub fn analyze(&mut self, notes: &[u8], opts: &Options) -> Value {
        let list = notes
            .iter()
            .map(u8::to_string)
            .collect::<Vec<_>>()
            .join(",");
        let call = format!(
            "oracle([{list}], '{}', '{}', {}, '{}', '{}')",
            match opts.spelling {
                Spelling::Flats => "flats",
                Spelling::Sharps => "sharps",
            },
            match opts.ratio_set {
                RatioSet::FiveLimit => "5-limit",
                RatioSet::SevenLimit => "7-limit",
                RatioSet::Stolzenburg => panic!("the prototype has no Stolzenburg set"),
            },
            opts.a4,
            match opts.timbre {
                Timbre::Sine => "sine",
                Timbre::Harmonic6 => "harmonic",
            },
            match opts.tuning {
                Tuning::Just => "just",
                Tuning::Et => "et",
            },
        );
        let v: JsValue = self
            .ctx
            .eval(Source::from_bytes(&call))
            .expect("oracle call");
        let s = v
            .as_string()
            .expect("JSON string")
            .to_std_string()
            .expect("valid UTF-16");
        let _ = js_string!("");
        serde_json::from_str(&s).expect("valid JSON")
    }
}
