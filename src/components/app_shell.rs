use crate::app::Route;
use crate::components::toast::ToastProvider;
use dioxus::prelude::*;

#[component]
pub fn AppShell() -> Element {
    rsx! {
        ToastProvider {
            div { class: "min-h-dvh bg-slate-50 text-slate-900 safe-top safe-bottom",
                header { class: "sticky top-0 z-20 border-b border-slate-200/80 bg-slate-50/95 px-4 py-3 backdrop-blur",
                    div { class: "mx-auto flex max-w-3xl items-center justify-between",
                        Link { to: Route::Home {}, class: "text-lg font-bold tracking-tight text-red-700", "China Travel" }
                        span { class: "text-xs font-medium uppercase tracking-[0.18em] text-slate-500", "Companion" }
                    }
                }
                main { class: "mx-auto flex w-full max-w-3xl flex-1 flex-col px-4 pb-24 pt-5",
                    Outlet::<Route> {}
                }
                nav { class: "fixed inset-x-0 bottom-0 z-20 border-t border-slate-200 bg-white/95 px-4 py-3 backdrop-blur",
                    div { class: "mx-auto flex max-w-3xl justify-center",
                        Link {
                            to: Route::Home {},
                            class: "flex min-h-12 min-w-24 items-center justify-center rounded-xl px-4 text-sm font-semibold text-slate-700 transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700",
                            "Overview"
                        }
                    }
                }
            }
        }
    }
}
