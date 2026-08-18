use crate::app::Route;
use crate::components::button::Button;
use crate::components::toast::{use_toast, ToastOptions};
use crate::domain::{CalendarDate, CalendarEvent, NewCalendarEvent, Trip, UpdateCalendarEvent};
use crate::state::{use_database, use_revision};
use dioxus::prelude::*;
use dioxus_icons::lucide::{ChevronDown, ChevronLeft, ChevronRight, Pencil, Plus, Trash2};

const WEEKDAYS: [&str; 7] = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct CalendarMonth {
    year: i32,
    month: u8,
}

impl CalendarMonth {
    fn today() -> Self {
        let date = CalendarDate::today();
        Self {
            year: date.year,
            month: date.month,
        }
    }

    fn previous(self) -> Self {
        if self.month > 1 {
            Self {
                month: self.month - 1,
                ..self
            }
        } else if self.year > 1 {
            Self {
                year: self.year - 1,
                month: 12,
            }
        } else {
            self
        }
    }

    fn next(self) -> Self {
        if self.month < 12 {
            Self {
                month: self.month + 1,
                ..self
            }
        } else if self.year < 9999 {
            Self {
                year: self.year + 1,
                month: 1,
            }
        } else {
            self
        }
    }

    fn title(self) -> String {
        format!(
            "{} {}",
            CalendarDate {
                year: self.year,
                month: self.month,
                day: 1,
            }
            .month_name(),
            self.year
        )
    }

    fn first_weekday(self) -> usize {
        (CalendarDate {
            year: self.year,
            month: self.month,
            day: 1,
        }
        .days_since_unix_epoch()
            + 3)
        .rem_euclid(7) as usize
    }

    fn days_in_month(self) -> u8 {
        CalendarDate::days_in_month(self.year, self.month)
    }
}

#[component]
pub fn Calendar(trip_id: i64) -> Element {
    let database = use_database();
    let mut revision = use_revision();
    let toast = use_toast();
    let mut data = use_resource({
        let database = database.clone();
        move || {
            let database = database.clone();
            let _revision = revision();
            async move {
                let trip = database.get_trip(trip_id).await?;
                let events = database.list_calendar_events(trip_id).await?;
                Ok::<_, crate::error::DbError>((trip, events))
            }
        }
    });
    let mut visible_month = use_signal(CalendarMonth::today);
    let mut start_date = use_signal(String::new);
    let mut end_date = use_signal(String::new);
    let mut date_error = use_signal(|| None::<String>);
    let mut dates_saving = use_signal(|| false);
    let mut synced_trip_dates = use_signal(|| None::<(i64, Option<String>, Option<String>)>);
    let mut edit_dates_open = use_signal(|| false);
    let mut edit_start_date = use_signal(String::new);
    let mut edit_end_date = use_signal(String::new);
    let mut event_name = use_signal(String::new);
    let mut event_start_date = use_signal(String::new);
    let mut event_end_date = use_signal(String::new);
    let mut event_error = use_signal(|| None::<String>);
    let mut event_saving = use_signal(|| false);
    let mut event_editor_open = use_signal(|| false);
    let mut editing_event_id = use_signal(|| None::<i64>);
    let mut removing_event = use_signal(|| None::<i64>);

    use_effect(move || {
        let value = data.value();
        let value = value.read_unchecked();
        let Some(Ok((trip, _))) = value.as_ref() else {
            return;
        };
        let snapshot = (trip.id, trip.start_date.clone(), trip.end_date.clone());
        if synced_trip_dates() == Some(snapshot.clone()) {
            return;
        }
        start_date.set(trip.start_date.clone().unwrap_or_default());
        end_date.set(trip.end_date.clone().unwrap_or_default());
        if let Some(date) = trip
            .start_date
            .as_deref()
            .and_then(|value| CalendarDate::parse(value).ok())
        {
            visible_month.set(CalendarMonth {
                year: date.year,
                month: date.month,
            });
        }
        synced_trip_dates.set(Some(snapshot));
    });

    let save_trip_dates = use_callback({
        let database = database.clone();
        move |_: ()| {
            if dates_saving() {
                return;
            }
            let submitted_start_date = edit_start_date();
            let submitted_end_date = edit_end_date();
            let (submitted_start_date, submitted_end_date) = match Trip::normalize_date_range(
                Some(&submitted_start_date),
                Some(&submitted_end_date),
            ) {
                Ok(dates) => dates,
                Err(error) => {
                    date_error.set(Some(error.to_string()));
                    return;
                }
            };
            date_error.set(None);
            dates_saving.set(true);
            let database = database.clone();
            spawn(async move {
                match database
                    .update_trip_dates(
                        trip_id,
                        submitted_start_date.as_deref(),
                        submitted_end_date.as_deref(),
                    )
                    .await
                {
                    Ok(_) => {
                        start_date.set(submitted_start_date.clone().unwrap_or_default());
                        end_date.set(submitted_end_date.clone().unwrap_or_default());
                        edit_dates_open.set(false);
                        revision.set(revision() + 1);
                    }
                    Err(error) => date_error.set(Some(error.to_string())),
                }
                dates_saving.set(false);
            });
        }
    });

    let open_date_editor = use_callback(move |_: ()| {
        edit_start_date.set(start_date());
        edit_end_date.set(end_date());
        date_error.set(None);
        edit_dates_open.set(true);
    });

    let close_date_editor = use_callback(move |_: ()| {
        edit_dates_open.set(false);
        date_error.set(None);
    });

    let open_add_event = use_callback(move |_: ()| {
        editing_event_id.set(None);
        event_name.set(String::new());
        event_start_date.set(String::new());
        event_end_date.set(String::new());
        event_error.set(None);
        event_editor_open.set(true);
    });

    let open_edit_event = use_callback(move |event: CalendarEvent| {
        editing_event_id.set(Some(event.id));
        event_name.set(event.name);
        event_start_date.set(event.start_date);
        event_end_date.set(event.end_date);
        event_error.set(None);
        event_editor_open.set(true);
    });

    let close_event_editor = use_callback(move |_: ()| {
        event_editor_open.set(false);
        editing_event_id.set(None);
        event_error.set(None);
    });

    let save_event = use_callback({
        let database = database.clone();
        move |_: ()| {
            if event_saving() {
                return;
            }
            let command = match editing_event_id() {
                Some(event_id) => UpdateCalendarEvent::new(
                    event_id,
                    trip_id,
                    &event_name(),
                    &event_start_date(),
                    &event_end_date(),
                )
                .map(EventCommand::Update),
                None => NewCalendarEvent::new(
                    trip_id,
                    &event_name(),
                    &event_start_date(),
                    &event_end_date(),
                )
                .map(EventCommand::Create),
            };
            let command = match command {
                Ok(command) => command,
                Err(error) => {
                    event_error.set(Some(error.to_string()));
                    return;
                }
            };
            event_error.set(None);
            event_saving.set(true);
            let database = database.clone();
            spawn(async move {
                let result = match command {
                    EventCommand::Create(event) => database.create_calendar_event(event).await,
                    EventCommand::Update(event) => database.update_calendar_event(event).await,
                };
                match result {
                    Ok(_) => {
                        event_name.set(String::new());
                        event_start_date.set(String::new());
                        event_end_date.set(String::new());
                        editing_event_id.set(None);
                        event_editor_open.set(false);
                        revision.set(revision() + 1);
                    }
                    Err(error) => event_error.set(Some(error.to_string())),
                }
                event_saving.set(false);
            });
        }
    });

    let delete_event = use_callback({
        let database = database.clone();
        move |event_id: i64| {
            if removing_event().is_some() {
                return;
            }
            removing_event.set(Some(event_id));
            let database = database.clone();
            spawn(async move {
                match database.delete_calendar_event(event_id).await {
                    Ok(()) => revision.set(revision() + 1),
                    Err(error) => toast.error(
                        "Event could not be removed".to_string(),
                        ToastOptions::default().description(error.to_string()),
                    ),
                }
                removing_event.set(None);
            });
        }
    });

    let view = match &*data.value().read_unchecked() {
        None => {
            rsx! { p { class: "rounded-2xl border border-slate-200 bg-white p-5 text-sm text-slate-600", "Loading calendar…" } }
        }
        Some(Err(crate::error::DbError::NotFound { entity: "trip", .. })) => rsx! {
            section { class: "rounded-2xl border border-amber-200 bg-amber-50 p-5",
                h1 { class: "text-lg font-bold text-amber-950", "Trip not found" }
                p { class: "mt-2 text-sm leading-6 text-amber-900", "This calendar route no longer points to a saved trip." }
                Link { to: Route::Home {}, class: "mt-4 inline-flex min-h-12 items-center rounded-xl bg-amber-900 px-4 text-sm font-semibold text-white", "Back to trips" }
            }
        },
        Some(Err(error)) => rsx! {
            section { class: "rounded-2xl border border-red-200 bg-red-50 p-5",
                h1 { class: "text-lg font-bold text-red-900", "Calendar unavailable" }
                p { class: "mt-2 text-sm leading-6 text-red-800", "{error}" }
                Button { class: "mt-4 min-h-12", on_press: move |_| data.restart(), "Retry" }
            }
        },
        Some(Ok((trip, events))) => {
            let month = visible_month();
            let today = CalendarDate::today();
            let trip_start = trip
                .start_date
                .as_deref()
                .and_then(|value| CalendarDate::parse(value).ok());
            let trip_end = trip
                .end_date
                .as_deref()
                .and_then(|value| CalendarDate::parse(value).ok());
            rsx! {
                section { class: "space-y-5 pb-8",
                    div { class: "flex items-start justify-between gap-3",
                        div {
                            p { class: "text-sm font-semibold uppercase tracking-[0.16em] text-red-700", "Trip calendar" }
                            h1 { class: "mt-1 text-2xl font-bold tracking-tight text-slate-950", "{trip.name}" }
                            p { class: "mt-2 text-sm text-slate-600", "Set your travel dates and keep each reservation or activity in one place." }
                        }
                        Link {
                            to: Route::Home {},
                            class: "flex min-h-12 items-center rounded-xl px-3 text-sm font-semibold text-slate-700 hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-red-700",
                            "Back"
                        }
                    }

                    section { class: "rounded-2xl border border-slate-200 bg-white p-4 shadow-sm",
                        div { class: "flex items-center justify-between gap-3",
                            h2 { class: "text-base font-bold text-slate-950", "Trip dates" }
                            button {
                                r#type: "button",
                                class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-slate-500 transition hover:bg-red-50 hover:text-red-700 focus-visible:outline-2 focus-visible:outline-red-700",
                                aria_label: "Edit trip dates",
                                title: "Edit trip dates",
                                onpointerup: move |_| open_date_editor.call(()),
                                Pencil { size: 19 }
                            }
                        }
                        p { class: "mt-1 text-sm text-slate-600", "They are marked on the calendar and shown on your trip overview." }
                        if let Some(label) = trip.date_range_label() {
                            p { class: "mt-3 inline-flex rounded-full bg-red-50 px-3 py-1.5 text-xs font-semibold text-red-700", "{label}" }
                        } else {
                            p { class: "mt-3 text-sm font-medium text-slate-500", "No trip dates set" }
                        }
                    }

                    section { class: "overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm",
                        div { class: "flex items-center justify-between border-b border-slate-200 px-3 py-3 sm:px-4",
                            button {
                                r#type: "button",
                                class: "flex min-h-11 min-w-11 items-center justify-center rounded-xl text-slate-600 transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-red-700",
                                aria_label: "Previous month",
                                title: "Previous month",
                                onpointerup: move |_| visible_month.set(visible_month().previous()),
                                ChevronLeft { size: 20 }
                            }
                            div { class: "text-center",
                                h2 { class: "text-lg font-bold text-slate-950", aria_live: "polite", "{month.title()}" }
                                button {
                                    r#type: "button",
                                    class: "mt-0.5 rounded px-2 py-1 text-xs font-semibold text-red-700 hover:bg-red-50 focus-visible:outline-2 focus-visible:outline-red-700",
                                    onpointerup: move |_| visible_month.set(CalendarMonth::today()),
                                    "Today"
                                }
                            }
                            button {
                                r#type: "button",
                                class: "flex min-h-11 min-w-11 items-center justify-center rounded-xl text-slate-600 transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-red-700",
                                aria_label: "Next month",
                                title: "Next month",
                                onpointerup: move |_| visible_month.set(visible_month().next()),
                                ChevronRight { size: 20 }
                            }
                        }
                        div { class: "grid grid-cols-7 border-b border-slate-200 bg-slate-50",
                            for weekday in WEEKDAYS {
                                div { class: "px-1 py-2 text-center text-[10px] font-bold uppercase tracking-wide text-slate-500 sm:text-xs", "{weekday}" }
                            }
                        }
                        div { class: "grid grid-cols-7",
                            for _ in 0..month.first_weekday() {
                                div { class: "min-h-22 border-b border-r border-slate-100 bg-slate-50/60 sm:min-h-28" }
                            }
                            for day in 1..=month.days_in_month() {
                                {
                                    let date = CalendarDate { year: month.year, month: month.month, day };
                                    let is_today = date == today;
                                    let is_trip_start = trip_start == Some(date);
                                    let is_trip_end = trip_end == Some(date);
                                    let day_events = events
                                        .iter()
                                        .filter(|event| event_occurs_on(event, date))
                                        .cloned()
                                        .collect::<Vec<_>>();
                                    let cell_class = if is_trip_start || is_trip_end {
                                        "min-h-22 border-b border-r border-slate-100 bg-red-50/60 p-1.5 sm:min-h-28 sm:p-2"
                                    } else {
                                        "min-h-22 border-b border-r border-slate-100 bg-white p-1.5 sm:min-h-28 sm:p-2"
                                    };
                                    let day_class = if is_today {
                                        "flex h-6 w-6 items-center justify-center rounded-full bg-red-700 text-xs font-bold text-white"
                                    } else {
                                        "flex h-6 w-6 items-center justify-center rounded-full text-xs font-semibold text-slate-700"
                                    };
                                    rsx! {
                                        div { key: "day-{date}", class: cell_class, aria_label: "{date}",
                                            span { class: day_class, "{day}" }
                                            if is_trip_start {
                                                p { class: "mt-1 truncate text-[10px] font-bold leading-4 text-red-800", title: "Trip starts", "Trip starts" }
                                            }
                                            if is_trip_end {
                                                p { class: "truncate text-[10px] font-bold leading-4 text-red-800", title: "Trip ends", "Trip ends" }
                                            }
                                            for event in day_events {
                                                p {
                                                    key: "event-on-{event.id}-{date}",
                                                    class: "mt-1 truncate rounded bg-sky-100 px-1 text-[10px] font-semibold leading-4 text-sky-900",
                                                    title: "{event.name} ({event.start_date} – {event.end_date})",
                                                    "{abbreviate_event_name(&event.name)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    section { class: "rounded-2xl border border-slate-200 bg-white p-4 shadow-sm",
                        div { class: "flex items-center justify-between gap-3",
                            h2 { class: "text-base font-bold text-slate-950", "Scheduled events" }
                            button {
                                r#type: "button",
                                class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-slate-500 transition hover:bg-red-50 hover:text-red-700 focus-visible:outline-2 focus-visible:outline-red-700",
                                aria_label: "Add an event",
                                title: "Add an event",
                                onpointerup: move |_| open_add_event.call(()),
                                Plus { size: 20 }
                            }
                        }
                        if events.is_empty() {
                            p { class: "mt-2 text-sm leading-6 text-slate-600", "No events yet. Tap the plus button to add a flight, hotel stay, train ride, or activity." }
                        } else {
                            ul { class: "mt-3 divide-y divide-slate-100",
                                for event in events.iter().cloned() {
                                    {
                                        let event_id = event.id;
                                        let event_for_edit = event.clone();
                                        let is_removing = removing_event() == Some(event_id);
                                        rsx! {
                                            li { key: "calendar-event-{event_id}", class: "py-3 first:pt-0 last:pb-0",
                                                div { class: "flex items-center justify-between gap-3",
                                                    p { class: "min-w-0 truncate text-sm font-semibold text-slate-900", "{event.name}" }
                                                    div { class: "flex shrink-0 items-center gap-1",
                                                        button {
                                                            r#type: "button",
                                                            class: "flex h-10 w-10 items-center justify-center rounded-xl text-slate-400 transition hover:bg-slate-100 hover:text-slate-700 focus-visible:outline-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                                                            aria_label: "Edit {event.name}",
                                                            title: "Edit event",
                                                            disabled: removing_event().is_some() || event_saving(),
                                                            onpointerup: move |_| open_edit_event.call(event_for_edit.clone()),
                                                            Pencil { size: 18 }
                                                        }
                                                        button {
                                                            r#type: "button",
                                                            class: "flex h-10 w-10 items-center justify-center rounded-xl text-slate-400 transition hover:bg-red-50 hover:text-red-700 focus-visible:outline-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                                                            aria_label: "Remove {event.name}",
                                                            title: "Remove event",
                                                            disabled: removing_event().is_some(),
                                                            onpointerup: move |_| delete_event.call(event_id),
                                                            if is_removing {
                                                                span { class: "text-xs font-semibold", "…" }
                                                            } else {
                                                                Trash2 { size: 18 }
                                                            }
                                                        }
                                                    }
                                                }
                                                p { class: "mt-0.5 text-xs text-slate-500", "{event.start_date} – {event.end_date}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    CalendarEventSheet {
                        open: event_editor_open(),
                        editing: editing_event_id().is_some(),
                        trip_name: trip.name.clone(),
                        name: event_name(),
                        start_date: event_start_date(),
                        end_date: event_end_date(),
                        saving: event_saving(),
                        validation_error: event_error(),
                        on_name_change: move |event: FormEvent| event_name.set(event.value()),
                        on_start_date_change: move |event: FormEvent| event_start_date.set(event.value()),
                        on_end_date_change: move |event: FormEvent| event_end_date.set(event.value()),
                        on_save: move |_| save_event.call(()),
                        on_cancel: move |_| close_event_editor.call(()),
                    }
                    TripDatesSheet {
                        open: edit_dates_open(),
                        trip_name: trip.name.clone(),
                        start_date: edit_start_date(),
                        end_date: edit_end_date(),
                        saving: dates_saving(),
                        validation_error: date_error(),
                        on_start_date_change: move |event: FormEvent| edit_start_date.set(event.value()),
                        on_end_date_change: move |event: FormEvent| edit_end_date.set(event.value()),
                        on_save: move |_| save_trip_dates.call(()),
                        on_cancel: move |_| close_date_editor.call(()),
                    }
                }
            }
        }
    };

    rsx! { {view} }
}

#[component]
fn DateInput(
    id: String,
    label: String,
    value: String,
    disabled: bool,
    on_change: EventHandler<FormEvent>,
) -> Element {
    rsx! {
        div {
            label { class: "block text-sm font-semibold text-slate-800", r#for: "{id}", "{label}" }
            div { class: "relative mt-1",
                input {
                    id: "{id}",
                    r#type: "date",
                    class: "block min-h-12 w-full appearance-none rounded-xl border border-slate-300 bg-white px-4 pr-14 text-base text-slate-900 shadow-sm outline-none transition focus:border-red-700 focus:ring-2 focus:ring-red-100 disabled:cursor-not-allowed disabled:opacity-50",
                    value,
                    disabled,
                    onchange: move |event| on_change.call(event),
                    oninput: move |event| on_change.call(event),
                }
                span { class: "pointer-events-none absolute inset-y-0 right-4 flex items-center text-slate-500",
                    ChevronDown { size: 19 }
                }
            }
        }
    }
}

#[component]
fn TripDatesSheet(
    open: bool,
    trip_name: String,
    start_date: String,
    end_date: String,
    saving: bool,
    validation_error: Option<String>,
    on_start_date_change: EventHandler<FormEvent>,
    on_end_date_change: EventHandler<FormEvent>,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    if !open {
        return rsx! {};
    }

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
            aria_labelledby: "trip-dates-form-title",
            onpointerup: move |event| event.stop_propagation(),
            header { class: "flex items-start justify-between gap-4 border-b border-slate-200 px-5 py-5",
                div { class: "min-w-0",
                    p { class: "text-xs font-bold uppercase tracking-[0.16em] text-red-700", "Trip calendar" }
                    h2 { id: "trip-dates-form-title", class: "mt-1 text-2xl font-bold tracking-tight text-slate-950", "Edit trip dates" }
                    p { class: "mt-2 max-w-xs text-sm leading-5 text-slate-600", "Update the travel window for {trip_name}. Dates are marked on the calendar and trip overview." }
                }
                button {
                    r#type: "button",
                    class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-3xl leading-none text-slate-500 transition hover:bg-slate-100 hover:text-slate-900 focus-visible:outline-2 focus-visible:outline-red-700",
                    aria_label: "Close trip dates form",
                    onpointerup: move |_| on_cancel.call(()),
                    "×"
                }
            }
            div { class: "flex flex-1 flex-col gap-6 overflow-y-auto px-5 py-6",
                DateInput {
                    id: "trip-start-date-edit",
                    label: "Start date",
                    value: start_date,
                    disabled: saving,
                    on_change: move |event: FormEvent| on_start_date_change.call(event),
                }
                DateInput {
                    id: "trip-end-date-edit",
                    label: "End date",
                    value: end_date,
                    disabled: saving,
                    on_change: move |event: FormEvent| on_end_date_change.call(event),
                }
                if let Some(error) = validation_error {
                    p { class: "rounded-xl bg-red-50 p-3 text-sm leading-5 text-red-800", "{error}" }
                }
            }
            footer { class: "grid grid-cols-2 gap-3 border-t border-slate-200 bg-slate-50 px-5 py-4 pb-[calc(1rem+env(safe-area-inset-bottom))]",
                button {
                    r#type: "button",
                    class: "min-h-12 rounded-xl border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-700 transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                    disabled: saving,
                    onpointerup: move |_| on_cancel.call(()),
                    "Cancel"
                }
                button {
                    r#type: "button",
                    class: "min-h-12 rounded-xl bg-red-700 px-4 text-sm font-semibold text-white shadow-sm transition hover:bg-red-800 focus-visible:outline-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                    disabled: saving,
                    onpointerup: move |_| on_save.call(()),
                    if saving { "Saving…" } else { "Save dates" }
                }
            }
        }
    }
}

#[component]
fn CalendarEventSheet(
    open: bool,
    editing: bool,
    trip_name: String,
    name: String,
    start_date: String,
    end_date: String,
    saving: bool,
    validation_error: Option<String>,
    on_name_change: EventHandler<FormEvent>,
    on_start_date_change: EventHandler<FormEvent>,
    on_end_date_change: EventHandler<FormEvent>,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    if !open {
        return rsx! {};
    }

    let title = if editing {
        "Edit event"
    } else {
        "Add an event"
    };
    let action = if editing { "Save event" } else { "Add event" };

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
            aria_labelledby: "calendar-event-form-title",
            onpointerup: move |event| event.stop_propagation(),
            header { class: "flex items-start justify-between gap-4 border-b border-slate-200 px-5 py-5",
                div { class: "min-w-0",
                    p { class: "text-xs font-bold uppercase tracking-[0.16em] text-red-700", "Trip calendar" }
                    h2 { id: "calendar-event-form-title", class: "mt-1 text-2xl font-bold tracking-tight text-slate-950", "{title}" }
                    p { class: "mt-2 max-w-xs text-sm leading-5 text-slate-600", "Add an activity or reservation to {trip_name}. It will appear as a short label on the calendar." }
                }
                button {
                    r#type: "button",
                    class: "flex h-10 w-10 shrink-0 items-center justify-center rounded-xl text-3xl leading-none text-slate-500 transition hover:bg-slate-100 hover:text-slate-900 focus-visible:outline-2 focus-visible:outline-red-700",
                    aria_label: "Close event form",
                    onpointerup: move |_| on_cancel.call(()),
                    "×"
                }
            }
            div { class: "flex flex-1 flex-col gap-6 overflow-y-auto px-5 py-6",
                div { class: "space-y-2",
                    label { class: "block text-sm font-semibold text-slate-800", r#for: "calendar-event-form-name", "Event name" }
                    input {
                        id: "calendar-event-form-name",
                        r#type: "text",
                        class: "mt-1 block min-h-12 w-full rounded-xl border border-slate-300 bg-white px-3 py-3 text-base text-slate-900 shadow-sm outline-none transition placeholder:text-slate-400 focus:border-red-700 focus:ring-2 focus:ring-red-100 disabled:cursor-not-allowed disabled:opacity-50",
                        value: name,
                        placeholder: "e.g. Forbidden City visit",
                        disabled: saving,
                        oninput: move |event| on_name_change.call(event),
                    }
                }
                DateInput {
                    id: "calendar-event-form-start-date",
                    label: "Event start date",
                    value: start_date,
                    disabled: saving,
                    on_change: move |event: FormEvent| on_start_date_change.call(event),
                }
                DateInput {
                    id: "calendar-event-form-end-date",
                    label: "Event end date",
                    value: end_date,
                    disabled: saving,
                    on_change: move |event: FormEvent| on_end_date_change.call(event),
                }
                if let Some(error) = validation_error {
                    p { class: "rounded-xl bg-red-50 p-3 text-sm leading-5 text-red-800", "{error}" }
                }
            }
            footer { class: "grid grid-cols-2 gap-3 border-t border-slate-200 bg-slate-50 px-5 py-4 pb-[calc(1rem+env(safe-area-inset-bottom))]",
                button {
                    r#type: "button",
                    class: "min-h-12 rounded-xl border border-slate-300 bg-white px-4 text-sm font-semibold text-slate-700 transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                    disabled: saving,
                    onpointerup: move |_| on_cancel.call(()),
                    "Cancel"
                }
                button {
                    r#type: "button",
                    class: "min-h-12 rounded-xl bg-red-700 px-4 text-sm font-semibold text-white shadow-sm transition hover:bg-red-800 focus-visible:outline-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                    disabled: saving,
                    onpointerup: move |_| on_save.call(()),
                    if saving { "Saving…" } else { "{action}" }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum EventCommand {
    Create(NewCalendarEvent),
    Update(UpdateCalendarEvent),
}

fn event_occurs_on(event: &CalendarEvent, date: CalendarDate) -> bool {
    let Ok(start_date) = CalendarDate::parse(&event.start_date) else {
        return false;
    };
    let Ok(end_date) = CalendarDate::parse(&event.end_date) else {
        return false;
    };
    start_date <= date && date <= end_date
}

fn abbreviate_event_name(name: &str) -> String {
    const MAX_CHARS: usize = 11;
    let mut characters = name.chars();
    let abbreviated = characters.by_ref().take(MAX_CHARS).collect::<String>();
    if characters.next().is_some() {
        format!("{abbreviated}…")
    } else {
        abbreviated
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn months_roll_over_at_year_boundaries() {
        assert_eq!(
            CalendarMonth {
                year: 2027,
                month: 1
            }
            .previous(),
            CalendarMonth {
                year: 2026,
                month: 12
            }
        );
        assert_eq!(
            CalendarMonth {
                year: 2027,
                month: 12
            }
            .next(),
            CalendarMonth {
                year: 2028,
                month: 1
            }
        );
    }

    #[test]
    fn event_labels_are_unicode_safe_and_events_cover_their_full_range() {
        let event = CalendarEvent {
            id: 1,
            trip_id: 1,
            name: "长城一日游和晚餐".to_string(),
            start_date: "2027-04-02".to_string(),
            end_date: "2027-04-04".to_string(),
            created_at: 0,
            updated_at: 0,
        };
        assert_eq!(abbreviate_event_name(&event.name), event.name);
        assert!(event_occurs_on(
            &event,
            CalendarDate::parse("2027-04-03").unwrap()
        ));
        assert!(!event_occurs_on(
            &event,
            CalendarDate::parse("2027-04-05").unwrap()
        ));
    }
}
