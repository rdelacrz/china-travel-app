use crate::domain::{date_range_label, normalize_date_range};
use crate::error::ValidationError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trip {
    pub id: i64,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
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

    pub fn normalize_date_range(
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<(Option<String>, Option<String>), ValidationError> {
        normalize_date_range(start_date, end_date)
    }

    pub fn date_range_label(&self) -> Option<String> {
        date_range_label(self.start_date.as_deref(), self.end_date.as_deref())
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
