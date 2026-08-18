#[cfg(feature = "mobile")]
use china_travel_app::App;
#[cfg(feature = "mobile")]
use dioxus::prelude::*;

#[cfg(feature = "mobile")]
fn main() {
    let custom_head = format!(
        r#"<link rel="stylesheet" href="{}"><link rel="stylesheet" href="{}">"#,
        asset!("/assets/tailwind.css"),
        asset!("/assets/dx-components-theme.css"),
    );
    let config = dioxus::mobile::Config::new()
        .with_custom_head(custom_head)
        .with_background_color((248, 250, 252, 255));
    dioxus::LaunchBuilder::new().with_cfg(config).launch(App);
}

// LiveView runs the native app on the host and streams its UI to the browser.
#[cfg(all(feature = "liveview", not(feature = "mobile")))]
fn main() {
    use axum::{response::Html, routing::get, Router};
    use dioxus_liveview::LiveviewRouter;
    use tower_http::services::ServeDir;

    let watch_data_dir = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/target/dx/china-travel-app/debug/liveview-data"
    );
    std::env::set_var("CHINA_TRAVEL_WATCH_DATA_DIR", watch_data_dir);

    const PUBLIC_ASSETS: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/target/dx/china-travel-app/debug/web/public/assets"
    );
    let router = Router::new()
        .route(
            "/",
            get(|| async {
                Html(format!(
                    "<!doctype html><html><head><title>China Travel App</title></head><body><div id=\"main\"></div>{}</body></html>",
                    dioxus_liveview::interpreter_glue("/ws")
                ))
            }),
        )
        .nest_service("/assets", ServeDir::new(PUBLIC_ASSETS))
        .with_app("/", china_travel_app::App);
    let address: std::net::SocketAddr = "10.8.0.1:8081"
        .parse()
        .expect("WireGuard LiveView address must be valid");
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("LiveView runtime must start");

    runtime.block_on(async move {
        let listener = tokio::net::TcpListener::bind(address)
            .await
            .expect("LiveView listener must bind to WireGuard");
        println!(
            "China Travel App LiveView started on http://{address} (SQLite: {watch_data_dir})"
        );
        axum::serve(listener, router)
            .await
            .expect("LiveView server must remain available");
    });
}

#[cfg(all(feature = "web", not(feature = "mobile"), not(feature = "liveview")))]
fn main() {
    dioxus::launch(china_travel_app::App);
}

#[cfg(all(
    not(feature = "mobile"),
    not(feature = "web"),
    not(feature = "liveview")
))]
fn main() {
    // The library target remains available for host database/protocol tests.
}
