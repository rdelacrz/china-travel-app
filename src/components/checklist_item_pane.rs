use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::checkbox::Checkbox;

use crate::domain::ChecklistItem;
use dioxus::prelude::*;
use dioxus_icons::lucide::X;
use dioxus_primitives::checkbox::CheckboxState;

#[component]
pub fn ChecklistItemPane(
    item: ChecklistItem,
    editing: bool,
    draft: String,
    busy: bool,
    checkbox_disabled: bool,
    validation_error: Option<String>,
    on_begin_edit: EventHandler<MouseEvent>,
    on_draft_change: EventHandler<FormEvent>,
    on_commit: EventHandler<FocusEvent>,
    on_keydown: EventHandler<KeyboardEvent>,
    on_checked_change: EventHandler<CheckboxState>,
    on_delete: EventHandler<MouseEvent>,
) -> Element {
    let checked = if item.is_checked {
        CheckboxState::Checked
    } else {
        CheckboxState::Unchecked
    };

    rsx! {
        li { class: "flex min-h-16 items-center gap-3 rounded-2xl border border-slate-200 bg-white p-3 shadow-sm",
            div { class: "min-w-0 flex-1",
                if editing {
                    input {
                        class: "min-h-12 w-full rounded-xl border-slate-300 px-3 text-base focus:border-red-700 focus:ring-red-700",
                        value: draft,
                        aria_label: "Edit checklist item",
                        enterkeyhint: "go",
                        autofocus: true,
                        disabled: busy,
                        oninput: move |event| on_draft_change.call(event),
                        onblur: move |event| on_commit.call(event),
                        onkeydown: move |event| on_keydown.call(event),
                    }
                } else {
                    button {
                        class: "min-h-12 w-full rounded-xl px-1 text-left text-base leading-6 text-slate-800 transition hover:bg-slate-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700",
                        disabled: busy,
                        onclick: move |event| on_begin_edit.call(event),
                        "{item.text}"
                    }
                }
                if let Some(error) = validation_error {
                    p { class: "mt-1 text-xs leading-5 text-red-700", "{error}" }
                }
            }
            Checkbox {
                checked: Some(checked),
                disabled: checkbox_disabled,
                aria_label: if item.is_checked { "Mark item incomplete" } else { "Mark item complete" },
                on_checked_change: move |state| on_checked_change.call(state),
                span { class: "text-sm font-bold text-red-700", "✓" }
            }
            Button {
                variant: ButtonVariant::Ghost,
                size: ButtonSize::Icon,
                aria_label: "Delete checklist item",
                title: "Delete checklist item",
                disabled: busy,
                onclick: move |event| on_delete.call(event),
                X { size: 18 }
            }
        }
    }
}
