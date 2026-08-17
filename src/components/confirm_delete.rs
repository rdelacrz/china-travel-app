use crate::components::alert_dialog::{
    AlertDialog, AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogDescription,
    AlertDialogTitle,
};
use crate::domain::ChecklistItem;
use dioxus::prelude::*;

#[component]
pub fn ConfirmDeleteDialog(
    item: Option<ChecklistItem>,
    on_confirm: EventHandler<MouseEvent>,
    on_cancel: EventHandler<()>,
) -> Element {
    let open = item.is_some();
    let item_name = item
        .as_ref()
        .map(|value| value.text.clone())
        .unwrap_or_else(|| "this checklist item".to_string());
    rsx! {
        AlertDialog {
            open,
            on_open_change: move |is_open: bool| {
                if !is_open {
                    on_cancel.call(());
                }
            },
            AlertDialogTitle { "Delete checklist item?" }
            AlertDialogDescription { "Delete \"{item_name}\"? This cannot be undone." }
            AlertDialogActions {
                AlertDialogCancel {
                    on_click: move |_| on_cancel.call(()),
                    "Cancel"
                }
                AlertDialogAction {
                    class: "rounded-xl bg-red-700 px-4 py-3 text-sm font-semibold text-white hover:bg-red-800 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700",
                    on_click: move |event| on_confirm.call(event),
                    "Delete"
                }
            }
        }
    }
}
