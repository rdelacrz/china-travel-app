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
    on_attach: EventHandler<()>,
    on_remove_attachment: EventHandler<()>,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    if !open {
        return rsx! {};
    }

    let title = match mode {
        DocumentSheetMode::Add => "Add document",
        DocumentSheetMode::Edit(_) => "Edit document",
    };
    let attachment_name = attachment
        .as_ref()
        .and_then(|item| item.display_name.clone())
        .unwrap_or_else(|| "Attached file".to_string());

    rsx! {
        div {
            class: "fixed inset-0 z-40 bg-slate-950/40",
            role: "presentation",
            onpointerup: move |_| on_cancel.call(()),
        }
        section {
            class: "fixed inset-y-0 right-0 z-50 flex w-full max-w-md flex-col border-l border-slate-200 bg-white shadow-2xl",
            role: "dialog",
            aria_modal: "true",
            aria_labelledby: "document-form-title",
            onpointerup: move |event| event.stop_propagation(),
            header { class: "flex items-start justify-between gap-4 border-b border-slate-200 px-5 py-5",
                div { class: "min-w-0",
                    p { class: "text-xs font-bold uppercase tracking-[0.16em] text-red-700", "Travel documents" }
                    h2 { id: "document-form-title", class: "mt-1 text-2xl font-bold tracking-tight text-slate-950", "{title}" }
                    p { class: "mt-2 max-w-xs text-sm leading-5 text-slate-600", "Save travel notes and optional files locally on this device." }
                }
                button {
                    r#type: "button",
                    class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-3xl leading-none text-slate-500 transition hover:bg-slate-100 hover:text-slate-900 focus-visible:outline-2 focus-visible:outline-red-700",
                    aria_label: "Close document form",
                    onpointerup: move |_| on_cancel.call(()),
                    "×"
                }
            }
            div { class: "flex flex-1 flex-col gap-6 overflow-y-auto px-5 py-6",
                div { class: "space-y-2",
                    label { class: "block text-sm font-semibold text-slate-800", r#for: "document-name", "Document name" }
                    input {
                        id: "document-name",
                        class: "mt-2 block min-h-12 w-full rounded-xl border border-slate-300 bg-white px-3 py-3 text-base text-slate-900 shadow-sm outline-none transition placeholder:text-slate-400 focus:border-red-700 focus:ring-2 focus:ring-red-100",
                        value: name,
                        placeholder: "e.g. Passport scan",
                        aria_label: "Document name",
                        disabled: saving,
                        oninput: move |event| on_name_change.call(event),
                    }
                }
                div { class: "space-y-2",
                    label { class: "block text-sm font-semibold text-slate-800", r#for: "document-description", "Description" }
                    textarea {
                        id: "document-description",
                        class: "mt-2 block min-h-36 w-full resize-y rounded-xl border border-slate-300 bg-white px-3 py-3 text-base leading-6 text-slate-900 shadow-sm outline-none transition placeholder:text-slate-400 focus:border-red-700 focus:ring-2 focus:ring-red-100",
                        value: description,
                        placeholder: "Add visa notes, addresses, or reminders…",
                        aria_label: "Document description",
                        disabled: saving,
                        oninput: move |event| on_description_change.call(event),
                    }
                }
                div { class: "space-y-2",
                    label { class: "block text-sm font-semibold text-slate-800", r#for: "document-attachment", "Attachment (optional)" }
                    button {
                        id: "document-attachment",
                        r#type: "button",
                        class: "mt-2 flex min-h-12 w-full items-center justify-start rounded-xl border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-700 shadow-sm transition hover:bg-slate-50 focus-visible:outline-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                        disabled: saving || picking,
                        onpointerup: move |_| on_attach.call(()),
                        if picking { "Opening file picker…" } else { "Choose a file" }
                    }
                    if attachment.is_some() {
                        div { class: "flex items-center justify-between gap-3 rounded-xl bg-slate-100 p-3 text-sm",
                            span { class: "min-w-0 truncate text-slate-700", "📎 {attachment_name}" }
                            button {
                                r#type: "button",
                                class: "min-h-10 shrink-0 rounded-lg px-3 font-semibold text-red-700 underline-offset-2 hover:underline focus-visible:outline-2 focus-visible:outline-red-700",
                                disabled: saving || picking,
                                aria_label: "Remove attachment from form",
                                onpointerup: move |_| on_remove_attachment.call(()),
                                "Remove"
                            }
                        }
                    }
                }
                if let Some(error) = validation_error {
                    p { class: "rounded-xl bg-red-50 p-3 text-sm leading-5 text-red-800", "{error}" }
                }
            }
            footer { class: "grid grid-cols-2 gap-3 border-t border-slate-200 bg-slate-50 px-5 py-4 pb-[calc(1rem+env(safe-area-inset-bottom))]",
                button {
                    r#type: "button",
                    class: "min-h-12 rounded-xl border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-700 transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                    disabled: saving || picking,
                    onpointerup: move |_| on_cancel.call(()),
                    "Cancel"
                }
                button {
                    r#type: "button",
                    class: "min-h-12 rounded-xl bg-red-700 px-4 text-sm font-semibold text-white shadow-sm transition hover:bg-red-800 focus-visible:outline-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                    disabled: saving || picking,
                    onpointerup: move |_| on_save.call(()),
                    if saving { "Saving…" } else { "Save" }
                }
            }
        }
    }
}

#[allow(dead_code)]
fn _document_mode_for_record(document: &TravelDocument) -> DocumentSheetMode {
    DocumentSheetMode::Edit(document.id)
}
