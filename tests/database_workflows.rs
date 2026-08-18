use china_travel_app::db::Database;
use china_travel_app::domain::{
    AttachmentRef, NewCalendarEvent, NewTravelDocument, UpdateCalendarEvent, UpdateTravelDocument,
};
use china_travel_app::error::DbError;
use tempfile::tempdir;

#[tokio::test(flavor = "multi_thread")]
async fn fresh_database_migrates_and_supports_trip_checklist_and_document_workflows() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("china-travel.sqlite3");
    let database = Database::open(&path).await.unwrap();

    assert!(database.list_trip_overviews().await.unwrap().is_empty());
    assert!(matches!(
        database.create_trip("   ").await,
        Err(DbError::InvalidInput(_))
    ));

    let trip = database
        .create_trip("  Beijing and Shanghai  ")
        .await
        .unwrap();
    let first = database
        .add_checklist_item(trip.id, "passport")
        .await
        .unwrap();
    let second = database
        .add_checklist_item(trip.id, "power adapter")
        .await
        .unwrap();
    assert_eq!(first.sort_order, 0);
    assert_eq!(second.sort_order, 1);
    assert!(matches!(
        database.rename_checklist_item(first.id, " ").await,
        Err(DbError::InvalidInput(_))
    ));

    database
        .set_checklist_checked(first.id, true)
        .await
        .unwrap();
    database
        .rename_checklist_item(second.id, "universal power adapter")
        .await
        .unwrap();
    database
        .reorder_checklist_items(trip.id, vec![second.id, first.id])
        .await
        .unwrap();
    let reordered = database.list_checklist_items(trip.id).await.unwrap();
    assert_eq!(
        reordered.iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![second.id, first.id]
    );

    let document = NewTravelDocument::new(
        trip.id,
        "Hotel address",
        "The hotel is near https://example.test/booking.".to_string(),
        None,
    )
    .unwrap();
    let saved_document = database.create_document(document).await.unwrap();
    let attachment = AttachmentRef::new(
        "content://com.example.provider/document/42".to_string(),
        Some("booking.pdf".to_string()),
        Some("application/pdf".to_string()),
    )
    .unwrap();
    let updated_document = database
        .update_document(
            UpdateTravelDocument::new(
                saved_document.id,
                "Hotel booking",
                "Updated notes".to_string(),
                Some(attachment.clone()),
            )
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(updated_document.attachment, Some(attachment));

    let overview = database.list_trip_overviews().await.unwrap();
    assert_eq!(overview.len(), 1);
    assert_eq!(overview[0].trip.name, "Beijing and Shanghai");
    assert_eq!(overview[0].checklist_total, 2);
    assert_eq!(overview[0].checklist_completed, 1);
    assert_eq!(overview[0].document_count, 1);
    assert_eq!(overview[0].trip.start_date, None);
    assert_eq!(overview[0].trip.end_date, None);

    let dated_trip = database
        .update_trip_dates(trip.id, Some("2027-04-02"), Some("2027-04-16"))
        .await
        .unwrap();
    assert_eq!(dated_trip.start_date.as_deref(), Some("2027-04-02"));
    assert_eq!(dated_trip.end_date.as_deref(), Some("2027-04-16"));
    assert!(matches!(
        database
            .update_trip_dates(trip.id, Some("2027-04-17"), Some("2027-04-16"))
            .await,
        Err(DbError::InvalidInput(_))
    ));

    let event = database
        .create_calendar_event(
            NewCalendarEvent::new(trip.id, "Flight to Beijing", "2027-04-02", "2027-04-02")
                .unwrap(),
        )
        .await
        .unwrap();
    let events = database.list_calendar_events(trip.id).await.unwrap();
    assert_eq!(events, vec![event.clone()]);
    let updated_event = database
        .update_calendar_event(
            UpdateCalendarEvent::new(
                event.id,
                trip.id,
                "Forbidden City visit",
                "2027-04-03",
                "2027-04-04",
            )
            .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(updated_event.name, "Forbidden City visit");
    assert_eq!(updated_event.start_date, "2027-04-03");
    assert_eq!(updated_event.end_date, "2027-04-04");

    database.delete_checklist_item(second.id).await.unwrap();
    assert_eq!(
        database.list_checklist_items(trip.id).await.unwrap().len(),
        1
    );

    drop(database);
    let reopened = Database::open(&path).await.unwrap();
    let documents = reopened.list_documents(trip.id).await.unwrap();
    assert_eq!(documents.len(), 1);
    assert_eq!(documents[0].name, "Hotel booking");
    assert_eq!(
        documents[0].attachment.as_ref().unwrap().uri,
        "content://com.example.provider/document/42"
    );
    assert_eq!(
        reopened
            .get_trip(trip.id)
            .await
            .unwrap()
            .date_range_label()
            .as_deref(),
        Some("April 2 2027 – April 16 2027")
    );
    assert_eq!(
        reopened.list_calendar_events(trip.id).await.unwrap(),
        vec![updated_event]
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn in_memory_database_is_isolated_and_rejects_missing_records() {
    let database = Database::open_in_memory().await.unwrap();
    assert!(matches!(
        database.get_trip(999).await,
        Err(DbError::NotFound {
            entity: "trip",
            id: 999
        })
    ));
    assert!(database.list_trip_overviews().await.unwrap().is_empty());
}

#[tokio::test(flavor = "multi_thread")]
async fn trip_can_be_created_with_optional_dates() {
    let database = Database::open_in_memory().await.unwrap();
    let trip = database
        .create_trip_with_dates(" Guangzhou", Some("2027-05-01"), Some("2027-05-05"))
        .await
        .unwrap();
    assert_eq!(trip.name, "Guangzhou");
    assert_eq!(trip.start_date.as_deref(), Some("2027-05-01"));
    assert_eq!(trip.end_date.as_deref(), Some("2027-05-05"));
    assert!(matches!(
        database
            .create_trip_with_dates("Invalid dates", Some("2027-05-06"), Some("2027-05-05"))
            .await,
        Err(DbError::InvalidInput(_))
    ));
}

#[tokio::test(flavor = "multi_thread")]
async fn version_one_database_upgrades_to_include_trip_dates_and_calendar_events() {
    let directory = tempdir().unwrap();
    let path = directory.path().join("china-travel-v1.sqlite3");
    {
        let connection = tokio_rusqlite::rusqlite::Connection::open(&path).unwrap();
        connection
            .execute_batch(include_str!("../migrations/0001_initial.sql"))
            .unwrap();
        connection
            .execute("INSERT INTO trips (name) VALUES ('Existing trip')", [])
            .unwrap();
        connection
            .execute_batch("PRAGMA user_version = 1;")
            .unwrap();
    }

    let database = Database::open(&path).await.unwrap();
    let trip = database.get_trip(1).await.unwrap();
    assert_eq!(trip.start_date, None);
    assert_eq!(trip.end_date, None);

    database
        .update_trip_dates(trip.id, Some("2027-10-01"), None)
        .await
        .unwrap();
    let event = database
        .create_calendar_event(
            NewCalendarEvent::new(trip.id, "National Day", "2027-10-01", "2027-10-07").unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(
        database.list_calendar_events(trip.id).await.unwrap(),
        vec![event]
    );
}
