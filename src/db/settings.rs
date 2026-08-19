use super::Database;
use crate::domain::settings::SAFE_MODE_KEY;
use crate::error::DbError;

impl Database {
    pub async fn get_safe_mode_enabled(&self) -> Result<bool, DbError> {
        self.call(|connection| {
            let value: String = connection.query_row(
                "SELECT value FROM app_settings WHERE key = ?1",
                [SAFE_MODE_KEY],
                |row| row.get(0),
            )?;
            Ok(value == "true")
        })
        .await
    }

    pub async fn set_safe_mode_enabled(&self, enabled: bool) -> Result<(), DbError> {
        let value = if enabled { "true" } else { "false" };
        self.call(move |connection| {
            connection.execute(
                "INSERT INTO app_settings (key, value)
                 VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                [SAFE_MODE_KEY, value],
            )?;
            Ok(())
        })
        .await
    }
}
