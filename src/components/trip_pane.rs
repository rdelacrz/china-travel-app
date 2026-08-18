use crate::app::Route;
use crate::components::button::{Button, ButtonSize, ButtonVariant};
use crate::domain::TripOverview;
use dioxus::prelude::*;
use dioxus_icons::lucide::{CalendarDays, Trash2};

#[component]
pub fn TripPane(overview: TripOverview, on_delete: EventHandler<()>) -> Element {
    let trip_id = overview.trip.id;
    rsx! {
        article { class: "rounded-3xl border border-slate-200 bg-white p-5 shadow-sm",
            div { class: "flex items-center justify-between gap-3",
                div { class: "min-w-0 flex-1",
                    h2 { class: "truncate text-xl font-bold tracking-tight text-slate-950", "{overview.trip.name}" }
                }
                div { class: "flex shrink-0 items-center gap-1",
                    Link {
                        to: Route::Calendar { trip_id },
                        class: "flex h-10 w-10 items-center justify-center rounded-xl text-slate-400 transition hover:bg-red-50 hover:text-red-700 focus-visible:outline-2 focus-visible:outline-red-700",
                        aria_label: "Open trip calendar",
                        title: "Open trip calendar",
                        CalendarDays { size: 19 }
                    }
                    Button {
                        variant: ButtonVariant::Ghost,
                        size: ButtonSize::Icon,
                        class: "h-10 w-10 text-slate-400 hover:bg-red-50 hover:text-red-700",
                        aria_label: "Delete trip",
                        title: "Delete trip",
                        on_press: move |_| on_delete.call(()),
                        Trash2 { size: 18 }
                    }
                }
            }
            div { class: "mt-3",
                span { class: "inline-flex rounded-full bg-red-50 px-3 py-1.5 text-xs font-bold text-red-700",
                    "{overview.checklist_completed}/{overview.checklist_total} done"
                }
            }
            div { class: "mt-3 w-full space-y-1",
                p { class: "w-full text-sm leading-6 text-slate-500",
                    "{overview.checklist_outstanding()} checklist items outstanding · {overview.document_count} documents"
                }
                if let Some(date_range) = overview.trip.date_range_label() {
                    p { class: "w-full text-xs font-semibold text-red-700", "{date_range}" }
                }
            }
            div { class: "mt-5 grid grid-cols-2 gap-3",
                Link {
                    to: Route::Checklist { trip_id },
                    class: "flex min-h-12 items-center justify-center rounded-xl bg-red-700 px-3 text-sm font-semibold text-white transition hover:bg-red-800 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700",
                    "Checklist"
                }
                Link {
                    to: Route::Documentation { trip_id },
                    class: "flex min-h-12 items-center justify-center rounded-xl border border-slate-300 bg-white px-3 text-sm font-semibold text-slate-700 transition hover:bg-slate-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700",
                    "Documentation"
                }
            }
        }
    }
}
