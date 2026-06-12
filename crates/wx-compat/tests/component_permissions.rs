use wx_compat::{
    unsupported_api, unsupported_api_registry, Capability, CapabilityProfile, CardEvent,
    InMemoryCardEventSink, ModelContext, RequestBroker, UnsupportedApiKind,
    UnsupportedRequestBroker, WxRequest, WxRequestError,
};

#[test]
fn atomic_api_profile_allows_request_but_broker_is_step_08_unsupported() {
    let broker = UnsupportedRequestBroker;
    let profile = CapabilityProfile::atomic_api();

    let error = broker
        .request(
            &profile,
            WxRequest::get("https://merchant.example.invalid/drinks"),
        )
        .expect_err("Step 07 broker should not perform network");

    assert!(matches!(error, WxRequestError::Unsupported(message) if message.contains("Step 08")));
}

#[test]
fn atomic_api_profile_does_not_treat_payment_as_real_capability() {
    let profile = CapabilityProfile::atomic_api();

    assert!(!profile.check(Capability::Payment).is_allowed());
}

#[test]
fn component_profile_denies_request_and_timer_by_default() {
    let broker = UnsupportedRequestBroker;
    let profile = CapabilityProfile::component();

    let error = broker
        .request(
            &profile,
            WxRequest::get("https://merchant.example.invalid/drinks"),
        )
        .expect_err("component request must be denied");

    assert!(matches!(error, WxRequestError::Denied(reason) if reason.contains("request")));
    assert!(!profile.check(Capability::Timer).is_allowed());
}

#[test]
fn dynamic_component_profile_can_enable_request_broker_boundary() {
    let profile = CapabilityProfile::component().with_dynamic_component_request();

    assert!(profile.check(Capability::Request).is_allowed());
}

#[test]
fn model_context_records_card_expiration_events() {
    let context = ModelContext::new(
        "session-1",
        "coffee",
        "did:example:alice",
        "did:example:merchant",
    );
    let sink = InMemoryCardEventSink::new();

    context.expire_all_cards(
        &sink,
        ["components/drink-list/index"],
        Some("session".to_owned()),
    );
    context.expire_previous_cards(&sink, ["components/order-confirm/index"], None);

    assert_eq!(
        sink.events(),
        vec![
            CardEvent::ExpireAllCards {
                component_paths: vec!["components/drink-list/index".to_owned()],
                match_policy: Some("session".to_owned()),
            },
            CardEvent::ExpirePreviousCards {
                component_paths: vec!["components/order-confirm/index".to_owned()],
                match_policy: None,
            },
        ]
    );
}

#[test]
fn unsupported_wx_apis_have_explicit_fail_shape() {
    let payment = unsupported_api("requestPayment");

    assert_eq!(
        payment.get("errMsg").and_then(|value| value.as_str()),
        Some("requestPayment:fail unsupported")
    );
    assert_eq!(
        payment.get("code").and_then(|value| value.as_str()),
        Some("unsupported")
    );
    assert!(payment
        .get("reason")
        .and_then(|value| value.as_str())
        .is_some_and(|reason| !reason.contains("4111111111111111")));
    assert!(payment
        .get("suggestion")
        .and_then(|value| value.as_str())
        .is_some_and(|suggestion| suggestion.contains("ConsentGate")));
}

#[test]
fn unsupported_registry_marks_sync_and_async_apis() {
    let registry = unsupported_api_registry();

    assert!(registry.iter().any(|api| {
        api.name == "requestPayment" && matches!(api.kind, UnsupportedApiKind::Async)
    }));
    assert!(registry
        .iter()
        .any(|api| api.name == "getStorageSync" && matches!(api.kind, UnsupportedApiKind::Sync)));
    assert!(registry
        .iter()
        .any(|api| api.name == "wx.cloud.callFunction"));
}

#[test]
fn unknown_unsupported_api_uses_safe_fallback_shape() {
    let result = unsupported_api("../../../Authorization: Bearer secret-token");

    assert_eq!(
        result.get("errMsg").and_then(|value| value.as_str()),
        Some("unknownWxApi:fail unsupported")
    );
    assert_eq!(
        result.get("code").and_then(|value| value.as_str()),
        Some("unsupported")
    );
    let result_json = serde_json::Value::Object(result).to_string();
    assert!(!result_json.contains("secret-token"));
}
