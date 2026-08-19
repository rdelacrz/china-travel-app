use crate::components::toast::{use_toast, ToastOptions};
use crate::db::AppBackup;
use crate::platform::PickDocumentOutcome;
use crate::state::{use_database, use_platform, use_revision, use_safe_mode};
use dioxus::prelude::*;
use dioxus_icons::lucide::{Check, Download, Settings, Upload};

#[component]
pub fn GearMenu() -> Element {
    let database = use_database();
    let mut revision = use_revision();
    let mut safe_mode = use_safe_mode();
    let toast = use_toast();
    let platform = use_platform();
    let mut open = use_signal(|| false);
    let mut exporting = use_signal(|| false);
    let mut importing = use_signal(|| false);
    let mut safe_mode_toggling = use_signal(|| false);

    let export_db = database.clone();
    let export_platform = platform.clone();
    let import_db = database.clone();
    let import_platform = platform.clone();
    let safe_mode_db = database.clone();

    rsx! {
        div { class: "relative",
            button {
                r#type: "button",
                class: "flex h-11 w-11 items-center justify-center rounded-xl border border-slate-300 bg-white text-slate-700 shadow-sm ring-1 ring-slate-200/70 transition hover:border-red-300 hover:bg-red-50 hover:text-red-700 hover:shadow-md focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700 active:scale-95",
                aria_label: "Open settings menu",
                title: "Settings",
                onpointerup: move |event| {
                    event.stop_propagation();
                    open.set(!open());
                },
                Settings { size: 20 }
            }
            if open() {
                div {
                    class: "fixed inset-0 z-40",
                    aria_hidden: "true",
                    onpointerup: move |_| open.set(false),
                }
                div {
                    class: "absolute right-0 z-50 mt-2 w-60 origin-top-right rounded-2xl border border-slate-200 bg-white p-2 shadow-lg",
                    role: "menu",
                    onpointerup: move |event| event.stop_propagation(),

                    // Export
                    button {
                        r#type: "button",
                        class: "flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm font-medium text-slate-700 transition hover:bg-slate-50 focus-visible:outline-2 focus-visible:outline-red-700 disabled:opacity-50",
                        role: "menuitem",
                        disabled: exporting(),
                        onpointerup: move |_| {
                            if exporting() { return; }
                            exporting.set(true);
                            let database = export_db.clone();
                            let platform = export_platform.clone();
                            spawn(async move {
                                match database.export_full_backup().await {
                                    Ok(backup) => {
                                        match serde_json::to_string_pretty(&backup) {
                                            Ok(json) => {
                                                match platform
                                                    .create_document(
                                                        "china_travel_app_backup.json",
                                                        "application/json",
                                                        json.as_bytes(),
                                                    )
                                                    .await
                                                {
                                                    Ok(true) => {
                                                        toast.success(
                                                            "Backup exported".to_string(),
                                                            ToastOptions::default().description("china_travel_app_backup.json saved.".to_string()),
                                                        );
                                                    }
                                                    Ok(false) => {}
                                                    Err(error) => {
                                                        toast.error("Export failed".to_string(), ToastOptions::default().description(error.to_string()));
                                                    }
                                                }
                                            }
                                            Err(error) => {
                                                toast.error("Export failed".to_string(), ToastOptions::default().description(error.to_string()));
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        toast.error("Export failed".to_string(), ToastOptions::default().description(error.to_string()));
                                    }
                                }
                                exporting.set(false);
                                open.set(false);
                            });
                        },
                        if exporting() {
                            span { class: "text-xs", "Exporting…" }
                        } else {
                            Upload { size: 18 }
                            span { "Export" }
                        }
                    }

                    // Import
                    button {
                        r#type: "button",
                        class: "flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm font-medium text-slate-700 transition hover:bg-slate-50 focus-visible:outline-2 focus-visible:outline-red-700 disabled:opacity-50",
                        role: "menuitem",
                        disabled: importing(),
                        onpointerup: move |_| {
                            if importing() { return; }
                            importing.set(true);
                            let database = import_db.clone();
                            let platform = import_platform.clone();
                            spawn(async move {
                                match platform.pick_document(true).await {
                                    Ok(PickDocumentOutcome::Selected(file)) => {
                                        let json = match platform.read_text_document(&file.uri).await {
                                            Ok(json) => json,
                                            Err(error) => {
                                                toast.error("Import failed".to_string(), ToastOptions::default().description(error.to_string()));
                                                importing.set(false);
                                                open.set(false);
                                                return;
                                            }
                                        };
                                        match serde_json::from_str::<AppBackup>(&json) {
                                            Ok(backup) => {
                                                match database.import_full_backup(&backup).await {
                                                    Ok(()) => {
                                                        revision.set(revision() + 1);
                                                        toast.success(
                                                            "Backup imported".to_string(),
                                                            ToastOptions::default().description("All data replaced with backup.".to_string()),
                                                        );
                                                    }
                                                    Err(error) => {
                                                        toast.error("Import failed".to_string(), ToastOptions::default().description(error.to_string()));
                                                    }
                                                }
                                            }
                                            Err(error) => {
                                                toast.error("Import failed".to_string(), ToastOptions::default().description(format!("Invalid file: {error}")));
                                            }
                                        }
                                    }
                                    Ok(PickDocumentOutcome::Cancelled) => {}
                                    Err(error) => {
                                        toast.error("Import failed".to_string(), ToastOptions::default().description(error.to_string()));
                                    }
                                }
                                importing.set(false);
                                open.set(false);
                            });
                        },
                        if importing() {
                            span { class: "text-xs", "Importing…" }
                        } else {
                            Download { size: 18 }
                            span { "Import" }
                        }
                    }

                    div { class: "my-1 border-t border-slate-100" }

                    // Safe Mode toggle
                    button {
                        r#type: "button",
                        class: "flex w-full items-center gap-3 rounded-xl px-3 py-2.5 text-left text-sm font-medium text-slate-700 transition hover:bg-slate-50 focus-visible:outline-2 focus-visible:outline-red-700 disabled:opacity-50",
                        role: "menuitem",
                        disabled: safe_mode_toggling(),
                        onpointerup: move |_| {
                            if safe_mode_toggling() { return; }
                            let new_value = !safe_mode();
                            safe_mode_toggling.set(true);
                            let database = safe_mode_db.clone();
                            spawn(async move {
                                if let Err(error) = database.set_safe_mode_enabled(new_value).await {
                                    toast.error("Safe Mode not saved".to_string(), ToastOptions::default().description(error.to_string()));
                                    safe_mode_toggling.set(false);
                                    return;
                                }
                                safe_mode.set(new_value);
                                safe_mode_toggling.set(false);
                                if new_value {
                                    toast.success("Safe Mode enabled".to_string(), ToastOptions::default().description("Delete actions hidden.".to_string()));
                                } else {
                                    toast.info("Safe Mode disabled".to_string(), ToastOptions::default().description("Delete actions visible.".to_string()));
                                }
                            });
                        },
                        div { class: "flex h-6 w-6 shrink-0 items-center justify-center rounded-md border-2 border-slate-300 transition",
                            if safe_mode() {
                                Check { size: 16 }
                            }
                        }
                        span { "Safe Mode" }
                    }
                }
            }
        }
    }
}
