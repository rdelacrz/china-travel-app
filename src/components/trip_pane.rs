use crate::app::Route;
use crate::domain::TripOverview;
use dioxus::prelude::*;

#[component]
pub fn TripPane(overview: TripOverview) -> Element {
    let trip_id = overview.trip.id;
    rsx! {
        article { class: "rounded-2xl border border-slate-200 bg-white p-4 shadow-sm",
            div { class: "flex items-start justify-between gap-4",
                div { class: "min-w-0",
                    h2 { class: "truncate text-lg font-semibold text-slate-950", "{overview.trip.name}" }
                    p { class: "mt-1 text-sm text-slate-500",
                        "{overview.checklist_outstanding()} checklist items outstanding · {overview.document_count} documents"
                    }
                }
                span { class: "shrink-0 rounded-full bg-red-50 px-2.5 py-1 text-xs font-semibold text-red-700",
                    "{overview.checklist_completed}/{overview.checklist_total} done"
                }
            }
            div { class: "mt-4 grid grid-cols-2 gap-3",
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
