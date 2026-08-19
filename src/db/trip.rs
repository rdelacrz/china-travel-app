use super::Database;
use crate::domain::{Trip, TripOverview};
use crate::error::DbError;
use tokio_rusqlite::rusqlite::params;

impl Database {
    pub async fn ensure_default_trip(&self) -> Result<Trip, DbError> {
        self.call(|connection| {
            connection.execute(
                "INSERT INTO trips (name)
                 SELECT 'China trip'
                 WHERE NOT EXISTS (SELECT 1 FROM trips)",
                [],
            )?;
            connection.query_row(
                "SELECT id, name, start_date, end_date, created_at, updated_at
                 FROM trips ORDER BY id LIMIT 1",
                [],
                map_trip,
            )
        })
        .await
    }

    pub async fn list_trip_overviews(&self) -> Result<Vec<TripOverview>, DbError> {
        self.call(|connection| {
            let mut statement = connection.prepare(
                "SELECT
                    t.id, t.name, t.start_date, t.end_date, t.created_at, t.updated_at,
                    (SELECT COUNT(*) FROM checklist_items i WHERE i.trip_id = t.id),
                    (SELECT COUNT(*) FROM checklist_items i WHERE i.trip_id = t.id AND i.is_checked = 1),
                    (SELECT COUNT(*) FROM travel_documents d WHERE d.trip_id = t.id)
                 FROM trips t
                 ORDER BY t.updated_at DESC, t.id DESC",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(TripOverview {
                    trip: map_trip(row)?,
                    checklist_total: row.get(6)?,
                    checklist_completed: row.get(7)?,
                    document_count: row.get(8)?,
                })
            })?;
            rows.collect()
        })
        .await
    }

    pub async fn get_trip(&self, trip_id: i64) -> Result<Trip, DbError> {
        self.call(move |connection| {
            connection.query_row(
                "SELECT id, name, start_date, end_date, created_at, updated_at
                 FROM trips WHERE id = ?1",
                [trip_id],
                map_trip,
            )
        })
        .await
        .map_err(|error| match error {
            DbError::Operation(tokio_rusqlite::Error::Error(
                tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows,
            )) => DbError::NotFound {
                entity: "trip",
                id: trip_id,
            },
            other => other,
        })
    }

    pub async fn create_trip(&self, name: &str) -> Result<Trip, DbError> {
        self.create_trip_with_dates(name, None, None).await
    }

    pub async fn create_trip_with_dates(
        &self,
        name: &str,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<Trip, DbError> {
        let name = crate::domain::Trip::validate_name(name).map_err(DbError::InvalidInput)?;
        let (start_date, end_date) =
            Trip::normalize_date_range(start_date, end_date).map_err(DbError::InvalidInput)?;
        self.call(move |connection| {
            connection.execute(
                "INSERT INTO trips (name, start_date, end_date) VALUES (?1, ?2, ?3)",
                params![name, start_date, end_date],
            )?;
            connection.query_row(
                "SELECT id, name, start_date, end_date, created_at, updated_at
                 FROM trips WHERE id = last_insert_rowid()",
                [],
                map_trip,
            )
        })
        .await
    }

    pub async fn delete_trip(&self, trip_id: i64) -> Result<(), DbError> {
        self.call(move |connection| {
            let changed = connection.execute(
                "DELETE FROM trips
                 WHERE id = ?1
                   AND NOT EXISTS (
                       SELECT 1 FROM app_settings WHERE key = 'safe_mode' AND value = 'true'
                   )",
                [trip_id],
            )?;
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
                entity: "trip",
                id: trip_id,
            },
            other => other,
        })
    }

    pub async fn update_trip_dates(
        &self,
        trip_id: i64,
        start_date: Option<&str>,
        end_date: Option<&str>,
    ) -> Result<Trip, DbError> {
        let (start_date, end_date) =
            Trip::normalize_date_range(start_date, end_date).map_err(DbError::InvalidInput)?;
        self.call(move |connection| {
            let changed = connection.execute(
                "UPDATE trips
                 SET start_date = ?1, end_date = ?2, updated_at = unixepoch()
                 WHERE id = ?3",
                params![start_date, end_date, trip_id],
            )?;
            if changed == 0 {
                return Err(tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows);
            }
            connection.query_row(
                "SELECT id, name, start_date, end_date, created_at, updated_at
                 FROM trips WHERE id = ?1",
                [trip_id],
                map_trip,
            )
        })
        .await
        .map_err(|error| match error {
            DbError::Operation(tokio_rusqlite::Error::Error(
                tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows,
            )) => DbError::NotFound {
                entity: "trip",
                id: trip_id,
            },
            other => other,
        })
    }
}

fn map_trip(row: &tokio_rusqlite::rusqlite::Row<'_>) -> tokio_rusqlite::rusqlite::Result<Trip> {
    Ok(Trip {
        id: row.get(0)?,
        name: row.get(1)?,
        start_date: row.get(2)?,
        end_date: row.get(3)?,
        created_at: row.get(4)?,
        updated_at: row.get(5)?,
    })
}
