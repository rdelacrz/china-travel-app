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

#[cfg(not(feature = "mobile"))]
fn main() {
    // The library target remains available for host database/protocol tests.
}
