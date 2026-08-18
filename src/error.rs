#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ValidationError {
    #[error("trip name cannot be blank")]
    BlankTripName,
    #[error("checklist item cannot be blank")]
    BlankChecklistText,
    #[error("document name cannot be blank")]
    BlankDocumentName,
    #[error("event name cannot be blank")]
    BlankCalendarEventName,
    #[error("date must use the YYYY-MM-DD format")]
    InvalidCalendarDate,
    #[error("end date cannot be earlier than start date")]
    CalendarEndBeforeStart,
    #[error("only HTTP and HTTPS links can be opened")]
    UnsupportedUrlScheme,
}

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("database worker closed")]
    WorkerClosed,
    #[error("{entity} {id} was not found")]
    NotFound { entity: &'static str, id: i64 },
    #[error(transparent)]
    InvalidInput(#[from] ValidationError),
    #[error("database migration failed: {0}")]
    Migration(String),
    #[error("database could not be opened: {0}")]
    Open(#[source] tokio_rusqlite::rusqlite::Error),
    #[error("SQLite operation failed: {0}")]
    Operation(#[source] tokio_rusqlite::Error),
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum PlatformError {
    #[error("this capability is unsupported on the current platform")]
    Unsupported,
    #[error("the selected provider did not grant persistent read access")]
    PersistablePermissionDenied,
    #[error("access to the attached file is no longer available")]
    AttachmentUnavailable,
    #[error("no installed app can open this content")]
    NoActivityHandler,
    #[error("only HTTP and HTTPS links can be opened")]
    UnsupportedUrlScheme,
    #[error("native bridge protocol error: {0}")]
    Protocol(String),
    #[error("native operation failed: {0}")]
    Native(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Validation(#[from] ValidationError),
    #[error(transparent)]
    Database(#[from] DbError),
    #[error(transparent)]
    Platform(#[from] PlatformError),
    #[error("application data directory could not be prepared: {0}")]
    Io(#[from] std::io::Error),
}

impl From<dioxus::document::EvalError> for PlatformError {
    fn from(value: dioxus::document::EvalError) -> Self {
        Self::Protocol(value.to_string())
    }
}
