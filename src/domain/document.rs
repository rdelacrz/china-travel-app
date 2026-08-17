use crate::error::ValidationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttachmentRef {
    pub uri: String,
    pub display_name: Option<String>,
    pub mime_type: Option<String>,
}

impl AttachmentRef {
    pub fn new(
        uri: String,
        display_name: Option<String>,
        mime_type: Option<String>,
    ) -> Option<Self> {
        if uri.trim().is_empty() {
            None
        } else {
            Some(Self {
                uri,
                display_name,
                mime_type,
            })
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TravelDocument {
    pub id: i64,
    pub trip_id: i64,
    pub name: String,
    pub description: String,
    pub attachment: Option<AttachmentRef>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTravelDocument {
    pub trip_id: i64,
    pub name: String,
    pub description: String,
    pub attachment: Option<AttachmentRef>,
}

impl NewTravelDocument {
    pub fn new(
        trip_id: i64,
        name: &str,
        description: String,
        attachment: Option<AttachmentRef>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            trip_id,
            name: validate_name(name)?,
            description,
            attachment,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateTravelDocument {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub attachment: Option<AttachmentRef>,
}

impl UpdateTravelDocument {
    pub fn new(
        id: i64,
        name: &str,
        description: String,
        attachment: Option<AttachmentRef>,
    ) -> Result<Self, ValidationError> {
        Ok(Self {
            id,
            name: validate_name(name)?,
            description,
            attachment,
        })
    }
}

fn validate_name(name: &str) -> Result<String, ValidationError> {
    let value = name.trim();
    if value.is_empty() {
        return Err(ValidationError::BlankDocumentName);
    }
    Ok(value.to_string())
}
