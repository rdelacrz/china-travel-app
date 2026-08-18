use crate::app::Route;
use crate::components::toast::ToastProvider;
use dioxus::prelude::*;

const APP_ICON: Asset = asset!("/assets/icon.png");

#[component]
pub fn AppShell() -> Element {
    rsx! {
        ToastProvider {
            div { class: "min-h-dvh bg-slate-50 text-slate-900 safe-top safe-bottom",
                header { class: "sticky top-0 z-20 border-b border-slate-200/80 bg-slate-50/95 px-4 py-3 backdrop-blur",
                    div { class: "mx-auto flex max-w-3xl items-center justify-between",
                        Link {
                            to: Route::Home {},
                            class: "flex min-w-0 items-center gap-2.5 rounded-xl focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700",
                            img {
                                class: "h-9 w-9 shrink-0 rounded-[0.7rem] shadow-sm",
                                src: APP_ICON,
                                alt: "",
                                width: "36",
                                height: "36",
                            }
                            span { class: "truncate text-lg font-bold tracking-tight text-red-700", "China Travel" }
                        }
                        span { class: "text-xs font-medium uppercase tracking-[0.18em] text-slate-500", "Companion" }
                    }
                }
                main { class: "mx-auto flex w-full max-w-3xl flex-1 flex-col px-4 pb-8 pt-5",
                    Outlet::<Route> {}
                }
            }
        }
    }
}
