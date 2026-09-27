//! App state: the voicing, view toggles and settings, shared through context.

use anatomy::{Analysis, Model, Options, PitchClass, RatioSet, Spelling, Timbre, Tuning};
use dioxus::prelude::*;
use harmony::midi::{format_notes, normalize, parse_notes};

pub const DEFAULT_NOTES: [u8; 4] = [48, 55, 56, 63];

/// Reference chords on the valence × arousal map.
/// Reference chords on the valence × arousal map, a few per quadrant.
pub const REFERENCES: [(&str, &[u8]); 12] = [
    ("maj", &[48, 52, 55]),
    ("min", &[48, 51, 55]),
    ("maj7", &[48, 52, 55, 59]),
    ("m9", &[48, 51, 55, 58, 62]),
    ("aug", &[48, 52, 56]),
    ("maj7♯11", &[48, 52, 59, 66, 67]),
    ("13♯11", &[48, 52, 58, 62, 66, 69]),
    ("7♯9", &[48, 52, 55, 58, 63]),
    ("dim7", &[48, 51, 54, 57]),
    ("7♭9", &[36, 40, 46, 49]),
    ("cluster", &[60, 61, 62]),
    ("quartal", &[50, 55, 60, 65]),
];

/// The settings the handoff calls "Tweaks", plus the model choice.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Settings {
    pub spelling: Spelling,
    pub ratio_set: RatioSet,
    pub a4: f64,
    pub model: Model,
}

impl Default for Settings {
    fn default() -> Self {
        let o = Options::default();
        Self {
            spelling: o.spelling,
            ratio_set: o.ratio_set,
            a4: o.a4,
            model: o.model,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppState {
    pub notes: Signal<Vec<u8>>,
    pub text: Signal<String>,
    pub selected_root: Signal<Option<PitchClass>>,
    pub tuning: Signal<Tuning>,
    pub timbre: Signal<Timbre>,
    pub window_periods: Signal<u32>,
    pub settings: Signal<Settings>,
    pub analysis: Memo<Option<Analysis>>,
    /// MIDI notes currently sounding (lit on the keyboard).
    pub sounding: Signal<Vec<u8>>,
    /// Incremented per sound so only the latest one clears the highlight.
    pub sound_token: Signal<u64>,
    pub theme: Signal<Theme>,
    /// Chooses which presets are on show; re-rolled by "shuffle".
    pub preset_seed: Signal<u64>,
}

/// Colour scheme: follow the system, or force light or dark.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Theme {
    Auto,
    Light,
    Dark,
}

impl Theme {
    pub fn attr(self) -> &'static str {
        match self {
            Theme::Auto => "auto",
            Theme::Light => "light",
            Theme::Dark => "dark",
        }
    }
}

impl AppState {
    /// Creates the state and provides it to the component tree.
    pub fn provide() -> Self {
        let settings = use_signal(Settings::default);
        let notes = use_signal(|| DEFAULT_NOTES.to_vec());
        let text = use_signal(|| format_notes(&DEFAULT_NOTES, settings.peek().spelling));
        let tuning = use_signal(|| Tuning::Et);
        let timbre = use_signal(|| Timbre::Harmonic6);
        let state = Self {
            notes,
            text,
            selected_root: use_signal(|| None),
            tuning,
            timbre,
            window_periods: use_signal(|| 2),
            settings,
            sounding: use_signal(Vec::new),
            sound_token: use_signal(|| 0),
            theme: use_signal(|| Theme::Auto),
            preset_seed: use_signal(|| 1),
            analysis: use_memo(move || {
                let opts = options(&settings(), tuning(), timbre());
                anatomy::analyze(&notes.read(), &opts)
            }),
        };
        use_context_provider(|| state)
    }

    /// Replaces the voicing, rewrites the text field and resets the selected reading.
    pub fn set_notes(&mut self, notes: impl IntoIterator<Item = u8>) {
        let notes = normalize(notes.into_iter().map(i32::from));
        self.text
            .set(format_notes(&notes, self.settings.peek().spelling));
        self.notes.set(notes);
        self.selected_root.set(None);
    }

    pub fn toggle(&mut self, m: u8) {
        let mut v = self.notes.peek().clone();
        match v.iter().position(|&x| x == m) {
            Some(i) => {
                v.remove(i);
            }
            None => v.push(m),
        }
        self.set_notes(v);
    }

    pub fn commit_text(&mut self) {
        let parsed = parse_notes(&self.text.peek());
        self.set_notes(parsed);
    }
}

pub fn options(s: &Settings, tuning: Tuning, timbre: Timbre) -> Options {
    Options {
        spelling: s.spelling,
        ratio_set: s.ratio_set,
        a4: s.a4,
        timbre,
        tuning,
        model: s.model,
    }
}

/// Keeps the voicing in the URL fragment (`#C3,G3,Ab3,Eb4`) so a chord can be shared: read once
/// at startup, then rewritten whenever the notes change.
pub fn use_url_sync(mut app: AppState) {
    let mut loaded = use_signal(|| false);
    use_future(move || async move {
        // Sends the fragment now and again whenever the user edits it.
        let mut eval = document::eval(
            "dioxus.send(location.hash); window.addEventListener('hashchange', () => dioxus.send(location.hash)); await new Promise(() => {});",
        );
        while let Ok(v) = eval.recv::<serde_json::Value>().await {
            let hash = v
                .as_str()
                .unwrap_or("")
                .trim_start_matches('#')
                .replace("%20", " ");
            let notes = parse_notes(&hash);
            if !notes.is_empty() && notes != *app.notes.peek() {
                app.set_notes(notes);
            }
            loaded.set(true);
        }
    });
    use_effect(move || {
        let notes = app.notes.read().clone();
        if !loaded() {
            return;
        }
        // Flats only: '#' cannot appear inside a fragment.
        let hash = format_notes(&notes, Spelling::Flats).replace(' ', ",");
        let _ = document::eval(&format!("history.replaceState(null, '', '#{hash}')"));
    });
}

pub fn use_app() -> AppState {
    use_context::<AppState>()
}
