use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::input::Input;
use crate::components::trip_pane::TripPane;
use crate::domain::Trip;
use crate::state::{use_database, use_revision};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let database = use_database();
    let mut revision = use_revision();
    let mut trips = use_resource({
        let database = database.clone();
        move || {
            let database = database.clone();
            let _revision = revision();
            async move { database.list_trip_overviews().await }
        }
    });
    let mut show_add = use_signal(|| false);
    let mut name = use_signal(String::new);
    let mut form_error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);

    let data = trips.value();
    let view = match &*data.read_unchecked() {
        None => rsx! {
            p { class: "rounded-2xl border border-slate-200 bg-white p-5 text-sm text-slate-600", "Loading trips…" }
        },
        Some(Err(error)) => rsx! {
            section { class: "rounded-2xl border border-red-200 bg-red-50 p-5",
                h1 { class: "text-lg font-bold text-red-900", "Trips could not be loaded" }
                p { class: "mt-2 text-sm leading-6 text-red-800", "{error}" }
                Button {
                    class: "mt-4 min-h-12",
                    onclick: move |_| trips.restart(),
                    "Retry"
                }
            }
        },
        Some(Ok(overviews)) => {
            let trip_count = overviews.len();
            let outstanding: i64 = overviews
                .iter()
                .map(|trip| trip.checklist_outstanding())
                .sum();
            let completed: i64 = overviews.iter().map(|trip| trip.checklist_completed).sum();
            let document_count: i64 = overviews.iter().map(|trip| trip.document_count).sum();
            rsx! {
                section { class: "space-y-6",
                    div { class: "space-y-2",
                        p { class: "text-sm font-semibold uppercase tracking-[0.16em] text-red-700", "China travel companion" }
                        h1 { class: "text-3xl font-bold tracking-tight text-slate-950", "Travel prepared, one step at a time." }
                        p { class: "max-w-2xl text-sm leading-6 text-slate-600", "Keep packing tasks, important travel notes, and supporting files together for your China trips. Everything is stored locally on this device." }
                    }
                    div { class: "grid grid-cols-2 gap-3 sm:grid-cols-4",
                        SummaryMetric { label: "Trips", value: trip_count.to_string() }
                        SummaryMetric { label: "Outstanding", value: outstanding.to_string() }
                        SummaryMetric { label: "Completed", value: completed.to_string() }
                        SummaryMetric { label: "Documents", value: document_count.to_string() }
                    }
                    div { class: "flex items-center justify-between gap-3",
                        h2 { class: "text-xl font-bold text-slate-950", "Your trips" }
                        if !show_add() {
                            Button {
                                size: ButtonSize::Sm,
                                onclick: move |_| {
                                    form_error.set(None);
                                    show_add.set(true);
                                },
                                "+ Add trip"
                            }
                        }
                    }
                    if show_add() {
                        AddTripForm {
                            name,
                            error: form_error,
                            saving,
                            on_name_change: move |event: FormEvent| name.set(event.value()),
                            on_cancel: move |_| {
                                show_add.set(false);
                                name.set(String::new());
                                form_error.set(None);
                            },
                            on_save: move |_| {
                                if saving() {
                                    return;
                                }
                                let cleaned = match Trip::validate_name(&name()) {
                                    Ok(value) => value,
                                    Err(error) => {
                                        form_error.set(Some(error.to_string()));
                                        return;
                                    }
                                };
                                saving.set(true);
                                form_error.set(None);
                                let database = database.clone();
                                spawn(async move {
                                    match database.create_trip(&cleaned).await {
                                        Ok(_) => {
                                            name.set(String::new());
                                            show_add.set(false);
                                            saving.set(false);
                                            revision.set(revision() + 1);
                                        }
                                        Err(error) => {
                                            saving.set(false);
                                            form_error.set(Some(error.to_string()));
                                        }
                                    }
                                });
                            },
                        }
                    }
                    if overviews.is_empty() {
                        div { class: "rounded-2xl border border-dashed border-slate-300 bg-white p-8 text-center",
                            div { class: "text-4xl", "🧭" }
                            h3 { class: "mt-3 text-lg font-semibold text-slate-900", "No trips yet" }
                            p { class: "mx-auto mt-2 max-w-sm text-sm leading-6 text-slate-600", "Add your first China trip to start a checklist and keep your travel documents organized." }
                        }
                    } else {
                        ul { class: "space-y-3",
                            for overview in overviews.iter().cloned() {
                                li { key: "trip-{overview.trip.id}", TripPane { overview } }
                            }
                        }
                    }
                }
            }
        }
    };

    rsx! { {view} }
}

#[component]
fn SummaryMetric(label: &'static str, value: String) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-slate-200 bg-white p-4 shadow-sm",
            p { class: "text-2xl font-bold text-slate-950", "{value}" }
            p { class: "mt-1 text-xs font-semibold uppercase tracking-wide text-slate-500", "{label}" }
        }
    }
}

#[component]
fn AddTripForm(
    name: Signal<String>,
    error: Signal<Option<String>>,
    saving: Signal<bool>,
    on_name_change: EventHandler<FormEvent>,
    on_cancel: EventHandler<MouseEvent>,
    on_save: EventHandler<MouseEvent>,
) -> Element {
    rsx! {
        div { class: "rounded-2xl border border-red-100 bg-red-50/60 p-4",
            div { class: "space-y-2",
                label { class: "text-sm font-semibold text-slate-800", r#for: "new-trip-name", "Trip name" }
                Input {
                    id: "new-trip-name",
                    value: name(),
                    placeholder: "e.g. Beijing and Shanghai 2027",
                    aria_label: "Trip name",
                    disabled: saving(),
                    oninput: move |event| on_name_change.call(event),
                }
            }
            if let Some(message) = error() {
                p { class: "mt-2 text-sm text-red-800", "{message}" }
            }
            div { class: "mt-4 flex justify-end gap-2",
                Button {
                    variant: ButtonVariant::Ghost,
                    disabled: saving(),
                    onclick: move |event| on_cancel.call(event),
                    "Cancel"
                }
                Button {
                    disabled: saving(),
                    onclick: move |event| on_save.call(event),
                    if saving() { "Saving…" } else { "Save trip" }
                }
            }
        }
    }
}
