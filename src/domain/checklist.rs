use crate::error::ValidationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: i64,
    pub trip_id: i64,
    pub text: String,
    pub is_checked: bool,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
}

impl ChecklistItem {
    pub fn validate_text(text: &str) -> Result<String, ValidationError> {
        let value = text.trim();
        if value.is_empty() {
            return Err(ValidationError::BlankChecklistText);
        }
        Ok(value.to_string())
    }
}
