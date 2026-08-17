use crate::domain::AttachmentRef;
use crate::error::PlatformError;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;

#[cfg(target_os = "android")]
mod android;
mod fake;
pub mod protocol;

#[cfg(target_os = "android")]
pub use android::AndroidPlatform;
pub use fake::FakePlatform;
pub use protocol::{NativeOperation, NativeRequest, NativeResponse, NativeResult, ProtocolError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PickDocumentOutcome {
    Selected(AttachmentRef),
    Cancelled,
}

#[async_trait(?Send)]
pub trait PlatformPort: Send + Sync {
    async fn app_data_directory(&self) -> Result<PathBuf, PlatformError>;
    async fn pick_document(
        &self,
        prefer_downloads: bool,
    ) -> Result<PickDocumentOutcome, PlatformError>;
    async fn open_document(&self, attachment: &AttachmentRef) -> Result<(), PlatformError>;
    async fn open_url(&self, url: &str) -> Result<(), PlatformError>;
    async fn release_read_permission(&self, uri: &str) -> Result<(), PlatformError>;
}

pub fn default_platform() -> Arc<dyn PlatformPort> {
    #[cfg(target_os = "android")]
    {
        Arc::new(AndroidPlatform::default())
    }

    #[cfg(not(target_os = "android"))]
    {
        Arc::new(FakePlatform::default())
    }
}
