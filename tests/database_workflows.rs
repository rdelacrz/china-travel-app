use china_travel_app::db::Database;
use china_travel_app::domain::{AttachmentRef, NewTravelDocument, UpdateTravelDocument};
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
