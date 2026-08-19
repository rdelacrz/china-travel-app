use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::components::confirm_delete::ConfirmTripDeleteDialog;

use crate::components::toast::{use_toast, ToastOptions};
use crate::components::trip_pane::TripPane;
use crate::domain::Trip;
use crate::state::{use_database, use_revision, use_safe_mode};
use dioxus::prelude::*;

#[component]
pub fn Home() -> Element {
    let database = use_database();
    let mut revision = use_revision();
    let safe_mode = use_safe_mode();
    let toast = use_toast();
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
    let mut start_date = use_signal(String::new);
    let mut end_date = use_signal(String::new);
    let mut form_error = use_signal(|| None::<String>);
    let mut saving = use_signal(|| false);
    let mut pending_delete = use_signal(|| None::<Trip>);

    let delete_trip = use_callback({
        let database = database.clone();
        move |_: ()| {
            if safe_mode() {
                pending_delete.set(None);
                return;
            }
            let Some(trip) = pending_delete() else {
                return;
            };
            pending_delete.set(None);
            let trip_id = trip.id;
            let database = database.clone();
            spawn(async move {
                match database.delete_trip(trip_id).await {
                    Ok(()) => revision.set(revision() + 1),
                    Err(error) => toast.error(
                        "Trip could not be deleted".to_string(),
                        ToastOptions::default().description(error.to_string()),
                    ),
                }
            });
        }
    });

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
                    on_press: move |_| trips.restart(),
                    "Retry"
                }
            }
        },
        Some(Ok(overviews)) => {
            let trip_count = overviews.len();
            rsx! {
                section { class: "space-y-6",
                    div { class: "space-y-2",
                        p { class: "text-sm font-semibold uppercase tracking-[0.16em] text-red-700", "China travel companion" }
                        h1 { class: "text-3xl font-bold tracking-tight text-slate-950", "Travel prepared, one step at a time." }
                        p { class: "max-w-2xl text-sm leading-6 text-slate-600", "Keep packing tasks, important travel notes, and supporting files together for your China trips. Everything is stored locally on this device." }
                    }
                    div { class: "flex items-center justify-between gap-3",
                        h2 { class: "text-xl font-bold text-slate-950", "Your trips ({trip_count})" }
                        if !show_add() {
                            Button {
                                size: ButtonSize::Sm,
                                on_press: move |_| {
                                    name.set(String::new());
                                    start_date.set(String::new());
                                    end_date.set(String::new());
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
                            start_date,
                            end_date,
                            error: form_error,
                            saving,
                            on_name_change: move |event: FormEvent| name.set(event.value()),
                            on_start_date_change: move |event: FormEvent| start_date.set(event.value()),
                            on_end_date_change: move |event: FormEvent| end_date.set(event.value()),
                            on_cancel: move |_| {
                                show_add.set(false);
                                name.set(String::new());
                                start_date.set(String::new());
                                end_date.set(String::new());
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
                                let trip_dates = match Trip::normalize_date_range(
                                    Some(&start_date()),
                                    Some(&end_date()),
                                ) {
                                    Ok(dates) => dates,
                                    Err(error) => {
                                        form_error.set(Some(error.to_string()));
                                        return;
                                    }
                                };
                                saving.set(true);
                                form_error.set(None);
                                let database = database.clone();
                                spawn(async move {
                                    match database
                                        .create_trip_with_dates(
                                            &cleaned,
                                            trip_dates.0.as_deref(),
                                            trip_dates.1.as_deref(),
                                        )
                                        .await
                                    {
                                        Ok(_) => {
                                            name.set(String::new());
                                            start_date.set(String::new());
                                            end_date.set(String::new());
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
                                {
                                    let trip_for_delete = overview.trip.clone();
                                    rsx! {
                                        li { key: "trip-{overview.trip.id}",
                                            TripPane {
                                                overview,
                                                on_delete: move |_| pending_delete.set(Some(trip_for_delete.clone())),
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if !safe_mode() {
                        ConfirmTripDeleteDialog {
                            trip: pending_delete(),
                            on_confirm: move |_| delete_trip.call(()),
                            on_cancel: move |_| pending_delete.set(None),
                        }
                    }
                }
            }
        }
    };

    rsx! { {view} }
}

#[component]
fn AddTripForm(
    name: Signal<String>,
    start_date: Signal<String>,
    end_date: Signal<String>,
    error: Signal<Option<String>>,
    saving: Signal<bool>,
    on_name_change: EventHandler<FormEvent>,
    on_start_date_change: EventHandler<FormEvent>,
    on_end_date_change: EventHandler<FormEvent>,
    on_cancel: EventHandler<()>,
    on_save: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "rounded-3xl border border-red-100 bg-gradient-to-br from-red-50 to-white p-5 shadow-sm",
            div { class: "space-y-4",
                div {
                    p { class: "text-xs font-bold uppercase tracking-[0.16em] text-red-700", "New trip" }
                    label { class: "mt-2 block text-sm font-semibold text-slate-800", r#for: "new-trip-name", "Trip name" }
                }
                input {
                    id: "new-trip-name",
                    r#type: "text",
                    class: "mt-1 block min-h-12 w-full rounded-xl border border-slate-300 bg-white px-3 py-3 text-base text-slate-900 shadow-sm outline-none transition placeholder:text-slate-400 focus:border-red-700 focus:ring-2 focus:ring-red-100 disabled:cursor-not-allowed disabled:opacity-50",
                    value: name(),
                    placeholder: "e.g. Beijing and Shanghai 2027",
                    aria_label: "Trip name",
                    disabled: saving(),
                    oninput: move |event| on_name_change.call(event),
                }
                div { class: "grid gap-3 sm:grid-cols-2",
                    div {
                        label { class: "block text-sm font-semibold text-slate-800", r#for: "new-trip-start-date", "Start date" }
                        input {
                            id: "new-trip-start-date",
                            r#type: "date",
                            class: "mt-1 block min-h-12 w-full rounded-xl border border-slate-300 bg-white px-3 text-base text-slate-900 shadow-sm outline-none transition focus:border-red-700 focus:ring-2 focus:ring-red-100 disabled:cursor-not-allowed disabled:opacity-50",
                            value: start_date(),
                            aria_label: "Trip start date",
                            disabled: saving(),
                            oninput: move |event| on_start_date_change.call(event),
                            onchange: move |event| on_start_date_change.call(event),
                        }
                    }
                    div {
                        label { class: "block text-sm font-semibold text-slate-800", r#for: "new-trip-end-date", "End date" }
                        input {
                            id: "new-trip-end-date",
                            r#type: "date",
                            class: "mt-1 block min-h-12 w-full rounded-xl border border-slate-300 bg-white px-3 text-base text-slate-900 shadow-sm outline-none transition focus:border-red-700 focus:ring-2 focus:ring-red-100 disabled:cursor-not-allowed disabled:opacity-50",
                            value: end_date(),
                            aria_label: "Trip end date",
                            disabled: saving(),
                            oninput: move |event| on_end_date_change.call(event),
                            onchange: move |event| on_end_date_change.call(event),
                        }
                    }
                }
            }
            if let Some(message) = error() {
                p { class: "mt-2 text-sm text-red-800", "{message}" }
            }
            div { class: "mt-5 grid grid-cols-2 gap-3 sm:flex sm:justify-end",
                Button {
                    variant: ButtonVariant::Ghost,
                    class: "min-h-12",
                    disabled: saving(),
                    on_press: move |_| on_cancel.call(()),
                    "Cancel"
                }
                Button {
                    disabled: saving(),
                    on_press: move |_| on_save.call(()),
                    if saving() { "Saving…" } else { "Save trip" }
                }
            }
        }
    }
}
