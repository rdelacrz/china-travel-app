#[cfg(feature = "mobile")]
use china_travel_app::App;

#[cfg(feature = "mobile")]
fn main() {
    let config = dioxus::mobile::Config::new().with_background_color((248, 250, 252, 255));
    dioxus::LaunchBuilder::new().with_cfg(config).launch(App);
}

#[cfg(not(feature = "mobile"))]
fn main() {
    // The library target remains available for host database/protocol tests.
}
