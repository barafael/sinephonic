//! Web MIDI input: while keys are held, the voicing follows them; releasing keeps the chord.

use dioxus::prelude::*;
use serde_json::Value;

use crate::state::AppState;

const SCRIPT: &str = r#"
if (!navigator.requestMIDIAccess) {
  dioxus.send({ status: "Web MIDI is not available in this browser" });
  return;
}
try {
  dioxus.send({ status: "MIDI: waiting for browser permission…" });
  const access = await navigator.requestMIDIAccess();
  const held = new Set();
  const names = [];
  const hook = (input) => {
    names.push(input.name);
    input.onmidimessage = (e) => {
      const [st, note, vel] = e.data;
      const cmd = st & 0xf0;
      if (cmd === 0x90 && vel > 0) {
        held.add(note);
        dioxus.send({ notes: [...held].sort((a, b) => a - b) });
      } else if (cmd === 0x80 || (cmd === 0x90 && vel === 0)) {
        held.delete(note);
      }
    };
  };
  access.inputs.forEach(hook);
  access.onstatechange = (e) => {
    if (e.port.type === "input" && e.port.state === "connected" && !e.port.onmidimessage) {
      hook(e.port);
      dioxus.send({ status: "MIDI: " + names.join(", ") });
    }
  };
  dioxus.send({ status: names.length ? "MIDI: " + names.join(", ") : "MIDI: no input devices yet" });
  await new Promise(() => {});
} catch (err) {
  dioxus.send({ status: "MIDI access denied" });
}
"#;

/// Starts listening; status messages go to `status`, chords to the app state.
pub fn connect(mut app: AppState, mut status: Signal<Option<String>>) {
    status.set(Some("MIDI: connecting…".into()));
    spawn(async move {
        let mut eval = document::eval(SCRIPT);
        while let Ok(msg) = eval.recv::<Value>().await {
            if let Some(s) = msg.get("status").and_then(Value::as_str) {
                status.set(Some(s.to_string()));
            }
            if let Some(notes) = msg.get("notes").and_then(Value::as_array) {
                let notes: Vec<u8> = notes
                    .iter()
                    .filter_map(|n| n.as_u64())
                    .map(|n| n as u8)
                    .collect();
                app.set_notes(notes);
            }
        }
    });
}
