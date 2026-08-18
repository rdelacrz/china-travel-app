mod calendar;
mod checklist;
mod document;
mod trip;

pub use calendar::{
    date_range_label, normalize_date_range, normalize_optional_date, CalendarDate, CalendarEvent,
    NewCalendarEvent, UpdateCalendarEvent,
};
pub use checklist::ChecklistItem;
pub use document::{AttachmentRef, NewTravelDocument, TravelDocument, UpdateTravelDocument};
pub use trip::{Trip, TripOverview};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ValidationError;

    #[test]
    fn validation_trims_and_rejects_blank_values() {
        assert_eq!(Trip::validate_name("  Shanghai  ").unwrap(), "Shanghai");
        assert_eq!(
            ChecklistItem::validate_text("  Passport  ").unwrap(),
            "Passport"
        );
        assert_eq!(
            Trip::validate_name("  "),
            Err(ValidationError::BlankTripName)
        );
        assert_eq!(
            ChecklistItem::validate_text("\n"),
            Err(ValidationError::BlankChecklistText)
        );
        assert!(NewTravelDocument::new(1, "  Visa  ", String::new(), None).is_ok());
        assert!(NewTravelDocument::new(1, "", String::new(), None).is_err());
    }

    #[test]
    fn attachment_with_blank_uri_is_omitted() {
        assert!(AttachmentRef::new("  ".to_string(), None, None).is_none());
        assert!(AttachmentRef::new("content://example/file".to_string(), None, None).is_some());
    }
}
