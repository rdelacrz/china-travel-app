use crate::components::button::{Button, ButtonSize};
use crate::components::checklist_item_pane::ChecklistItemPane;
use crate::components::confirm_delete::ConfirmDeleteDialog;
use crate::domain::ChecklistItem;
use crate::state::{use_database, use_revision};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::collections::{HashMap, HashSet};

#[component]
pub fn Checklist(trip_id: i64) -> Element {
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
                let items = database.list_checklist_items(trip_id).await?;
                Ok::<_, crate::error::DbError>((trip, items))
            }
        }
    });
    let mut editing = use_signal(|| None::<i64>);
    let mut drafts = use_signal(HashMap::<i64, String>::new);
    let mut new_draft = use_signal(|| None::<String>);
    let mut row_errors = use_signal(HashMap::<i64, String>::new);
    let mut busy_rows = use_signal(HashSet::<i64>::new);
    let mut checked_overrides = use_signal(HashMap::<i64, bool>::new);
    let mut pending_delete = use_signal(|| None::<ChecklistItem>);

    let commit_database = database.clone();
    let commit = use_callback({
        let database = commit_database;
        move |id: i64| {
            if busy_rows.with(|rows| rows.contains(&id)) {
                return;
            }
            if id == -1 && new_draft().is_none() {
                return;
            }
            if id != -1 && editing() != Some(id) {
                return;
            }
            let raw = if id == -1 {
                new_draft().unwrap_or_default()
            } else {
                drafts.with(|values| values.get(&id).cloned().unwrap_or_default())
            };
            let cleaned = match ChecklistItem::validate_text(&raw) {
                Ok(value) => value,
                Err(error) => {
                    row_errors.write().insert(id, error.to_string());
                    return;
                }
            };
            row_errors.write().remove(&id);
            busy_rows.write().insert(id);
            let database = database.clone();
            spawn(async move {
                let result = if id == -1 {
                    database
                        .add_checklist_item(trip_id, &cleaned)
                        .await
                        .map(|_| ())
                } else {
                    database
                        .rename_checklist_item(id, &cleaned)
                        .await
                        .map(|_| ())
                };
                busy_rows.write().remove(&id);
                match result {
                    Ok(()) => {
                        if id == -1 {
                            new_draft.set(None);
                        } else {
                            drafts.write().remove(&id);
                            editing.set(None);
                        }
                        revision.set(revision() + 1);
                    }
                    Err(error) => {
                        row_errors.write().insert(id, error.to_string());
                    }
                }
            });
        }
    });

    let toggle_database = database.clone();
    let toggle = use_callback({
        let database = toggle_database;
        move |(id, state): (i64, CheckboxState)| {
            if id < 0 || busy_rows.with(|rows| rows.contains(&id)) {
                return;
            }
            let checked = matches!(state, CheckboxState::Checked);
            checked_overrides.write().insert(id, checked);
            busy_rows.write().insert(id);
            let database = database.clone();
            spawn(async move {
                let result = database
                    .set_checklist_checked(id, checked)
                    .await
                    .map(|_| ());
                busy_rows.write().remove(&id);
                match result {
                    Ok(()) => {
                        checked_overrides.write().remove(&id);
                        revision.set(revision() + 1);
                    }
                    Err(error) => {
                        checked_overrides.write().remove(&id);
                        toast.error(
                            "Checklist update failed".to_string(),
                            ToastOptions::default().description(error.to_string()),
                        );
                    }
                }
            });
        }
    });

    let delete_item = use_callback({
        move |_: MouseEvent| {
            let Some(item) = pending_delete() else {
                return;
            };
            if busy_rows.with(|rows| rows.contains(&item.id)) {
                return;
            }
            busy_rows.write().insert(item.id);
            let database = database.clone();
            spawn(async move {
                let result = database.delete_checklist_item(item.id).await;
                busy_rows.write().remove(&item.id);
                pending_delete.set(None);
                match result {
                    Ok(()) => revision.set(revision() + 1),
                    Err(error) => toast.error(
                        "Checklist item was not deleted".to_string(),
                        ToastOptions::default().description(error.to_string()),
                    ),
                }
            });
        }
    });

    let view = match &*data.value().read_unchecked() {
        None => {
            rsx! { p { class: "rounded-2xl border border-slate-200 bg-white p-5 text-sm text-slate-600", "Loading checklist…" } }
        }
        Some(Err(error)) => rsx! {
            section { class: "rounded-2xl border border-red-200 bg-red-50 p-5",
                h1 { class: "text-lg font-bold text-red-900", "Checklist unavailable" }
                p { class: "mt-2 text-sm leading-6 text-red-800", "{error}" }
                Button { class: "mt-4 min-h-12", onclick: move |_| data.restart(), "Retry" }
            }
        },
        Some(Ok((trip, items))) => {
            let completed = items.iter().filter(|item| item.is_checked).count();
            let total = items.len();
            let progress = completed
                .saturating_mul(100)
                .checked_div(total)
                .unwrap_or_default();
            rsx! {
                section { class: "space-y-5",
                    div { class: "flex items-start justify-between gap-3",
                        div {
                            p { class: "text-sm font-semibold uppercase tracking-[0.16em] text-red-700", "Checklist" }
                            h1 { class: "mt-1 text-2xl font-bold tracking-tight text-slate-950", "{trip.name}" }
                            p { class: "mt-2 text-sm text-slate-600", "{completed} of {total} items complete" }
                        }
                        a {
                            href: "/",
                            class: "flex min-h-12 items-center rounded-xl px-3 text-sm font-semibold text-slate-700 hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-red-700",
                            "Back"
                        }
                    }
                    div { class: "rounded-2xl border border-slate-200 bg-white p-4 shadow-sm",
                        div { class: "flex items-center justify-between text-xs font-semibold text-slate-500",
                            span { "Preparation progress" }
                            span { "{progress}%" }
                        }
                        div { class: "mt-3 h-2 overflow-hidden rounded-full bg-slate-100",
                            div { class: "h-full rounded-full bg-red-700 transition-all", style: "width: {progress}%" }
                        }
                    }
                    div { class: "flex items-center justify-between gap-3",
                        h2 { class: "text-xl font-bold text-slate-950", "Bring with you" }
                        Button {
                            size: ButtonSize::Sm,
                            disabled: new_draft().is_some(),
                            onclick: move |_| {
                                if new_draft().is_none() {
                                    new_draft.set(Some(String::new()));
                                    editing.set(Some(-1));
                                }
                            },
                            "+ Add item"
                        }
                    }
                    if items.is_empty() && new_draft().is_none() {
                        div { class: "rounded-2xl border border-dashed border-slate-300 bg-white p-8 text-center",
                            p { class: "text-3xl", "🎒" }
                            p { class: "mt-3 text-sm leading-6 text-slate-600", "Nothing added yet. Add passports, medication, adapters, or anything else you need." }
                        }
                    }
                    ul { class: "space-y-3",
                        if new_draft().is_some() {
                            ChecklistItemPane {
                                item: ChecklistItem {
                                    id: -1,
                                    trip_id,
                                    text: String::new(),
                                    is_checked: false,
                                    sort_order: 0,
                                    created_at: 0,
                                    updated_at: 0,
                                },
                                editing: true,
                                draft: new_draft().unwrap_or_default(),
                                busy: busy_rows.with(|rows| rows.contains(&-1)),
                                checkbox_disabled: true,
                                validation_error: row_errors.with(|errors| errors.get(&-1).cloned()),
                                on_begin_edit: move |_| {},
                                on_draft_change: move |event: FormEvent| new_draft.set(Some(event.value())),
                                on_commit: move |_| commit.call(-1),
                                on_keydown: move |event: KeyboardEvent| {
                                    if event.key() == Key::Enter {
                                        event.prevent_default();
                                        commit.call(-1);
                                    }
                                },
                                on_checked_change: move |_| {},
                                on_delete: move |_| {
                                    new_draft.set(None);
                                    editing.set(None);
                                    row_errors.write().remove(&-1);
                                },
                            }
                        }
                        for item in items.iter().cloned() {
                            {
                                let item_id = item.id;
                                let item_for_delete = item.clone();
                                let checked = checked_overrides
                                    .with(|values| values.get(&item_id).copied())
                                    .unwrap_or(item.is_checked);
                                let mut displayed_item = item.clone();
                                displayed_item.is_checked = checked;
                                rsx! {
                                    ChecklistItemPane {
                                        key: "item-{item_id}",
                                        item: displayed_item,
                                        editing: editing() == Some(item_id),
                                        draft: drafts.with(|values| values.get(&item_id).cloned().unwrap_or_else(|| item.text.clone())),
                                        busy: busy_rows.with(|rows| rows.contains(&item_id)),
                                        checkbox_disabled: busy_rows.with(|rows| rows.contains(&item_id)),
                                        validation_error: row_errors.with(|errors| errors.get(&item_id).cloned()),
                                        on_begin_edit: move |_| {
                                            drafts.write().insert(item_id, item.text.clone());
                                            editing.set(Some(item_id));
                                            row_errors.write().remove(&item_id);
                                        },
                                        on_draft_change: move |event: FormEvent| {
                                            drafts.write().insert(item_id, event.value());
                                        },
                                        on_commit: move |_| commit.call(item_id),
                                        on_keydown: move |event: KeyboardEvent| {
                                            if event.key() == Key::Enter {
                                                event.prevent_default();
                                                commit.call(item_id);
                                            }
                                        },
                                        on_checked_change: move |state| toggle.call((item_id, state)),
                                        on_delete: move |_| pending_delete.set(Some(item_for_delete.clone())),
                                    }
                                }
                            }
                        }
                    }
                    ConfirmDeleteDialog {
                        item: pending_delete(),
                        on_confirm: move |event| delete_item.call(event),
                        on_cancel: move |_| pending_delete.set(None),
                    }
                }
            }
        }
    };

    rsx! { {view} }
}
