use crate::components::button::{Button, ButtonSize};
use crate::components::document_pane::DocumentPane;
use crate::components::document_sheet::{DocumentSheet, DocumentSheetMode};
use crate::domain::{AttachmentRef, NewTravelDocument, TravelDocument, UpdateTravelDocument};
use crate::platform::PickDocumentOutcome;
use crate::state::{use_database, use_platform, use_revision};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};

#[component]
pub fn Documentation(trip_id: i64) -> Element {
    let database = use_database();
    let platform = use_platform();
    let mut revision = use_revision();
    let toast = use_toast();
    let mut data = use_resource({
        let database = database.clone();
        move || {
            let database = database.clone();
            let _revision = revision();
            async move {
                let trip = database.get_trip(trip_id).await?;
                let documents = database.list_documents(trip_id).await?;
                Ok::<_, crate::error::DbError>((trip, documents))
            }
        }
    });
    let mut sheet_mode = use_signal(|| None::<DocumentSheetMode>);
    let mut form_name = use_signal(String::new);
    let mut form_description = use_signal(String::new);
    let mut form_attachment = use_signal(|| None::<AttachmentRef>);
    let mut original_attachment = use_signal(|| None::<AttachmentRef>);
    let mut form_error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);
    let mut picking = use_signal(|| false);
    let mut expanded = use_signal(|| None::<i64>);

    let reset_form = use_callback(move |_: ()| {
        sheet_mode.set(None);
        form_name.set(String::new());
        form_description.set(String::new());
        form_attachment.set(None);
        original_attachment.set(None);
        form_error.set(None);
        saving.set(false);
        picking.set(false);
    });

    let edit_document = use_callback(move |document: TravelDocument| {
        form_name.set(document.name.clone());
        form_description.set(document.description.clone());
        form_attachment.set(document.attachment.clone());
        original_attachment.set(document.attachment.clone());
        form_error.set(None);
        sheet_mode.set(Some(DocumentSheetMode::Edit(document.id)));
    });

    let attach_file = use_callback({
        let platform = platform.clone();
        move |_: MouseEvent| {
            if picking() || saving() {
                return;
            }
            picking.set(true);
            let platform = platform.clone();
            spawn(async move {
                match platform.pick_document(true).await {
                    Ok(PickDocumentOutcome::Selected(attachment)) => {
                        form_attachment.set(Some(attachment));
                    }
                    Ok(PickDocumentOutcome::Cancelled) => {}
                    Err(error) => toast.error(
                        "File picker unavailable".to_string(),
                        ToastOptions::default().description(error.to_string()),
                    ),
                }
                picking.set(false);
            });
        }
    });

    let save_document = use_callback({
        let database = database.clone();
        let platform = platform.clone();
        move |_: MouseEvent| {
            if saving() || picking() {
                return;
            }
            let Some(mode) = sheet_mode() else {
                return;
            };
            let name = form_name();
            let description = form_description();
            let attachment = form_attachment();
            let command = match mode {
                DocumentSheetMode::Add => {
                    NewTravelDocument::new(trip_id, &name, description, attachment.clone())
                        .map(DocumentCommand::Create)
                }
                DocumentSheetMode::Edit(id) => {
                    UpdateTravelDocument::new(id, &name, description, attachment.clone())
                        .map(DocumentCommand::Update)
                }
            };
            let command = match command {
                Ok(command) => command,
                Err(error) => {
                    form_error.set(Some(error.to_string()));
                    return;
                }
            };
            let previous_attachment = original_attachment();
            saving.set(true);
            form_error.set(None);
            let database = database.clone();
            let platform = platform.clone();
            spawn(async move {
                let result = match command {
                    DocumentCommand::Create(document) => {
                        database.create_document(document).await.map(|_| ())
                    }
                    DocumentCommand::Update(document) => {
                        database.update_document(document).await.map(|_| ())
                    }
                };
                match result {
                    Ok(()) => {
                        let old_uri = previous_attachment.map(|attachment| attachment.uri);
                        let new_uri = attachment.map(|value| value.uri);
                        if let Some(uri) = old_uri.filter(|old| new_uri.as_ref() != Some(old)) {
                            if let Err(error) = platform.release_read_permission(&uri).await {
                                toast.warning(
                                    "Old file permission could not be released".to_string(),
                                    ToastOptions::default().description(error.to_string()),
                                );
                            }
                        }
                        reset_form.call(());
                        revision.set(revision() + 1);
                    }
                    Err(error) => {
                        saving.set(false);
                        form_error.set(Some(error.to_string()));
                    }
                }
            });
        }
    });

    let open_file = use_callback({
        let platform = platform.clone();
        move |attachment: AttachmentRef| {
            let platform = platform.clone();
            spawn(async move {
                if let Err(error) = platform.open_document(&attachment).await {
                    toast.error(
                        "Attached file could not be opened".to_string(),
                        ToastOptions::default().description(error.to_string()),
                    );
                }
            });
        }
    });

    let open_url = use_callback({
        let platform = platform.clone();
        move |url: String| {
            let platform = platform.clone();
            spawn(async move {
                if let Err(error) = platform.open_url(&url).await {
                    toast.error(
                        "Link could not be opened".to_string(),
                        ToastOptions::default().description(error.to_string()),
                    );
                }
            });
        }
    });

    let view = match &*data.value().read_unchecked() {
        None => {
            rsx! { p { class: "rounded-2xl border border-slate-200 bg-white p-5 text-sm text-slate-600", "Loading documents…" } }
        }
        Some(Err(error)) => rsx! {
            section { class: "rounded-2xl border border-red-200 bg-red-50 p-5",
                h1 { class: "text-lg font-bold text-red-900", "Documentation unavailable" }
                p { class: "mt-2 text-sm leading-6 text-red-800", "{error}" }
                Button { class: "mt-4 min-h-12", onclick: move |_| data.restart(), "Retry" }
            }
        },
        Some(Ok((trip, documents))) => rsx! {
            section { class: "space-y-5 pb-24",
                div { class: "flex items-start justify-between gap-3",
                    div {
                        p { class: "text-sm font-semibold uppercase tracking-[0.16em] text-red-700", "Travel documentation" }
                        h1 { class: "mt-1 text-2xl font-bold tracking-tight text-slate-950", "{trip.name}" }
                        p { class: "mt-2 text-sm text-slate-600", "Notes and files for your China trip." }
                    }
                    a {
                        href: "/",
                        class: "flex min-h-12 items-center rounded-xl px-3 text-sm font-semibold text-slate-700 hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-red-700",
                        "Back"
                    }
                }
                if documents.is_empty() {
                    div { class: "rounded-2xl border border-dashed border-slate-300 bg-white p-8 text-center",
                        p { class: "text-3xl", "🗂️" }
                        h2 { class: "mt-3 text-lg font-semibold text-slate-900", "No documents yet" }
                        p { class: "mx-auto mt-2 max-w-sm text-sm leading-6 text-slate-600", "Save visa notes, hotel addresses, train details, or an optional supporting file." }
                    }
                } else {
                    ul { class: "space-y-3",
                        for document in documents.iter().cloned() {
                            {
                                let document_id = document.id;
                                let document_for_edit = document.clone();
                                let document_for_file = document.clone();
                                rsx! {
                                    li { key: "document-{document_id}",
                                        DocumentPane {
                                            document,
                                            expanded: expanded() == Some(document_id),
                                            busy: saving() || picking(),
                                            on_toggle_view: move |_| {
                                                expanded.set(if expanded() == Some(document_id) { None } else { Some(document_id) });
                                            },
                                            on_edit: move |_| edit_document.call(document_for_edit.clone()),
                                            on_open_file: move |_| {
                                                if let Some(attachment) = document_for_file.attachment.clone() {
                                                    open_file.call(attachment);
                                                }
                                            },
                                            on_open_url: move |url: String| open_url.call(url),
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "fixed inset-x-0 bottom-0 z-10 border-t border-slate-200 bg-slate-50/95 px-4 py-3 backdrop-blur",
                    div { class: "mx-auto max-w-3xl",
                        Button {
                            size: ButtonSize::Lg,
                            class: "min-h-12 w-full",
                            onclick: move |_| {
                                form_name.set(String::new());
                                form_description.set(String::new());
                                form_attachment.set(None);
                                original_attachment.set(None);
                                form_error.set(None);
                                sheet_mode.set(Some(DocumentSheetMode::Add));
                            },
                            "+ Add document"
                        }
                    }
                }
                DocumentSheet {
                    open: sheet_mode().is_some(),
                    mode: sheet_mode().unwrap_or(DocumentSheetMode::Add),
                    name: form_name(),
                    description: form_description(),
                    attachment: form_attachment(),
                    saving: saving(),
                    picking: picking(),
                    validation_error: form_error(),
                    on_name_change: move |event: FormEvent| form_name.set(event.value()),
                    on_description_change: move |event: FormEvent| form_description.set(event.value()),
                    on_attach: move |event: MouseEvent| attach_file.call(event),
                    on_remove_attachment: move |_| form_attachment.set(None),
                    on_save: move |event: MouseEvent| save_document.call(event),
                    on_cancel: move |_| reset_form.call(()),
                }
            }
        },
    };

    rsx! { {view} }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DocumentCommand {
    Create(NewTravelDocument),
    Update(UpdateTravelDocument),
}
