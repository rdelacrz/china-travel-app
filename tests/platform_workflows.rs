use china_travel_app::domain::AttachmentRef;
use china_travel_app::platform::protocol::{
    decode_request_base64, encode_request_base64, NativeRequest,
};
use china_travel_app::platform::FakePlatform;
use china_travel_app::platform::{PickDocumentOutcome, PlatformPort};

#[test]
fn committed_native_bridge_fixture_vectors_are_valid_and_base64_safe() {
    let requests: Vec<NativeRequest> =
        serde_json::from_str(include_str!("fixtures/native_bridge_requests.json")).unwrap();
    assert_eq!(requests.len(), 7);
    for request in requests {
        let encoded = encode_request_base64(&request).unwrap();
        assert!(encoded
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=')));
        assert_eq!(decode_request_base64(&encoded).unwrap(), request);
    }
}

#[tokio::test(flavor = "current_thread")]
async fn fake_platform_models_picker_cancellation_and_permission_release() {
    let attachment = AttachmentRef::new(
        "content://provider/document/99".to_string(),
        Some("visa.pdf".to_string()),
        Some("application/pdf".to_string()),
    )
    .unwrap();
    let platform =
        FakePlatform::with_pick_result(PickDocumentOutcome::Selected(attachment.clone()));
    assert_eq!(
        platform.pick_document(true).await.unwrap(),
        PickDocumentOutcome::Selected(attachment.clone())
    );
    assert_eq!(
        platform.pick_document(true).await.unwrap(),
        PickDocumentOutcome::Cancelled
    );
    platform
        .release_read_permission(&attachment.uri)
        .await
        .unwrap();
    assert_eq!(platform.released_uris(), vec![attachment.uri]);
}

#[tokio::test(flavor = "current_thread")]
async fn fake_platform_creates_and_reads_backup_documents() {
    let platform = FakePlatform::default();
    platform.set_text_document("content://backup/file", "{\"version\":1}");
    assert_eq!(
        platform
            .read_text_document("content://backup/file")
            .await
            .unwrap(),
        "{\"version\":1}"
    );
    assert!(platform
        .create_document(
            "china_travel_app_backup.json",
            "application/json",
            b"{\"version\":1}",
        )
        .await
        .unwrap());
    let created = platform.created_documents();
    assert_eq!(created.len(), 1);
    assert_eq!(created[0].0, "china_travel_app_backup.json");
    assert_eq!(created[0].2, b"{\"version\":1}");
}
