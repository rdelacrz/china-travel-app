use crate::components::button::{Button, ButtonVariant};
use crate::domain::{ChecklistItem, TravelDocument, Trip};
use dioxus::prelude::*;

#[component]
pub fn ConfirmDeleteDialog(
    item: Option<ChecklistItem>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let open = item.is_some();
    let item_name = item
        .as_ref()
        .map(|value| value.text.clone())
        .unwrap_or_else(|| "this checklist item".to_string());
    rsx! {
        DeleteConfirmPanel {
            open,
            title: "Delete checklist item?",
            message: format!("Delete \"{item_name}\"? This cannot be undone."),
            confirm_label: "Delete",
            on_confirm,
            on_cancel,
        }
    }
}

#[component]
pub fn ConfirmDocumentDeleteDialog(
    document: Option<TravelDocument>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let open = document.is_some();
    let document_name = document
        .as_ref()
        .map(|value| value.name.clone())
        .unwrap_or_else(|| "this document".to_string());
    rsx! {
        DeleteConfirmPanel {
            open,
            title: "Delete document?",
            message: format!("Delete \"{document_name}\"? This cannot be undone."),
            confirm_label: "Delete",
            on_confirm,
            on_cancel,
        }
    }
}

#[component]
fn DeleteConfirmPanel(
    open: bool,
    title: &'static str,
    message: String,
    confirm_label: &'static str,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    rsx! {
        if open {
            div {
                class: "fixed inset-0 z-50 flex items-center justify-center bg-slate-950/40 px-4 backdrop-blur-sm",
                role: "presentation",
                onpointerup: move |_| on_cancel.call(()),
                div {
                    class: "w-full max-w-md rounded-3xl border border-slate-200 bg-white p-6 shadow-2xl",
                    role: "alertdialog",
                    aria_modal: "true",
                    onpointerup: move |event| event.stop_propagation(),
                    h2 { class: "text-xl font-bold text-slate-950", "{title}" }
                    p { class: "mt-3 text-sm leading-6 text-slate-600", "{message}" }
                    div { class: "mt-6 grid grid-cols-2 gap-3",
                        Button {
                            variant: ButtonVariant::Outline,
                            class: "min-h-12",
                            on_press: move |_| on_cancel.call(()),
                            "Cancel"
                        }
                        Button {
                            variant: ButtonVariant::Destructive,
                            class: "min-h-12",
                            on_press: move |_| on_confirm.call(()),
                            "{confirm_label}"
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ConfirmTripDeleteDialog(
    trip: Option<Trip>,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let open = trip.is_some();
    let trip_name = trip
        .as_ref()
        .map(|value| value.name.clone())
        .unwrap_or_else(|| "this trip".to_string());
    rsx! {
        DeleteConfirmPanel {
            open,
            title: "Delete trip?",
            message: format!(
                "Delete \"{trip_name}\" and all of its checklist items and documents? This cannot be undone."
            ),
            confirm_label: "Delete trip",
            on_confirm,
            on_cancel,
        }
    }
}
