use super::Database;
use crate::domain::ChecklistItem;
use crate::error::DbError;
use tokio_rusqlite::rusqlite::params;

impl Database {
    pub async fn list_checklist_items(&self, trip_id: i64) -> Result<Vec<ChecklistItem>, DbError> {
        self.call(move |connection| {
            let mut statement = connection.prepare(
                "SELECT id, trip_id, text, is_checked, sort_order, created_at, updated_at
                 FROM checklist_items
                 WHERE trip_id = ?1
                 ORDER BY sort_order, id",
            )?;
            let rows = statement.query_map([trip_id], map_checklist_item)?;
            rows.collect()
        })
        .await
    }

    pub async fn add_checklist_item(
        &self,
        trip_id: i64,
        text: &str,
    ) -> Result<ChecklistItem, DbError> {
        let text = ChecklistItem::validate_text(text).map_err(DbError::InvalidInput)?;
        self.call(move |connection| {
            let sort_order: i64 = connection.query_row(
                "SELECT COALESCE(MAX(sort_order) + 1, 0) FROM checklist_items WHERE trip_id = ?1",
                [trip_id],
                |row| row.get(0),
            )?;
            connection.execute(
                "INSERT INTO checklist_items (trip_id, text, sort_order)
                 VALUES (?1, ?2, ?3)",
                params![trip_id, text, sort_order],
            )?;
            connection.query_row(
                "SELECT id, trip_id, text, is_checked, sort_order, created_at, updated_at
                 FROM checklist_items WHERE id = last_insert_rowid()",
                [],
                map_checklist_item,
            )
        })
        .await
    }

    pub async fn rename_checklist_item(
        &self,
        item_id: i64,
        text: &str,
    ) -> Result<ChecklistItem, DbError> {
        let text = ChecklistItem::validate_text(text).map_err(DbError::InvalidInput)?;
        self.call(move |connection| {
            let changed = connection.execute(
                "UPDATE checklist_items
                 SET text = ?1, updated_at = unixepoch()
                 WHERE id = ?2",
                params![text, item_id],
            )?;
            if changed == 0 {
                return Err(tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows);
            }
            connection.query_row(
                "SELECT id, trip_id, text, is_checked, sort_order, created_at, updated_at
                 FROM checklist_items WHERE id = ?1",
                [item_id],
                map_checklist_item,
            )
        })
        .await
        .map_err(|error| match error {
            DbError::Operation(tokio_rusqlite::Error::Error(
                tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows,
            )) => DbError::NotFound {
                entity: "checklist item",
                id: item_id,
            },
            other => other,
        })
    }

    pub async fn set_checklist_checked(
        &self,
        item_id: i64,
        is_checked: bool,
    ) -> Result<ChecklistItem, DbError> {
        self.call(move |connection| {
            let changed = connection.execute(
                "UPDATE checklist_items
                 SET is_checked = ?1, updated_at = unixepoch()
                 WHERE id = ?2",
                params![is_checked as i64, item_id],
            )?;
            if changed == 0 {
                return Err(tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows);
            }
            connection.query_row(
                "SELECT id, trip_id, text, is_checked, sort_order, created_at, updated_at
                 FROM checklist_items WHERE id = ?1",
                [item_id],
                map_checklist_item,
            )
        })
        .await
        .map_err(|error| match error {
            DbError::Operation(tokio_rusqlite::Error::Error(
                tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows,
            )) => DbError::NotFound {
                entity: "checklist item",
                id: item_id,
            },
            other => other,
        })
    }

    pub async fn delete_checklist_item(&self, item_id: i64) -> Result<(), DbError> {
        self.call(move |connection| {
            let changed =
                connection.execute("DELETE FROM checklist_items WHERE id = ?1", [item_id])?;
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
                entity: "checklist item",
                id: item_id,
            },
            other => other,
        })
    }
}

fn map_checklist_item(
    row: &tokio_rusqlite::rusqlite::Row<'_>,
) -> tokio_rusqlite::rusqlite::Result<ChecklistItem> {
    Ok(ChecklistItem {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        text: row.get(2)?,
        is_checked: row.get::<_, i64>(3)? != 0,
        sort_order: row.get(4)?,
        created_at: row.get(5)?,
        updated_at: row.get(6)?,
    })
}
