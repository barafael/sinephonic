use dioxus::prelude::*;

/// A row of mutually exclusive buttons; the selected one is ink-filled.
#[component]
pub fn Segmented<T: Clone + PartialEq + 'static>(
    options: Vec<(T, String)>,
    value: T,
    onchange: EventHandler<T>,
) -> Element {
    rsx! {
        div { class: "segmented",
            for (v, label) in options {
                button {
                    class: if v == value { "on" },
                    onclick: {
                        let v = v.clone();
                        move |_| onchange.call(v.clone())
                    },
                    "{label}"
                }
            }
        }
    }
}
