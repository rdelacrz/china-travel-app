use super::Database;
use crate::domain::{CalendarEvent, ChecklistItem, TravelDocument, Trip};
use crate::error::DbError;
use serde::{Deserialize, Serialize};
use tokio_rusqlite::rusqlite;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppBackup {
    pub version: u32,
    pub trips: Vec<FullTripBackup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FullTripBackup {
    pub trip: Trip,
    pub checklist_items: Vec<ChecklistItem>,
    pub documents: Vec<TravelDocument>,
    pub calendar_events: Vec<CalendarEvent>,
}

impl Database {
    pub async fn export_full_backup(&self) -> Result<AppBackup, DbError> {
        let trip_overviews = self.list_trip_overviews().await?;
        let mut trips = Vec::with_capacity(trip_overviews.len());
        for overview in trip_overviews {
            let trip_id = overview.trip.id;
            let checklist_items = self.list_checklist_items(trip_id).await?;
            let documents = self.list_documents(trip_id).await?;
            let calendar_events = self.list_calendar_events(trip_id).await?;
            trips.push(FullTripBackup {
                trip: overview.trip,
                checklist_items,
                documents,
                calendar_events,
            });
        }
        Ok(AppBackup { version: 1, trips })
    }

    pub async fn import_full_backup(&self, backup: &AppBackup) -> Result<(), DbError> {
        if backup.version != 1 {
            return Err(DbError::Migration(format!(
                "unsupported backup version {}",
                backup.version
            )));
        }
        let backup = backup.clone();
        self.call(move |connection| {
            connection.execute_batch("BEGIN IMMEDIATE;")?;
            let result = (|| {
                connection.execute_batch("DELETE FROM calendar_events;")?;
                connection.execute_batch("DELETE FROM travel_documents;")?;
                connection.execute_batch("DELETE FROM checklist_items;")?;
                connection.execute_batch("DELETE FROM trips;")?;

                for t in &backup.trips {
                    connection.execute(
                        "INSERT INTO trips (id, name, start_date, end_date, created_at, updated_at)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                        rusqlite::params![
                            t.trip.id,
                            t.trip.name,
                            t.trip.start_date,
                            t.trip.end_date,
                            t.trip.created_at,
                            t.trip.updated_at,
                        ],
                    )?;
                    for item in &t.checklist_items {
                        connection.execute(
                            "INSERT INTO checklist_items
                                (id, trip_id, text, is_checked, sort_order, created_at, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            rusqlite::params![
                                item.id,
                                item.trip_id,
                                item.text,
                                item.is_checked as i64,
                                item.sort_order,
                                item.created_at,
                                item.updated_at,
                            ],
                        )?;
                    }
                    for doc in &t.documents {
                        let (uri, display_name, mime_type) = doc
                            .attachment
                            .as_ref()
                            .map(|a| {
                                (
                                    Some(a.uri.as_str()),
                                    a.display_name.as_deref(),
                                    a.mime_type.as_deref(),
                                )
                            })
                            .unwrap_or((None, None, None));
                        connection.execute(
                            "INSERT INTO travel_documents
                                (id, trip_id, name, description,
                                 attachment_uri, attachment_display_name, attachment_mime_type,
                                 created_at, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                            rusqlite::params![
                                doc.id,
                                doc.trip_id,
                                doc.name,
                                doc.description,
                                uri,
                                display_name,
                                mime_type,
                                doc.created_at,
                                doc.updated_at,
                            ],
                        )?;
                    }
                    for event in &t.calendar_events {
                        connection.execute(
                            "INSERT INTO calendar_events
                                (id, trip_id, name, start_date, end_date, created_at, updated_at)
                             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                            rusqlite::params![
                                event.id,
                                event.trip_id,
                                event.name,
                                event.start_date,
                                event.end_date,
                                event.created_at,
                                event.updated_at,
                            ],
                        )?;
                    }
                }
                Ok(())
            })();
            if result.is_err() {
                let _ = connection.execute_batch("ROLLBACK;");
            } else {
                connection.execute_batch("COMMIT;")?;
            }
            result
        })
        .await
    }
}
