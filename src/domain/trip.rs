use crate::error::ValidationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trip {
    pub id: i64,
    pub name: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl Trip {
    pub fn validate_name(name: &str) -> Result<String, ValidationError> {
        let value = name.trim();
        if value.is_empty() {
            return Err(ValidationError::BlankTripName);
        }
        Ok(value.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TripOverview {
    pub trip: Trip,
    pub checklist_total: i64,
    pub checklist_completed: i64,
    pub document_count: i64,
}

impl TripOverview {
    pub fn checklist_outstanding(&self) -> i64 {
        self.checklist_total - self.checklist_completed
    }
}
