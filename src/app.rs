use crate::components::app_shell::AppShell;
use crate::db::Database;
use crate::platform::{default_platform, PlatformPort};
use crate::state::AppServices;
use crate::views::{Checklist, Documentation, Home};
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Routable, Clone, PartialEq, Debug)]
pub enum Route {
    #[layout(AppShell)]
    #[route("/")]
    Home {},
    #[route("/trip/:trip_id/checklist")]
    Checklist { trip_id: i64 },
    #[route("/trip/:trip_id/documentation")]
    Documentation { trip_id: i64 },
}

#[component]
pub fn App() -> Element {
    let platform = use_hook(default_platform);
    let mut initialization = use_resource({
        let platform = platform.clone();
        move || {
            let platform = platform.clone();
            async move { initialize_services(platform).await }
        }
    });

    match &*initialization.value().read_unchecked() {
        None => rsx! { StartupState { state: "Preparing secure local storage…", busy: true } },
        Some(Err(error)) => rsx! {
            StartupState {
                state: "The local travel database could not be opened.",
                detail: error.to_string(),
                busy: false,
                retry: move |_| initialization.restart(),
            }
        },
        Some(Ok(services)) => rsx! { ReadyApp { services: services.clone() } },
    }
}

#[component]
fn ReadyApp(services: AppServices) -> Element {
    use_context_provider(|| services.database.clone());
    use_context_provider(|| services.platform.clone());
    let revision = use_signal(|| 0_u64);
    use_context_provider(|| revision);

    rsx! {
        AppStyles {}
        document::Meta { name: "viewport", content: "width=device-width, initial-scale=1, viewport-fit=cover" }
        Router::<Route> {}
    }
}

#[component]
fn StartupState(
    state: &'static str,
    #[props(default)] detail: String,
    #[props(default)] busy: bool,
    #[props(default)] retry: Option<EventHandler<MouseEvent>>,
) -> Element {
    rsx! {
        AppStyles {}
        div { class: "flex min-h-dvh items-center justify-center px-6 safe-top safe-bottom",
            section { class: "w-full max-w-md rounded-3xl border border-slate-200 bg-white p-6 shadow-sm",
                div { class: "mb-4 text-3xl", "🧧" }
                h1 { class: "text-xl font-bold text-slate-950", "China Travel Companion" }
                p { class: "mt-3 text-sm leading-6 text-slate-600", "{state}" }
                if !detail.is_empty() {
                    p { class: "mt-2 rounded-xl bg-red-50 p-3 text-xs leading-5 text-red-800", "{detail}" }
                }
                if busy {
                    p { class: "mt-5 text-sm font-medium text-slate-500", "Loading…" }
                } else if let Some(retry) = retry {
                    button {
                        class: "mt-5 min-h-12 w-full rounded-xl bg-red-700 px-4 text-sm font-semibold text-white transition hover:bg-red-800 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700",
                        onclick: move |event| retry.call(event),
                        "Try again"
                    }
                }
            }
        }
    }
}

#[cfg(target_os = "android")]
#[component]
fn AppStyles() -> Element {
    rsx! {}
}

#[cfg(not(target_os = "android"))]
#[component]
fn AppStyles() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        document::Stylesheet { href: asset!("/assets/dx-components-theme.css") }
    }
}

async fn initialize_services(
    platform: Arc<dyn PlatformPort>,
) -> Result<AppServices, crate::error::AppError> {
    let data_directory = platform.app_data_directory().await?;
    let database = if data_directory == std::path::Path::new(":memory:") {
        Database::open_in_memory().await?
    } else {
        tokio::fs::create_dir_all(&data_directory).await?;
        Database::open(data_directory.join("china-travel.sqlite3")).await?
    };
    Ok(AppServices { database, platform })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn routes_are_stable_and_mobile_readable() {
        assert_eq!(Route::Home {}.to_string(), "/");
        assert_eq!(
            Route::Checklist { trip_id: 42 }.to_string(),
            "/trip/42/checklist"
        );
        assert_eq!(
            Route::Documentation { trip_id: 42 }.to_string(),
            "/trip/42/documentation"
        );
    }
}
