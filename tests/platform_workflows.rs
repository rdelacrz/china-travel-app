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
    assert_eq!(requests.len(), 5);
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
