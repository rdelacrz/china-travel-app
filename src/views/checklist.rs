use crate::app::Route;
use crate::components::button::Button;
use crate::components::checklist_item_pane::{ChecklistItemPane, DragPoint};
use crate::components::toast::{use_toast, ToastOptions};
use crate::domain::ChecklistItem;
use crate::state::{use_database, use_revision};
use dioxus::prelude::*;
use dioxus_primitives::checkbox::CheckboxState;

use std::collections::{HashMap, HashSet};
use std::time::Duration;

const AXIS_LOCK_DISTANCE: f64 = 8.0;
const DELETE_SWIPE_DISTANCE: f64 = 72.0;
// The standard checklist row is 4rem (64px) with Tailwind's `space-y-3` 12px gap.
// Keeping the preview pitch equal to that layout pitch moves a displaced row into the
// dragged row's original slot instead of merely near it.
const REORDER_ROW_DISTANCE: f64 = 76.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DragAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq)]
struct DragState {
    id: i64,
    item: ChecklistItem,
    pointer_id: i32,
    start_x: f64,
    start_y: f64,
    min_y: f64,
    max_y: f64,
    x: f64,
    y: f64,
    axis: Option<DragAxis>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct DismissingDelete {
    id: i64,
    direction: i8,
}

fn reorder_indices(
    items: &[ChecklistItem],
    active: &DragState,
    pointer_y: f64,
) -> Option<(usize, usize)> {
    if active.axis != Some(DragAxis::Vertical) {
        return None;
    }
    let from = items.iter().position(|row| row.id == active.id)?;
    let shift = ((pointer_y - active.start_y) / REORDER_ROW_DISTANCE).round() as isize;
    if shift == 0 {
        return None;
    }
    let target = (from as isize + shift).clamp(0, items.len().saturating_sub(1) as isize) as usize;
    (target != from).then_some((from, target))
}

fn constrained_vertical_y(active: &DragState, pointer_y: f64) -> f64 {
    pointer_y.clamp(active.min_y, active.max_y)
}

fn reorder_preview_items(items: &[ChecklistItem], active: &DragState) -> Vec<ChecklistItem> {
    let Some((from, target)) = reorder_indices(items, active, active.y) else {
        return items.to_vec();
    };
    let mut preview = items.to_vec();
    let moved = preview.remove(from);
    preview.insert(target, moved);
    preview
}

fn active_preview_offset_y(items: &[ChecklistItem], active: &DragState) -> f64 {
    let preview_slot_offset = reorder_indices(items, active, active.y)
        .map(|(from, target)| (target as isize - from as isize) as f64 * REORDER_ROW_DISTANCE)
        .unwrap_or_default();
    (active.y - active.start_y) - preview_slot_offset
}

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
    let mut dismissing_delete = use_signal(|| None::<DismissingDelete>);
    let mut delete_token = use_signal(|| 0_u64);
    let mut drag = use_signal(|| None::<DragState>);
    let mut saving_order = use_signal(|| false);

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
                            editing.set(None);
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

    let schedule_delete = use_callback({
        let database = database.clone();
        move |(item, direction): (ChecklistItem, i8)| {
            if pending_delete().is_some()
                || dismissing_delete().is_some()
                || busy_rows.with(|rows| rows.contains(&item.id))
            {
                return;
            }
            let token = delete_token() + 1;
            delete_token.set(token);
            dismissing_delete.set(Some(DismissingDelete {
                id: item.id,
                direction,
            }));
            let database = database.clone();
            spawn(async move {
                tokio::time::sleep(Duration::from_millis(260)).await;
                if delete_token() != token {
                    return;
                }
                pending_delete.set(Some(item.clone()));
                dismissing_delete.set(None);
                tokio::time::sleep(Duration::from_secs(5)).await;
                let still_pending = delete_token() == token
                    && pending_delete()
                        .as_ref()
                        .map(|pending| pending.id == item.id)
                        .unwrap_or(false);
                if !still_pending {
                    return;
                }
                busy_rows.write().insert(item.id);
                let result = database.delete_checklist_item(item.id).await;
                busy_rows.write().remove(&item.id);
                if delete_token() != token {
                    return;
                }
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

    let undo_delete = use_callback(move |_: ()| {
        delete_token.set(delete_token() + 1);
        dismissing_delete.set(None);
        pending_delete.set(None);
    });

    let move_drag = use_callback({
        move |point: DragPoint| {
            let Some(mut active) = drag() else {
                return;
            };
            if active.pointer_id != point.pointer_id {
                return;
            }
            let delta_x = point.x - active.start_x;
            let delta_y = point.y - active.start_y;
            if active.axis.is_none() && delta_x.abs().max(delta_y.abs()) >= AXIS_LOCK_DISTANCE {
                active.axis = Some(if delta_x.abs() >= delta_y.abs() {
                    DragAxis::Horizontal
                } else {
                    DragAxis::Vertical
                });
            }
            match active.axis {
                Some(DragAxis::Horizontal) => {
                    let next_x = active.start_x + delta_x.clamp(-160.0, 160.0);
                    active.x = next_x;
                    drag.set(Some(active.clone()));
                    if delta_x.abs() >= DELETE_SWIPE_DISTANCE {
                        drag.set(None);
                        let direction = if delta_x >= 0.0 { 1 } else { -1 };
                        schedule_delete.call((active.item.clone(), direction));
                    }
                }
                Some(DragAxis::Vertical) => {
                    active.y = constrained_vertical_y(&active, point.y);
                    drag.set(Some(active));
                }
                None => {}
            }
        }
    });

    let end_drag = use_callback({
        let database = database.clone();
        let data_for_reorder = data;
        move |point: DragPoint| {
            let Some(active) = drag() else {
                return;
            };
            if active.pointer_id != point.pointer_id {
                return;
            }
            drag.set(None);
            if active.axis != Some(DragAxis::Vertical) || saving_order() {
                return;
            }
            let ordered_ids = {
                let data = data_for_reorder.value();
                let data = data.read_unchecked();
                let Some(Ok((_, items))) = data.as_ref() else {
                    return;
                };
                let Some((from, target)) = reorder_indices(items, &active, point.y) else {
                    return;
                };
                let mut ordered_ids = items.iter().map(|row| row.id).collect::<Vec<_>>();
                let moved = ordered_ids.remove(from);
                ordered_ids.insert(target, moved);
                ordered_ids
            };
            saving_order.set(true);
            let database = database.clone();
            spawn(async move {
                if let Err(error) = database.reorder_checklist_items(trip_id, ordered_ids).await {
                    toast.error(
                        "Checklist order was not saved".to_string(),
                        ToastOptions::default().description(error.to_string()),
                    );
                } else {
                    revision.set(revision() + 1);
                }
                saving_order.set(false);
            });
        }
    });

    let view = match &*data.value().read_unchecked() {
        None => {
            rsx! { p { class: "rounded-2xl border border-slate-200 bg-white p-5 text-sm text-slate-600", "Loading checklist…" } }
        }
        Some(Err(crate::error::DbError::NotFound { entity: "trip", .. })) => rsx! {
            section { class: "rounded-2xl border border-amber-200 bg-amber-50 p-5",
                h1 { class: "text-lg font-bold text-amber-950", "Trip not found" }
                p { class: "mt-2 text-sm leading-6 text-amber-900", "This checklist route no longer points to a saved trip." }
                Link { to: Route::Home {}, class: "mt-4 inline-flex min-h-12 items-center rounded-xl bg-amber-900 px-4 text-sm font-semibold text-white", "Back to trips" }
            }
        },
        Some(Err(error)) => rsx! {
            section { class: "rounded-2xl border border-red-200 bg-red-50 p-5",
                h1 { class: "text-lg font-bold text-red-900", "Checklist unavailable" }
                p { class: "mt-2 text-sm leading-6 text-red-800", "{error}" }
                Button { class: "mt-4 min-h-12", on_press: move |_| data.restart(), "Retry" }
            }
        },
        Some(Ok((trip, items))) => {
            let completed = items.iter().filter(|item| item.is_checked).count();
            let total = items.len();
            let progress = completed
                .saturating_mul(100)
                .checked_div(total)
                .unwrap_or_default();
            let active_drag = drag();
            let preview_items = active_drag
                .as_ref()
                .map(|active| reorder_preview_items(items, active))
                .unwrap_or_else(|| items.to_vec());
            rsx! {
                section { class: "space-y-5 pb-28",
                    if drag()
                        .as_ref()
                        .map(|active| active.axis == Some(DragAxis::Vertical))
                        .unwrap_or(false)
                    {
                        div {
                            class: "fixed inset-0",
                            style: "z-index: 900; touch-action: none;",
                            aria_hidden: "true",
                            onpointermove: move |event| move_drag.call(DragPoint::from(&event)),
                            onpointerup: move |event| end_drag.call(DragPoint::from(&event)),
                            onpointercancel: move |event| end_drag.call(DragPoint::from(&event)),
                        }
                    }
                    div { class: "flex items-start justify-between gap-3",
                        div {
                            p { class: "text-sm font-semibold uppercase tracking-[0.16em] text-red-700", "Checklist" }
                            h1 { class: "mt-1 text-2xl font-bold tracking-tight text-slate-950", "{trip.name}" }
                            p { class: "mt-2 text-sm text-slate-600", "{completed} of {total} items complete" }
                        }
                        Link {
                            to: Route::Home {},
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
                    h2 { class: "text-xl font-bold text-slate-950", "Checklist items" }
                    if items.is_empty() && new_draft().is_none() {
                        div { class: "rounded-2xl border border-dashed border-slate-300 bg-white p-8 text-center",
                            p { class: "text-3xl", "🎒" }
                            p { class: "mt-3 text-sm leading-6 text-slate-600", "Nothing added yet. Add passports, medication, adapters, or anything else you need." }
                        }
                    }
                    ul {
                        class: "space-y-3",
                        onpointermove: move |event| move_drag.call(DragPoint::from(&event)),
                        onpointerup: move |event| end_drag.call(DragPoint::from(&event)),
                        onpointercancel: move |event| end_drag.call(DragPoint::from(&event)),
                        for item in preview_items
                            .iter()
                            .filter(|item| {
                                pending_delete()
                                    .as_ref()
                                    .map(|pending| pending.id != item.id)
                                    .unwrap_or(true)
                            })
                            .cloned()
                        {
                            {
                                let item_id = item.id;
                                let item_for_edit = item.clone();
                                let item_for_drag = item.clone();
                                let item_index = items
                                    .iter()
                                    .position(|candidate| candidate.id == item_id)
                                    .unwrap_or_default();
                                let minimum_drag_offset_y =
                                    -(item_index as f64 * REORDER_ROW_DISTANCE);
                                let maximum_drag_offset_y = (items
                                    .len()
                                    .saturating_sub(1)
                                    .saturating_sub(item_index)
                                    as f64)
                                    * REORDER_ROW_DISTANCE;
                                let active_drag = active_drag.clone();
                                let row_dragging = active_drag
                                    .as_ref()
                                    .map(|active| active.id == item_id)
                                    .unwrap_or(false);
                                let drag_offset_x = active_drag
                                    .as_ref()
                                    .filter(|active| active.id == item_id)
                                    .map(|active| active.x - active.start_x)
                                    .unwrap_or_default();
                                let drag_offset_y = active_drag
                                    .as_ref()
                                    .filter(|active| active.id == item_id)
                                    .map(|active| active_preview_offset_y(items, active))
                                    .unwrap_or_default();
                                let horizontal_dragging = active_drag
                                    .as_ref()
                                    .filter(|active| active.id == item_id)
                                    .map(|active| active.axis == Some(DragAxis::Horizontal))
                                    .unwrap_or(false);
                                let dismissing_direction = dismissing_delete()
                                    .filter(|dismissing| dismissing.id == item_id)
                                    .map(|dismissing| dismissing.direction);
                                let order_saving = saving_order();
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
                                        busy: busy_rows.with(|rows| rows.contains(&item_id)) || order_saving,
                                        checkbox_disabled: busy_rows.with(|rows| rows.contains(&item_id)) || order_saving,
                                        validation_error: row_errors.with(|errors| errors.get(&item_id).cloned()),
                                        dragging: row_dragging,
                                        drag_offset_x,
                                        drag_offset_y,
                                        horizontal_dragging,
                                        dismissing_direction,
                                        on_begin_edit: move |_| {
                                            drafts
                                                .write()
                                                .insert(item_id, item_for_edit.text.clone());
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
                                        on_drag_start: move |point: DragPoint| {
                                            if editing().is_none()
                                                && pending_delete().is_none()
                                                && dismissing_delete().is_none()
                                                && !saving_order()
                                                && drag().is_none()
                                            {
                                                drag.set(Some(DragState {
                                                    id: item_id,
                                                    item: item_for_drag.clone(),
                                                    pointer_id: point.pointer_id,
                                                    start_x: point.x,
                                                    start_y: point.y,
                                                    min_y: point.y + minimum_drag_offset_y,
                                                    max_y: point.y + maximum_drag_offset_y,
                                                    x: point.x,
                                                    y: point.y,
                                                    axis: None,
                                                }));
                                            }
                                        },
                                    }
                                }
                            }
                        }
                        if new_draft().is_some() {
                            ChecklistItemPane {
                                item: ChecklistItem {
                                    id: -1,
                                    trip_id,
                                    text: String::new(),
                                    is_checked: false,
                                    sort_order: items.len() as i64,
                                    created_at: 0,
                                    updated_at: 0,
                                },
                                editing: true,
                                draft: new_draft().unwrap_or_default(),
                                busy: busy_rows.with(|rows| rows.contains(&-1)),
                                checkbox_disabled: true,
                                validation_error: row_errors.with(|errors| errors.get(&-1).cloned()),
                                dragging: false,
                                drag_offset_x: 0.0,
                                drag_offset_y: 0.0,
                                horizontal_dragging: false,
                                dismissing_direction: None,
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
                                on_drag_start: move |_| {},
                            }
                        }
                    }
                    div { class: "fixed inset-x-0 bottom-0 z-30 border-t border-slate-200 bg-slate-50/95 px-4 py-3 pb-[calc(0.75rem+env(safe-area-inset-bottom))] shadow-[0_-8px_24px_rgb(15_23_42_/_8%)] backdrop-blur",
                        div { class: "mx-auto max-w-3xl",
                            button {
                                r#type: "button",
                                class: "flex min-h-12 w-full items-center justify-center rounded-xl bg-red-700 px-4 text-sm font-semibold text-white shadow-sm transition hover:bg-red-800 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:opacity-50",
                                disabled: new_draft().is_some(),
                                onpointerup: move |_| {
                                    if new_draft().is_none() {
                                        new_draft.set(Some(String::new()));
                                        editing.set(Some(-1));
                                    }
                                },
                                "+ Add item"
                            }
                        }
                    }
                    if let Some(item) = pending_delete() {
                        div {
                            class: "fixed inset-x-0 bottom-0 z-50 border-t border-slate-300 bg-slate-950 px-4 py-4 pb-[calc(1rem+env(safe-area-inset-bottom))] text-white shadow-[0_-10px_30px_rgb(15_23_42_/_20%)]",
                            div { class: "mx-auto flex max-w-3xl items-center justify-between gap-4",
                                p { class: "min-w-0 text-sm font-semibold", "{item.text} removed" }
                                button {
                                    r#type: "button",
                                    class: "flex min-h-11 shrink-0 items-center rounded-xl bg-white px-4 text-sm font-semibold text-slate-950 shadow-sm transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-white",
                                    onpointerup: move |_| undo_delete.call(()),
                                    onclick: move |_| undo_delete.call(()),
                                    "Undo"
                                }
                            }
                        }
                    }
                }
            }
        }
    };

    rsx! { {view} }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(id: i64) -> ChecklistItem {
        ChecklistItem {
            id,
            trip_id: 1,
            text: format!("Item {id}"),
            is_checked: false,
            sort_order: id,
            created_at: 0,
            updated_at: 0,
        }
    }

    fn vertical_drag(id: i64, start_y: f64, y: f64) -> DragState {
        DragState {
            id,
            item: item(id),
            pointer_id: 1,
            start_x: 0.0,
            start_y,
            min_y: start_y - (5.0 * REORDER_ROW_DISTANCE),
            max_y: start_y + (5.0 * REORDER_ROW_DISTANCE),
            x: 0.0,
            y,
            axis: Some(DragAxis::Vertical),
        }
    }

    #[test]
    fn preview_replaces_the_dragged_row_with_the_target_row() {
        let items = vec![item(1), item(2), item(3)];
        let active = vertical_drag(1, 100.0, 176.0);

        assert_eq!(reorder_indices(&items, &active, active.y), Some((0, 1)));
        assert_eq!(
            reorder_preview_items(&items, &active)
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec![2, 1, 3]
        );
        assert_eq!(active_preview_offset_y(&items, &active), 0.0);
    }

    #[test]
    fn preview_waits_until_the_drag_reaches_an_adjacent_row() {
        let items = vec![item(1), item(2), item(3)];
        let active = vertical_drag(3, 100.0, 63.0);

        assert_eq!(reorder_indices(&items, &active, active.y), None);
        assert_eq!(
            reorder_preview_items(&items, &active)
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec![1, 2, 3]
        );

        let active = vertical_drag(3, 100.0, 16.0);
        assert_eq!(reorder_indices(&items, &active, active.y), Some((2, 1)));
        assert_eq!(
            reorder_preview_items(&items, &active)
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec![1, 3, 2]
        );
    }

    #[test]
    fn preview_reorders_across_the_full_checklist_with_the_first_displaced_row_in_the_old_slot() {
        let items = (1..=6).map(item).collect::<Vec<_>>();
        let active = vertical_drag(1, 100.0, 100.0 + (REORDER_ROW_DISTANCE * 5.0));

        assert_eq!(reorder_indices(&items, &active, active.y), Some((0, 5)));
        assert_eq!(
            reorder_preview_items(&items, &active)
                .iter()
                .map(|item| item.id)
                .collect::<Vec<_>>(),
            vec![2, 3, 4, 5, 6, 1]
        );
        assert_eq!(active_preview_offset_y(&items, &active), 0.0);
    }

    #[test]
    fn vertical_drag_is_constrained_to_the_checklist_rows() {
        let active = DragState {
            min_y: 100.0,
            max_y: 252.0,
            ..vertical_drag(1, 100.0, 100.0)
        };

        assert_eq!(constrained_vertical_y(&active, 20.0), 100.0);
        assert_eq!(constrained_vertical_y(&active, 180.0), 180.0);
        assert_eq!(constrained_vertical_y(&active, 340.0), 252.0);
    }
}
