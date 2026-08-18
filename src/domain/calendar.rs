use crate::error::ValidationError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

const MONTH_NAMES: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct CalendarDate {
    pub year: i32,
    pub month: u8,
    pub day: u8,
}

impl CalendarDate {
    pub fn parse(value: &str) -> Result<Self, ValidationError> {
        if value.len() != 10
            || value.as_bytes().get(4) != Some(&b'-')
            || value.as_bytes().get(7) != Some(&b'-')
        {
            return Err(ValidationError::InvalidCalendarDate);
        }
        let mut parts = value.split('-');
        let year_part = parts.next();
        let month_part = parts.next();
        let day_part = parts.next();
        if parts.next().is_some()
            || year_part.map(str::len) != Some(4)
            || month_part.map(str::len) != Some(2)
            || day_part.map(str::len) != Some(2)
        {
            return Err(ValidationError::InvalidCalendarDate);
        }
        let year = year_part
            .and_then(|part| part.parse::<i32>().ok())
            .filter(|year| (1..=9999).contains(year));
        let month = month_part
            .and_then(|part| part.parse::<u8>().ok())
            .filter(|month| (1..=12).contains(month));
        let day = day_part.and_then(|part| part.parse::<u8>().ok());
        let (Some(year), Some(month), Some(day)) = (year, month, day) else {
            return Err(ValidationError::InvalidCalendarDate);
        };
        if day == 0 || day > Self::days_in_month(year, month) {
            return Err(ValidationError::InvalidCalendarDate);
        }
        Ok(Self { year, month, day })
    }

    pub fn today() -> Self {
        let days = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| (duration.as_secs() / 86_400) as i64)
            .unwrap_or_default();
        Self::from_days_since_unix_epoch(days)
    }

    pub fn days_in_month(year: i32, month: u8) -> u8 {
        match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if Self::is_leap_year(year) => 29,
            2 => 28,
            _ => 0,
        }
    }

    pub fn is_leap_year(year: i32) -> bool {
        year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
    }

    pub fn month_name(self) -> &'static str {
        MONTH_NAMES[(self.month - 1) as usize]
    }

    pub fn short_label(self) -> String {
        format!("{} {}", self.month_name(), self.day)
    }

    pub fn days_since_unix_epoch(self) -> i64 {
        let mut year = self.year as i64;
        let month = self.month as i64;
        let day = self.day as i64;
        year -= i64::from(month <= 2);
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400;
        let shifted_month = month + if month > 2 { -3 } else { 9 };
        let day_of_year = (153 * shifted_month + 2) / 5 + day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    fn from_days_since_unix_epoch(days: i64) -> Self {
        let shifted = days + 719_468;
        let era = shifted.div_euclid(146_097);
        let day_of_era = shifted - era * 146_097;
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let mut year = year_of_era + era * 400;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let shifted_month = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * shifted_month + 2) / 5 + 1;
        let month = shifted_month + if shifted_month < 10 { 3 } else { -9 };
        year += i64::from(month <= 2);
        Self {
            year: year as i32,
            month: month as u8,
            day: day as u8,
        }
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{:04}-{:02}-{:02}",
            self.year, self.month, self.day
        )
    }
}

pub fn normalize_optional_date(value: Option<&str>) -> Result<Option<String>, ValidationError> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(None);
    };
    Ok(Some(CalendarDate::parse(value)?.to_string()))
}

pub fn normalize_date_range(
    start_date: Option<&str>,
    end_date: Option<&str>,
) -> Result<(Option<String>, Option<String>), ValidationError> {
    let start_date = normalize_optional_date(start_date)?;
    let end_date = normalize_optional_date(end_date)?;
    if let (Some(start), Some(end)) = (&start_date, &end_date) {
        if CalendarDate::parse(end)? < CalendarDate::parse(start)? {
            return Err(ValidationError::CalendarEndBeforeStart);
        }
    }
    Ok((start_date, end_date))
}

pub fn date_range_label(start_date: Option<&str>, end_date: Option<&str>) -> Option<String> {
    let start = start_date.and_then(|value| CalendarDate::parse(value).ok());
    let end = end_date.and_then(|value| CalendarDate::parse(value).ok());
    match (start, end) {
        (Some(start), Some(end)) if start == end => {
            Some(format!("{} {}", start.short_label(), start.year))
        }
        (Some(start), Some(end)) => Some(format!(
            "{} {} – {} {}",
            start.short_label(),
            start.year,
            end.short_label(),
            end.year
        )),
        (Some(start), None) => Some(format!("Starts {} {}", start.short_label(), start.year)),
        (None, Some(end)) => Some(format!("Ends {} {}", end.short_label(), end.year)),
        (None, None) => None,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: i64,
    pub trip_id: i64,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewCalendarEvent {
    pub trip_id: i64,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
}

impl NewCalendarEvent {
    pub fn new(
        trip_id: i64,
        name: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Self, ValidationError> {
        let name = name.trim();
        if name.is_empty() {
            return Err(ValidationError::BlankCalendarEventName);
        }
        let start_date = CalendarDate::parse(start_date.trim())?;
        let end_date = CalendarDate::parse(end_date.trim())?;
        if end_date < start_date {
            return Err(ValidationError::CalendarEndBeforeStart);
        }
        Ok(Self {
            trip_id,
            name: name.to_string(),
            start_date: start_date.to_string(),
            end_date: end_date.to_string(),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateCalendarEvent {
    pub id: i64,
    pub trip_id: i64,
    pub name: String,
    pub start_date: String,
    pub end_date: String,
}

impl UpdateCalendarEvent {
    pub fn new(
        id: i64,
        trip_id: i64,
        name: &str,
        start_date: &str,
        end_date: &str,
    ) -> Result<Self, ValidationError> {
        let validated = NewCalendarEvent::new(trip_id, name, start_date, end_date)?;
        Ok(Self {
            id,
            trip_id: validated.trip_id,
            name: validated.name,
            start_date: validated.start_date,
            end_date: validated.end_date,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_are_validated_and_normalized() {
        assert_eq!(
            CalendarDate::parse("2028-02-29").unwrap().to_string(),
            "2028-02-29"
        );
        assert_eq!(
            CalendarDate::parse("2027-02-29"),
            Err(ValidationError::InvalidCalendarDate)
        );
        assert_eq!(
            CalendarDate::parse("2027-2-09"),
            Err(ValidationError::InvalidCalendarDate)
        );
        assert_eq!(
            normalize_date_range(Some(" 2027-04-02 "), Some("2027-04-06")).unwrap(),
            (
                Some("2027-04-02".to_string()),
                Some("2027-04-06".to_string())
            )
        );
        assert_eq!(
            normalize_date_range(Some("2027-04-06"), Some("2027-04-02")),
            Err(ValidationError::CalendarEndBeforeStart)
        );
    }

    #[test]
    fn calendar_date_round_trips_through_unix_days() {
        let date = CalendarDate::parse("2027-10-01").unwrap();
        assert_eq!(
            CalendarDate::from_days_since_unix_epoch(date.days_since_unix_epoch()),
            date
        );
    }

    #[test]
    fn new_event_requires_a_name_and_ordered_dates() {
        assert!(NewCalendarEvent::new(1, "Flight to Beijing", "2027-04-02", "2027-04-02").is_ok());
        assert_eq!(
            NewCalendarEvent::new(1, " ", "2027-04-02", "2027-04-02"),
            Err(ValidationError::BlankCalendarEventName)
        );
    }
}
