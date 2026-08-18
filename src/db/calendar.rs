use super::Database;
use crate::domain::{CalendarEvent, NewCalendarEvent, UpdateCalendarEvent};
use crate::error::DbError;
use tokio_rusqlite::rusqlite::params;

impl Database {
    pub async fn list_calendar_events(&self, trip_id: i64) -> Result<Vec<CalendarEvent>, DbError> {
        self.call(move |connection| {
            let mut statement = connection.prepare(
                "SELECT id, trip_id, name, start_date, end_date, created_at, updated_at
                 FROM calendar_events
                 WHERE trip_id = ?1
                 ORDER BY start_date, end_date, id",
            )?;
            let rows = statement.query_map([trip_id], map_calendar_event)?;
            rows.collect()
        })
        .await
    }

    pub async fn create_calendar_event(
        &self,
        event: NewCalendarEvent,
    ) -> Result<CalendarEvent, DbError> {
        self.call(move |connection| {
            connection.execute(
                "INSERT INTO calendar_events (trip_id, name, start_date, end_date)
                 VALUES (?1, ?2, ?3, ?4)",
                params![event.trip_id, event.name, event.start_date, event.end_date],
            )?;
            connection.query_row(
                "SELECT id, trip_id, name, start_date, end_date, created_at, updated_at
                 FROM calendar_events WHERE id = last_insert_rowid()",
                [],
                map_calendar_event,
            )
        })
        .await
    }

    pub async fn delete_calendar_event(&self, event_id: i64) -> Result<(), DbError> {
        self.call(move |connection| {
            let changed =
                connection.execute("DELETE FROM calendar_events WHERE id = ?1", [event_id])?;
            if changed == 0 {
                return Err(tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows);
            }
            Ok(())
        })
        .await
        .map_err(|error| match error {
            DbError::Operation(tokio_rusqlite::Error::Error(
                tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows,
            )) => DbError::NotFound {
                entity: "calendar event",
                id: event_id,
            },
            other => other,
        })
    }

    pub async fn update_calendar_event(
        &self,
        event: UpdateCalendarEvent,
    ) -> Result<CalendarEvent, DbError> {
        let event_id = event.id;
        self.call(move |connection| {
            let changed = connection.execute(
                "UPDATE calendar_events
                 SET name = ?1, start_date = ?2, end_date = ?3, updated_at = unixepoch()
                 WHERE id = ?4 AND trip_id = ?5",
                params![
                    event.name,
                    event.start_date,
                    event.end_date,
                    event.id,
                    event.trip_id
                ],
            )?;
            if changed == 0 {
                return Err(tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows);
            }
            connection.query_row(
                "SELECT id, trip_id, name, start_date, end_date, created_at, updated_at
                 FROM calendar_events WHERE id = ?1",
                [event.id],
                map_calendar_event,
            )
        })
        .await
        .map_err(|error| match error {
            DbError::Operation(tokio_rusqlite::Error::Error(
                tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows,
            )) => DbError::NotFound {
                entity: "calendar event",
                id: event_id,
            },
            other => other,
        })
    }
}

fn map_calendar_event(
    row: &tokio_rusqlite::rusqlite::Row<'_>,
) -> tokio_rusqlite::rusqlite::Result<CalendarEvent> {
    Ok(CalendarEvent {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        name: row.get(2)?,
        start_date: row.get(3)?,
        end_date: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
