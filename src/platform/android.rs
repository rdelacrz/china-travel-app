use super::protocol::{
    encode_request_base64, NativeOperation, NativeRequest, NativeResponse, NativeResult,
};
use super::{PickDocumentOutcome, PlatformPort};
use crate::domain::AttachmentRef;
use crate::error::PlatformError;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use dioxus::prelude::document;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use url::Url;

static NEXT_REQUEST_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Default)]
pub struct AndroidPlatform;

#[async_trait(?Send)]
impl PlatformPort for AndroidPlatform {
    async fn app_data_directory(&self) -> Result<PathBuf, PlatformError> {
        match self.send(NativeOperation::AppDataDirectory).await? {
            NativeResult::AppDataDirectory { path } if !path.trim().is_empty() => {
                Ok(PathBuf::from(path))
            }
            NativeResult::AppDataDirectory { .. } => Err(PlatformError::Protocol(
                "Android returned an empty app data directory".to_string(),
            )),
            other => Err(result_error(other)),
        }
    }

    async fn pick_document(
        &self,
        prefer_downloads: bool,
    ) -> Result<PickDocumentOutcome, PlatformError> {
        match self
            .send(NativeOperation::PickDocument { prefer_downloads })
            .await?
        {
            NativeResult::DocumentSelected {
                uri,
                display_name,
                mime_type,
            } if !uri.trim().is_empty() => Ok(PickDocumentOutcome::Selected(AttachmentRef {
                uri,
                display_name,
                mime_type,
            })),
            NativeResult::Cancelled => Ok(PickDocumentOutcome::Cancelled),
            NativeResult::DocumentSelected { .. } => Err(PlatformError::AttachmentUnavailable),
            other => Err(result_error(other)),
        }
    }

    async fn create_document(
        &self,
        file_name: &str,
        mime_type: &str,
        content: &[u8],
    ) -> Result<bool, PlatformError> {
        match self
            .send(NativeOperation::CreateDocument {
                file_name: file_name.to_string(),
                mime_type: mime_type.to_string(),
                content_base64: STANDARD.encode(content),
            })
            .await?
        {
            NativeResult::Completed => Ok(true),
            NativeResult::Cancelled => Ok(false),
            other => Err(result_error(other)),
        }
    }

    async fn read_text_document(&self, uri: &str) -> Result<String, PlatformError> {
        match self
            .send(NativeOperation::ReadTextDocument {
                uri: uri.to_string(),
            })
            .await?
        {
            NativeResult::TextDocument { content } => Ok(content),
            other => Err(result_error(other)),
        }
    }

    async fn open_document(&self, attachment: &AttachmentRef) -> Result<(), PlatformError> {
        match self
            .send(NativeOperation::OpenDocument {
                uri: attachment.uri.clone(),
                mime_type: attachment.mime_type.clone(),
            })
            .await?
        {
            NativeResult::Completed => Ok(()),
            other => Err(result_error(other)),
        }
    }

    async fn open_url(&self, url: &str) -> Result<(), PlatformError> {
        let parsed = Url::parse(url).map_err(|_| PlatformError::UnsupportedUrlScheme)?;
        if !matches!(parsed.scheme(), "http" | "https") {
            return Err(PlatformError::UnsupportedUrlScheme);
        }
        match self
            .send(NativeOperation::OpenUrl {
                url: parsed.to_string(),
            })
            .await?
        {
            NativeResult::Completed => Ok(()),
            other => Err(result_error(other)),
        }
    }

    async fn release_read_permission(&self, uri: &str) -> Result<(), PlatformError> {
        if uri.trim().is_empty() {
            return Ok(());
        }
        match self
            .send(NativeOperation::ReleaseReadPermission {
                uri: uri.to_string(),
            })
            .await?
        {
            NativeResult::Completed => Ok(()),
            other => Err(result_error(other)),
        }
    }
}

impl AndroidPlatform {
    async fn send(&self, operation: NativeOperation) -> Result<NativeResult, PlatformError> {
        let request = NativeRequest::new(
            format!(
                "android-{}",
                NEXT_REQUEST_ID.fetch_add(1, Ordering::Relaxed)
            ),
            operation,
        );
        let payload = encode_request_base64(&request)
            .map_err(|error| PlatformError::Protocol(error.to_string()))?;
        let script = format!(
            r#"return (async () => {{
                const decode = (value) => {{
                    const binary = atob(value);
                    const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
                    return JSON.parse(new TextDecoder().decode(bytes));
                }};
                window.__chinaTravelNativePending = window.__chinaTravelNativePending || {{}};
                window.__chinaTravelNativeResolveBase64 = window.__chinaTravelNativeResolveBase64 || ((encoded) => {{
                    const response = decode(encoded);
                    const resolver = window.__chinaTravelNativePending[response.request_id];
                    if (resolver) {{
                        delete window.__chinaTravelNativePending[response.request_id];
                        resolver(response);
                    }}
                }});
                const request = decode('{payload}');
                const failure = (code, message) => ({{
                    version: 1,
                    request_id: request.request_id,
                    result: {{ kind: 'error', code, message }}
                }});
                const response = await new Promise((resolve) => {{
                    const timeout = setTimeout(() => {{
                        delete window.__chinaTravelNativePending[request.request_id];
                        resolve(failure('timeout', 'Native operation timed out'));
                    }}, 30000);
                    window.__chinaTravelNativePending[request.request_id] = (value) => {{
                        clearTimeout(timeout);
                        resolve(value);
                    }};
                    if (!window.ChinaTravelBridge) {{
                        clearTimeout(timeout);
                        delete window.__chinaTravelNativePending[request.request_id];
                        resolve(failure('bridge_unavailable', 'Android bridge is unavailable'));
                    }} else {{
                        window.ChinaTravelBridge.postMessageBase64('{payload}');
                    }}
                }});
                return response;
            }})()"#
        );
        let value = document::eval(&script).await?;
        let response: NativeResponse = serde_json::from_value(value)
            .map_err(|error| PlatformError::Protocol(error.to_string()))?;
        response
            .validate_for(&request)
            .map_err(|error| PlatformError::Protocol(error.to_string()))?;
        Ok(response.result)
    }
}

fn result_error(result: NativeResult) -> PlatformError {
    match result {
        NativeResult::Error { code, message } => match code.as_str() {
            "permission_denied" => PlatformError::PersistablePermissionDenied,
            "attachment_unavailable" => PlatformError::AttachmentUnavailable,
            "no_handler" => PlatformError::NoActivityHandler,
            _ => PlatformError::Native(format!("{code}: {message}")),
        },
        NativeResult::Cancelled => PlatformError::Native("operation was cancelled".to_string()),
        other => PlatformError::Protocol(format!("unexpected native response: {other:?}")),
    }
}
