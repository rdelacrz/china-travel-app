use super::Database;
use crate::domain::{AttachmentRef, NewTravelDocument, TravelDocument, UpdateTravelDocument};
use crate::error::DbError;
use tokio_rusqlite::rusqlite::params;

impl Database {
    pub async fn list_documents(&self, trip_id: i64) -> Result<Vec<TravelDocument>, DbError> {
        self.call(move |connection| {
            let mut statement = connection.prepare(
                "SELECT id, trip_id, name, description,
                        attachment_uri, attachment_display_name, attachment_mime_type,
                        created_at, updated_at
                 FROM travel_documents
                 WHERE trip_id = ?1
                 ORDER BY updated_at DESC, id DESC",
            )?;
            let rows = statement.query_map([trip_id], map_document)?;
            rows.collect()
        })
        .await
    }

    pub async fn get_document(&self, document_id: i64) -> Result<TravelDocument, DbError> {
        self.call(move |connection| {
            connection.query_row(
                "SELECT id, trip_id, name, description,
                        attachment_uri, attachment_display_name, attachment_mime_type,
                        created_at, updated_at
                 FROM travel_documents WHERE id = ?1",
                [document_id],
                map_document,
            )
        })
        .await
        .map_err(|error| match error {
            DbError::Operation(tokio_rusqlite::Error::Error(
                tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows,
            )) => DbError::NotFound {
                entity: "document",
                id: document_id,
            },
            other => other,
        })
    }

    pub async fn create_document(
        &self,
        document: NewTravelDocument,
    ) -> Result<TravelDocument, DbError> {
        self.call(move |connection| {
            let (uri, display_name, mime_type) = attachment_params(document.attachment.as_ref());
            connection.execute(
                "INSERT INTO travel_documents
                    (trip_id, name, description, attachment_uri,
                     attachment_display_name, attachment_mime_type)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    document.trip_id,
                    document.name,
                    document.description,
                    uri,
                    display_name,
                    mime_type
                ],
            )?;
            connection.query_row(
                "SELECT id, trip_id, name, description,
                        attachment_uri, attachment_display_name, attachment_mime_type,
                        created_at, updated_at
                 FROM travel_documents WHERE id = last_insert_rowid()",
                [],
                map_document,
            )
        })
        .await
    }

    pub async fn update_document(
        &self,
        document: UpdateTravelDocument,
    ) -> Result<TravelDocument, DbError> {
        let document_id = document.id;
        self.call(move |connection| {
            let (uri, display_name, mime_type) = attachment_params(document.attachment.as_ref());
            let changed = connection.execute(
                "UPDATE travel_documents
                 SET name = ?1, description = ?2, attachment_uri = ?3,
                     attachment_display_name = ?4, attachment_mime_type = ?5,
                     updated_at = unixepoch()
                 WHERE id = ?6",
                params![
                    document.name,
                    document.description,
                    uri,
                    display_name,
                    mime_type,
                    document.id
                ],
            )?;
            if changed == 0 {
                return Err(tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows);
            }
            connection.query_row(
                "SELECT id, trip_id, name, description,
                        attachment_uri, attachment_display_name, attachment_mime_type,
                        created_at, updated_at
                 FROM travel_documents WHERE id = ?1",
                [document.id],
                map_document,
            )
        })
        .await
        .map_err(|error| match error {
            DbError::Operation(tokio_rusqlite::Error::Error(
                tokio_rusqlite::rusqlite::Error::QueryReturnedNoRows,
            )) => DbError::NotFound {
                entity: "document",
                id: document_id,
            },
            other => other,
        })
    }
}

fn attachment_params(
    attachment: Option<&AttachmentRef>,
) -> (Option<&str>, Option<&str>, Option<&str>) {
    attachment
        .map(|item| {
            (
                Some(item.uri.as_str()),
                item.display_name.as_deref(),
                item.mime_type.as_deref(),
            )
        })
        .unwrap_or((None, None, None))
}

fn map_document(
    row: &tokio_rusqlite::rusqlite::Row<'_>,
) -> tokio_rusqlite::rusqlite::Result<TravelDocument> {
    let uri: Option<String> = row.get(4)?;
    let display_name: Option<String> = row.get(5)?;
    let mime_type: Option<String> = row.get(6)?;
    Ok(TravelDocument {
        id: row.get(0)?,
        trip_id: row.get(1)?,
        name: row.get(2)?,
        description: row.get(3)?,
        attachment: uri.map(|uri| AttachmentRef {
            uri,
            display_name,
            mime_type,
        }),
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}
