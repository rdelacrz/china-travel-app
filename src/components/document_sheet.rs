use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::input::Input;
use crate::components::label::Label;
use crate::components::sheet::{
    Sheet, SheetContentClose, SheetDescription, SheetFooter, SheetHeader, SheetTitle,
};
use crate::components::textarea::Textarea;
use crate::domain::{AttachmentRef, TravelDocument};
use dioxus::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentSheetMode {
    Add,
    Edit(i64),
}

#[component]
pub fn DocumentSheet(
    open: bool,
    mode: DocumentSheetMode,
    name: String,
    description: String,
    attachment: Option<AttachmentRef>,
    saving: bool,
    picking: bool,
    validation_error: Option<String>,
    on_name_change: EventHandler<FormEvent>,
    on_description_change: EventHandler<FormEvent>,
    on_attach: EventHandler<MouseEvent>,
    on_remove_attachment: EventHandler<MouseEvent>,
    on_save: EventHandler<MouseEvent>,
    on_cancel: EventHandler<()>,
) -> Element {
    let title = match mode {
        DocumentSheetMode::Add => "Add document",
        DocumentSheetMode::Edit(_) => "Edit document",
    };
    let attachment_name = attachment
        .as_ref()
        .and_then(|item| item.display_name.clone())
        .unwrap_or_else(|| "Attached file".to_string());

    rsx! {
        Sheet {
            open,
            on_open_change: move |is_open: bool| {
                if !is_open {
                    on_cancel.call(());
                }
            },
            SheetContentClose { aria_label: "Close document form" }
            SheetHeader {
                SheetTitle { "{title}" }
                SheetDescription { "Save travel notes and optional files locally on this device." }
            }
            div { class: "flex flex-1 flex-col gap-5 overflow-y-auto px-5 py-4",
                div { class: "space-y-2",
                    Label { html_for: "document-name", "Document name" }
                    Input {
                        id: "document-name",
                        value: name,
                        placeholder: "e.g. Passport scan",
                        aria_label: "Document name",
                        disabled: saving,
                        oninput: move |event| on_name_change.call(event),
                    }
                }
                div { class: "space-y-2",
                    Label { html_for: "document-description", "Description" }
                    Textarea {
                        id: "document-description",
                        class: "min-h-36 resize-y",
                        value: description,
                        placeholder: "Add visa notes, addresses, or reminders…",
                        aria_label: "Document description",
                        disabled: saving,
                        oninput: move |event| on_description_change.call(event),
                    }
                }
                div { class: "space-y-3",
                    Label { html_for: "document-attachment", "Attachment (optional)" }
                    Button {
                        id: "document-attachment",
                        variant: ButtonVariant::Outline,
                        size: ButtonSize::Lg,
                        class: "min-h-12 w-full justify-start",
                        disabled: saving || picking,
                        onclick: move |event| on_attach.call(event),
                        if picking { "Opening file picker…" } else { "Choose a file" }
                    }
                    if attachment.is_some() {
                        div { class: "flex items-center justify-between gap-3 rounded-xl bg-slate-100 p-3 text-sm",
                            span { class: "min-w-0 truncate text-slate-700", "📎 {attachment_name}" }
                            button {
                                class: "min-h-12 shrink-0 rounded-lg px-3 font-semibold text-red-700 underline-offset-2 hover:underline focus-visible:outline-2 focus-visible:outline-red-700",
                                type: "button",
                                disabled: saving || picking,
                                aria_label: "Remove attachment from form",
                                onclick: move |event| on_remove_attachment.call(event),
                                "Remove"
                            }
                        }
                    }
                }
                if let Some(error) = validation_error {
                    p { class: "rounded-xl bg-red-50 p-3 text-sm leading-5 text-red-800", "{error}" }
                }
            }
            SheetFooter {
                div { class: "grid w-full grid-cols-2 gap-3",
                    Button {
                        variant: ButtonVariant::Outline,
                        class: "flex min-h-12 items-center justify-center rounded-xl border border-slate-300 px-4 text-sm font-semibold text-slate-700 focus-visible:outline-2 focus-visible:outline-red-700",
                        disabled: saving || picking,
                        onclick: move |_| on_cancel.call(()),
                        "Cancel"
                    }
                    Button {
                        size: ButtonSize::Lg,
                        class: "min-h-12",
                        disabled: saving || picking,
                        onclick: move |event| on_save.call(event),
                        if saving { "Saving…" } else { "Save" }
                    }
                }
            }
        }
    }
}

#[allow(dead_code)]
fn _document_mode_for_record(document: &TravelDocument) -> DocumentSheetMode {
    DocumentSheetMode::Edit(document.id)
}
