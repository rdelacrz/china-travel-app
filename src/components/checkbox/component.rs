use dioxus::prelude::*;
use dioxus_primitives::checkbox::{CheckboxProps, CheckboxState};
use std::time::Duration;

#[component]
pub fn Checkbox(props: CheckboxProps) -> Element {
    let state = (props.checked)().unwrap_or(props.default_checked);
    let disabled = (props.disabled)();
    let next_state = !state;
    let checked = state == CheckboxState::Checked;
    let mut activation_locked = use_signal(|| false);
    let on_checked_change = props.on_checked_change;
    let activate = use_callback(move |_: ()| {
        if disabled || activation_locked() {
            return;
        }
        activation_locked.set(true);
        on_checked_change.call(next_state);
        spawn(async move {
            // Android WebView commonly emits click immediately after pointerup.
            // The short lock accepts either event without toggling twice.
            tokio::time::sleep(Duration::from_millis(350)).await;
            activation_locked.set(false);
        });
    });

    rsx! {
        button {
            r#type: "button",
            role: "checkbox",
            aria_checked: match state {
                CheckboxState::Checked => "true",
                CheckboxState::Indeterminate => "mixed",
                CheckboxState::Unchecked => "false",
            },
            aria_required: props.required,
            disabled,
            "data-state": match state {
                CheckboxState::Checked => "checked",
                CheckboxState::Indeterminate => "indeterminate",
                CheckboxState::Unchecked => "unchecked",
            },
            "data-disabled": disabled,
            class: "travel-checkbox",
            style: if checked {
                "display: inline-flex; flex: 0 0 1.5rem; width: 1.5rem; height: 1.5rem; align-items: center; justify-content: center; padding: 0; border: 2px solid #b91c1c; border-radius: 0.5rem; background: #b91c1c; color: #ffffff;"
            } else {
                "display: inline-flex; flex: 0 0 1.5rem; width: 1.5rem; height: 1.5rem; align-items: center; justify-content: center; padding: 0; border: 2px solid #94a3b8; border-radius: 0.5rem; background: #ffffff; color: #ffffff;"
            },
            onpointerup: move |_| activate.call(()),
            onclick: move |_| activate.call(()),
            onkeydown: move |event| {
                if event.key() == Key::Enter || event.key() == Key::Character(" ".to_string()) {
                    event.prevent_default();
                    activate.call(());
                }
            },
            if checked || state == CheckboxState::Indeterminate {
                span { class: "flex items-center justify-center text-sm font-bold", if checked { "✓" } else { "−" } }
            }
        }
    }
}
