use crate::error::DbError;
use std::path::{Path, PathBuf};
use tokio_rusqlite::{Connection, Error as TokioSqliteError};

pub mod backup;
mod calendar;
mod checklist;
mod document;
mod settings;
mod trip;

pub use backup::AppBackup;

const CURRENT_SCHEMA_VERSION: i64 = 3;
const MIGRATIONS: &[(i64, &str)] = &[
    (1, include_str!("../../migrations/0001_initial.sql")),
    (2, include_str!("../../migrations/0002_trip_calendar.sql")),
    (3, include_str!("../../migrations/0003_settings.sql")),
];

#[derive(Clone, Debug)]
pub struct Database {
    connection: Connection,
    path: Option<PathBuf>,
}

impl Database {
    pub async fn open(path: impl AsRef<Path>) -> Result<Self, DbError> {
        let path = path.as_ref().to_owned();
        let connection = Connection::open(&path).await.map_err(DbError::Open)?;
        let database = Self {
            connection,
            path: Some(path),
        };
        database.configure_and_migrate().await?;
        Ok(database)
    }

    pub async fn open_in_memory() -> Result<Self, DbError> {
        let connection = Connection::open_in_memory().await.map_err(DbError::Open)?;
        let database = Self {
            connection,
            path: None,
        };
        database.configure_and_migrate().await?;
        Ok(database)
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    async fn configure_and_migrate(&self) -> Result<(), DbError> {
        self.call(|connection| {
            connection.execute_batch(
                "PRAGMA foreign_keys = ON;
                 PRAGMA busy_timeout = 5000;
                 PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;",
            )
        })
        .await?;

        let version: i64 = self
            .call(|connection| connection.query_row("PRAGMA user_version", [], |row| row.get(0)))
            .await?;

        if version > CURRENT_SCHEMA_VERSION {
            return Err(DbError::Migration(format!(
                "database schema version {version} is newer than supported version {CURRENT_SCHEMA_VERSION}"
            )));
        }

        if version < CURRENT_SCHEMA_VERSION {
            self.call(move |connection| {
                let result = (|| {
                    connection.execute_batch("BEGIN IMMEDIATE;")?;
                    for (target_version, migration) in MIGRATIONS {
                        if *target_version > version {
                            connection.execute_batch(migration)?;
                            connection.execute_batch(&format!(
                                "PRAGMA user_version = {target_version};"
                            ))?;
                        }
                    }
                    connection.execute_batch("COMMIT;")
                })();

                if result.is_err() {
                    let _ = connection.execute_batch("ROLLBACK;");
                }
                result
            })
            .await?;
        }

        Ok(())
    }

    pub(crate) async fn call<F, R>(&self, function: F) -> Result<R, DbError>
    where
        F: FnOnce(&mut tokio_rusqlite::rusqlite::Connection) -> tokio_rusqlite::rusqlite::Result<R>
            + Send
            + 'static,
        R: Send + 'static,
    {
        self.connection
            .call(function)
            .await
            .map_err(|error| match &error {
                TokioSqliteError::ConnectionClosed => DbError::WorkerClosed,
                _ => DbError::Operation(error),
            })
    }
}
