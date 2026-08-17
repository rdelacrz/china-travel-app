use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::{Deserialize, Serialize};

pub const PROTOCOL_VERSION: u8 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeRequest {
    pub version: u8,
    pub request_id: String,
    pub operation: NativeOperation,
}

impl NativeRequest {
    pub fn new(request_id: String, operation: NativeOperation) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            request_id,
            operation,
        }
    }

    pub fn validate(&self) -> Result<(), ProtocolError> {
        if self.version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(self.version));
        }
        if self.request_id.trim().is_empty() {
            return Err(ProtocolError::MissingRequestId);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NativeOperation {
    AppDataDirectory,
    PickDocument {
        prefer_downloads: bool,
    },
    OpenDocument {
        uri: String,
        mime_type: Option<String>,
    },
    OpenUrl {
        url: String,
    },
    ReleaseReadPermission {
        uri: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NativeResponse {
    pub version: u8,
    pub request_id: String,
    pub result: NativeResult,
}

impl NativeResponse {
    pub fn success(request_id: String, result: NativeResult) -> Self {
        Self {
            version: PROTOCOL_VERSION,
            request_id,
            result,
        }
    }

    pub fn validate_for(&self, request: &NativeRequest) -> Result<(), ProtocolError> {
        if self.version != PROTOCOL_VERSION {
            return Err(ProtocolError::UnsupportedVersion(self.version));
        }
        if self.request_id != request.request_id {
            return Err(ProtocolError::RequestIdMismatch {
                expected: request.request_id.clone(),
                actual: self.request_id.clone(),
            });
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NativeResult {
    AppDataDirectory {
        path: String,
    },
    DocumentSelected {
        uri: String,
        display_name: Option<String>,
        mime_type: Option<String>,
    },
    Cancelled,
    Completed,
    Error {
        code: String,
        message: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ProtocolError {
    #[error("unsupported native bridge protocol version {0}")]
    UnsupportedVersion(u8),
    #[error("native bridge request ID cannot be blank")]
    MissingRequestId,
    #[error("native bridge request ID mismatch: expected {expected}, received {actual}")]
    RequestIdMismatch { expected: String, actual: String },
    #[error("invalid native bridge JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid native bridge base64 payload: {0}")]
    Base64(#[from] base64::DecodeError),
}

pub fn encode_request_base64(request: &NativeRequest) -> Result<String, ProtocolError> {
    request.validate()?;
    Ok(STANDARD.encode(serde_json::to_vec(request)?))
}

pub fn decode_request_base64(payload: &str) -> Result<NativeRequest, ProtocolError> {
    let bytes = STANDARD.decode(payload)?;
    let request: NativeRequest = serde_json::from_slice(&bytes)?;
    request.validate()?;
    Ok(request)
}

pub fn encode_response_base64(response: &NativeResponse) -> Result<String, ProtocolError> {
    if response.version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(response.version));
    }
    Ok(STANDARD.encode(serde_json::to_vec(response)?))
}

pub fn decode_response_base64(payload: &str) -> Result<NativeResponse, ProtocolError> {
    let bytes = STANDARD.decode(payload)?;
    let response: NativeResponse = serde_json::from_slice(&bytes)?;
    if response.version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(response.version));
    }
    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_operation_round_trips_through_base64_json() {
        let operations = [
            NativeOperation::AppDataDirectory,
            NativeOperation::PickDocument {
                prefer_downloads: true,
            },
            NativeOperation::OpenDocument {
                uri: "content://provider/秘密.pdf".to_string(),
                mime_type: Some("application/pdf".to_string()),
            },
            NativeOperation::OpenUrl {
                url: "https://example.test/旅程?q=1".to_string(),
            },
            NativeOperation::ReleaseReadPermission {
                uri: "content://provider/file".to_string(),
            },
        ];

        for (index, operation) in operations.into_iter().enumerate() {
            let request = NativeRequest::new(format!("request-{index}"), operation);
            let encoded = encode_request_base64(&request).unwrap();
            let decoded = decode_request_base64(&encoded).unwrap();
            assert_eq!(decoded, request);
        }
    }

    #[test]
    fn response_round_trip_preserves_cancelled_and_failure_results() {
        let responses = [
            NativeResponse::success("cancel".to_string(), NativeResult::Cancelled),
            NativeResponse::success(
                "failure".to_string(),
                NativeResult::Error {
                    code: "no_handler".to_string(),
                    message: "No compatible app".to_string(),
                },
            ),
        ];
        for response in responses {
            let encoded = encode_response_base64(&response).unwrap();
            assert_eq!(decode_response_base64(&encoded).unwrap(), response);
        }
    }

    #[test]
    fn malformed_payloads_and_mismatched_ids_are_rejected() {
        assert!(matches!(
            decode_request_base64("not-base64"),
            Err(ProtocolError::Base64(_))
        ));
        let request = NativeRequest::new("expected".to_string(), NativeOperation::AppDataDirectory);
        let response = NativeResponse::success("actual".to_string(), NativeResult::Completed);
        assert!(matches!(
            response.validate_for(&request),
            Err(ProtocolError::RequestIdMismatch { .. })
        ));
        assert!(matches!(
            NativeRequest {
                version: 9,
                ..request
            }
            .validate(),
            Err(ProtocolError::UnsupportedVersion(9))
        ));
    }
}
