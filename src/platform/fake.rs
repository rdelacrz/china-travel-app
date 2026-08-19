use super::{PickDocumentOutcome, PlatformPort};
use crate::domain::AttachmentRef;
use crate::error::PlatformError;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

#[derive(Clone, Default)]
pub struct FakePlatform {
    state: Arc<Mutex<FakePlatformState>>,
}

#[derive(Debug, Default)]
struct FakePlatformState {
    next_pick: Option<PickDocumentOutcome>,
    opened_documents: Vec<String>,
    opened_urls: Vec<String>,
    released_uris: Vec<String>,
    open_error: Option<PlatformError>,
    created_documents: Vec<(String, String, Vec<u8>)>,
    text_documents: std::collections::HashMap<String, String>,
}

impl FakePlatform {
    pub fn with_pick_result(result: PickDocumentOutcome) -> Self {
        let platform = Self::default();
        platform.set_next_pick(result);
        platform
    }

    pub fn set_next_pick(&self, result: PickDocumentOutcome) {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .next_pick = Some(result);
    }

    pub fn set_open_error(&self, error: Option<PlatformError>) {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .open_error = error;
    }

    pub fn opened_documents(&self) -> Vec<String> {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .opened_documents
            .clone()
    }

    pub fn opened_urls(&self) -> Vec<String> {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .opened_urls
            .clone()
    }

    pub fn released_uris(&self) -> Vec<String> {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .released_uris
            .clone()
    }

    pub fn set_text_document(&self, uri: &str, content: &str) {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .text_documents
            .insert(uri.to_string(), content.to_string());
    }

    pub fn created_documents(&self) -> Vec<(String, String, Vec<u8>)> {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .created_documents
            .clone()
    }
}

#[async_trait(?Send)]
impl PlatformPort for FakePlatform {
    async fn app_data_directory(&self) -> Result<PathBuf, PlatformError> {
        Ok(std::env::var_os("CHINA_TRAVEL_WATCH_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(":memory:")))
    }

    async fn pick_document(
        &self,
        _prefer_downloads: bool,
    ) -> Result<PickDocumentOutcome, PlatformError> {
        Ok(self
            .state
            .lock()
            .expect("fake platform lock poisoned")
            .next_pick
            .take()
            .unwrap_or(PickDocumentOutcome::Cancelled))
    }

    async fn create_document(
        &self,
        file_name: &str,
        mime_type: &str,
        content: &[u8],
    ) -> Result<bool, PlatformError> {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .created_documents
            .push((
                file_name.to_string(),
                mime_type.to_string(),
                content.to_vec(),
            ));
        Ok(true)
    }

    async fn read_text_document(&self, uri: &str) -> Result<String, PlatformError> {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .text_documents
            .get(uri)
            .cloned()
            .ok_or(PlatformError::AttachmentUnavailable)
    }

    async fn open_document(&self, attachment: &AttachmentRef) -> Result<(), PlatformError> {
        let mut state = self.state.lock().expect("fake platform lock poisoned");
        if let Some(error) = state.open_error.clone() {
            return Err(error);
        }
        state.opened_documents.push(attachment.uri.clone());
        Ok(())
    }

    async fn open_url(&self, url: &str) -> Result<(), PlatformError> {
        let mut state = self.state.lock().expect("fake platform lock poisoned");
        if let Some(error) = state.open_error.clone() {
            return Err(error);
        }
        state.opened_urls.push(url.to_string());
        Ok(())
    }

    async fn release_read_permission(&self, uri: &str) -> Result<(), PlatformError> {
        self.state
            .lock()
            .expect("fake platform lock poisoned")
            .released_uris
            .push(uri.to_string());
        Ok(())
    }
}
