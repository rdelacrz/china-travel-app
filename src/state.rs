use crate::db::Database;
use crate::platform::PlatformPort;
use dioxus::prelude::*;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppServices {
    pub database: Database,
    pub platform: Arc<dyn PlatformPort>,
    pub safe_mode: bool,
}

impl PartialEq for AppServices {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

pub fn use_database() -> Database {
    use_context::<Database>()
}

pub fn use_platform() -> Arc<dyn PlatformPort> {
    use_context::<Arc<dyn PlatformPort>>()
}

pub fn use_revision() -> Signal<u64> {
    use_context::<Signal<u64>>()
}

pub fn use_safe_mode() -> Signal<bool> {
    use_context::<Signal<bool>>()
}
