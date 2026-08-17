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
                "SELECT id, name, created_at, updated_at
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
                    t.id, t.name, t.created_at, t.updated_at,
                    (SELECT COUNT(*) FROM checklist_items i WHERE i.trip_id = t.id),
                    (SELECT COUNT(*) FROM checklist_items i WHERE i.trip_id = t.id AND i.is_checked = 1),
                    (SELECT COUNT(*) FROM travel_documents d WHERE d.trip_id = t.id)
                 FROM trips t
                 ORDER BY t.updated_at DESC, t.id DESC",
            )?;
            let rows = statement.query_map([], |row| {
                Ok(TripOverview {
                    trip: map_trip(row)?,
                    checklist_total: row.get(4)?,
                    checklist_completed: row.get(5)?,
                    document_count: row.get(6)?,
                })
            })?;
            rows.collect()
        })
        .await
    }

    pub async fn get_trip(&self, trip_id: i64) -> Result<Trip, DbError> {
        self.call(move |connection| {
            connection.query_row(
                "SELECT id, name, created_at, updated_at FROM trips WHERE id = ?1",
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
        let name = crate::domain::Trip::validate_name(name).map_err(DbError::InvalidInput)?;
        self.call(move |connection| {
            connection.execute("INSERT INTO trips (name) VALUES (?1)", params![name])?;
            connection.query_row(
                "SELECT id, name, created_at, updated_at FROM trips WHERE id = last_insert_rowid()",
                [],
                map_trip,
            )
        })
        .await
    }
}

fn map_trip(row: &tokio_rusqlite::rusqlite::Row<'_>) -> tokio_rusqlite::rusqlite::Result<Trip> {
    Ok(Trip {
        id: row.get(0)?,
        name: row.get(1)?,
        created_at: row.get(2)?,
        updated_at: row.get(3)?,
    })
}
