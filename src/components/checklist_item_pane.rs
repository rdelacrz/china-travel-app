use crate::components::checkbox::Checkbox;
use crate::domain::ChecklistItem;
use dioxus::html::point_interaction::InteractionLocation;
use dioxus::prelude::*;
use dioxus_icons::lucide::{GripVertical, Trash2};
use dioxus_primitives::checkbox::CheckboxState;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DragPoint {
    pub pointer_id: i32,
    pub x: f64,
    pub y: f64,
}

fn drag_point(event: &PointerEvent) -> DragPoint {
    let point = event.data().client_coordinates();
    DragPoint {
        pointer_id: event.data().pointer_id(),
        x: point.x,
        y: point.y,
    }
}

impl From<&PointerEvent> for DragPoint {
    fn from(event: &PointerEvent) -> Self {
        drag_point(event)
    }
}

#[component]
pub fn ChecklistItemPane(
    item: ChecklistItem,
    editing: bool,
    draft: String,
    busy: bool,
    checkbox_disabled: bool,
    validation_error: Option<String>,
    dragging: bool,
    drag_offset_x: f64,
    drag_offset_y: f64,
    horizontal_dragging: bool,
    active_stack: bool,
    dismissing_direction: Option<i8>,
    on_begin_edit: EventHandler<PointerEvent>,
    on_draft_change: EventHandler<FormEvent>,
    on_commit: EventHandler<FocusEvent>,
    on_keydown: EventHandler<KeyboardEvent>,
    on_checked_change: EventHandler<CheckboxState>,
    on_drag_start: EventHandler<DragPoint>,
) -> Element {
    let checked = if item.is_checked {
        CheckboxState::Checked
    } else {
        CheckboxState::Unchecked
    };
    let checkbox_visible = !editing || !draft.trim().is_empty();
    let show_delete_tray = horizontal_dragging || dismissing_direction.is_some();
    let vertical_dragging = dragging && !horizontal_dragging;
    let reveal_left =
        dismissing_direction.unwrap_or(if drag_offset_x >= 0.0 { 1 } else { -1 }) >= 0;
    let tray_class = if reveal_left {
        "absolute inset-0 flex items-center justify-start bg-red-700 px-6 text-white"
    } else {
        "absolute inset-0 flex items-center justify-end bg-red-700 px-6 text-white"
    };
    let panel_style = match dismissing_direction {
        Some(direction) if direction >= 0 => {
            "transform: translate3d(115%, 0, 0); opacity: 0;".to_string()
        }
        Some(_) => "transform: translate3d(-115%, 0, 0); opacity: 0;".to_string(),
        None => {
            format!("transform: translate3d({drag_offset_x}px, {drag_offset_y}px, 0); opacity: 1;")
        }
    };

    rsx! {
        li {
            class: "relative min-h-16",
            style: if active_stack {
                "touch-action: pan-y; z-index: 1000; isolation: isolate;"
            } else {
                "touch-action: pan-y; z-index: 0;"
            },
            "data-swipe-row": "true",
            "data-swipe-direction": if reveal_left { "right" } else { "left" },
            div {
                class: if vertical_dragging {
                    "relative min-h-16 overflow-visible"
                } else {
                    "relative min-h-16 overflow-hidden rounded-2xl"
                },
                if show_delete_tray {
                    div {
                        class: tray_class,
                        "data-swipe-background": "true",
                        aria_hidden: "true",
                        Trash2 { size: 24 }
                    }
                }
                div {
                    class: "relative z-10 flex min-h-16 select-none items-center gap-3 rounded-2xl border border-slate-200 bg-white p-3 shadow-sm transition-[transform,opacity] duration-200 ease-out",
                    style: panel_style,
                    "data-swipe-panel": "true",
                    if checkbox_visible {
                        Checkbox {
                            checked: Some(checked),
                            disabled: checkbox_disabled,
                            aria_label: if item.is_checked { "Mark item incomplete" } else { "Mark item complete" },
                            on_checked_change: move |state| on_checked_change.call(state),
                            span { class: "text-sm font-bold text-red-700", "✓" }
                        }
                    }
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
                                disabled: busy || dragging,
                                onpointerup: move |event| {
                                    if !dragging {
                                        on_begin_edit.call(event);
                                    }
                                },
                                "{item.text}"
                            }
                        }
                        if let Some(error) = validation_error {
                            p { class: "mt-1 text-xs leading-5 text-red-700", "{error}" }
                        }
                    }
                    button {
                        r#type: "button",
                        class: "flex min-h-12 min-w-12 shrink-0 touch-none items-center justify-center rounded-xl text-slate-400 transition hover:bg-slate-100 hover:text-slate-700 active:cursor-grabbing active:bg-slate-100 focus-visible:outline-2 focus-visible:outline-red-700",
                        style: "touch-action: none;",
                        aria_label: "Drag vertically to reorder or swipe horizontally to delete",
                        title: "Drag vertically to reorder or swipe horizontally to delete",
                        disabled: busy || editing,
                        onpointerdown: move |event| on_drag_start.call(drag_point(&event)),
                        GripVertical { size: 22 }
                    }
                }
            }
        }
    }
}
