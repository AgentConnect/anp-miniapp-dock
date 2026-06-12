use serde_json::json;
use wx_compat::{
    InMemoryScopedStorage, ModelContext, ScopedStorage, StorageError, StorageScope,
    DEFAULT_MAX_STORAGE_KEY_BYTES, DEFAULT_MAX_STORAGE_SCOPE_BYTES,
    DEFAULT_MAX_STORAGE_VALUE_BYTES,
};

#[test]
fn storage_is_scoped_by_user_merchant_and_skill() {
    let storage = InMemoryScopedStorage::new();
    let alice = StorageScope::new("did:example:alice", "did:example:merchant-a", "coffee");
    let bob = StorageScope::new("did:example:bob", "did:example:merchant-a", "coffee");
    let merchant_b = StorageScope::new("did:example:alice", "did:example:merchant-b", "coffee");
    let other_skill = StorageScope::new("did:example:alice", "did:example:merchant-a", "tea");

    storage
        .set_storage(&alice, "cart", json!({ "drinkId": "latte" }))
        .expect("set storage");

    assert_eq!(
        storage.get_storage(&alice, "cart").expect("get alice"),
        Some(json!({ "drinkId": "latte" }))
    );
    assert_eq!(storage.get_storage(&bob, "cart").expect("get bob"), None);
    assert_eq!(
        storage
            .get_storage(&merchant_b, "cart")
            .expect("get merchant-b"),
        None
    );
    assert_eq!(
        storage
            .get_storage(&other_skill, "cart")
            .expect("get other skill"),
        None
    );
}

#[test]
fn model_context_builds_storage_scope() {
    let context = ModelContext::new(
        "session-1",
        "coffee",
        "did:example:alice",
        "did:example:merchant",
    );

    assert_eq!(
        context.storage_scope(),
        StorageScope::new("did:example:alice", "did:example:merchant", "coffee")
    );
    assert_eq!(context.get_session_id(), "session-1");
}

#[test]
fn empty_storage_key_is_rejected() {
    let storage = InMemoryScopedStorage::new();
    let scope = StorageScope::new("did:example:alice", "did:example:merchant", "coffee");

    assert_eq!(
        storage.set_storage(&scope, " ", json!(true)),
        Err(StorageError::EmptyKey)
    );
}

#[test]
fn storage_rejects_nul_and_oversized_key() {
    let storage = InMemoryScopedStorage::new();
    let scope = StorageScope::new("did:example:alice", "did:example:merchant", "coffee");

    assert_eq!(
        storage.set_storage(&scope, "cart\0token", json!(true)),
        Err(StorageError::KeyContainsNul)
    );
    assert_eq!(
        storage.set_storage(
            &scope,
            "k".repeat(DEFAULT_MAX_STORAGE_KEY_BYTES + 1),
            json!(true)
        ),
        Err(StorageError::KeyTooLarge)
    );
}

#[test]
fn storage_rejects_sensitive_key_names() {
    let storage = InMemoryScopedStorage::new();
    let scope = StorageScope::new("did:example:alice", "did:example:merchant", "coffee");

    assert_eq!(
        storage.set_storage(&scope, "Authorization", json!("Bearer test-token")),
        Err(StorageError::SensitiveKey)
    );
}

#[test]
fn storage_rejects_oversized_value() {
    let storage = InMemoryScopedStorage::new();
    let scope = StorageScope::new("did:example:alice", "did:example:merchant", "coffee");

    assert_eq!(
        storage.set_storage(
            &scope,
            "cart",
            json!("x".repeat(DEFAULT_MAX_STORAGE_VALUE_BYTES + 1))
        ),
        Err(StorageError::ValueTooLarge)
    );
}

#[test]
fn storage_clear_removes_only_current_scope() {
    let storage = InMemoryScopedStorage::new();
    let alice = StorageScope::new("did:example:alice", "did:example:merchant", "coffee");
    let bob = StorageScope::new("did:example:bob", "did:example:merchant", "coffee");

    storage
        .set_storage(&alice, "cart", json!({ "drinkId": "latte" }))
        .expect("set alice");
    storage
        .set_storage(&bob, "cart", json!({ "drinkId": "tea" }))
        .expect("set bob");
    storage.clear_storage(&alice).expect("clear alice");

    assert_eq!(
        storage.get_storage(&alice, "cart").expect("get alice"),
        None
    );
    assert_eq!(
        storage.get_storage(&bob, "cart").expect("get bob"),
        Some(json!({ "drinkId": "tea" }))
    );
}

#[test]
fn storage_scope_quota_is_enforced_without_partial_write() {
    let storage = InMemoryScopedStorage::new();
    let scope = StorageScope::new("did:example:alice", "did:example:merchant", "coffee");

    storage
        .set_storage(&scope, "cart", json!("ok"))
        .expect("set baseline");
    let payload = json!("x".repeat(1024));
    let mut rejected_key = None;
    for index in 0..DEFAULT_MAX_STORAGE_SCOPE_BYTES {
        let key = format!("quota-{index}");
        match storage.set_storage(&scope, key.clone(), payload.clone()) {
            Ok(()) => {}
            Err(StorageError::QuotaExceeded) => {
                rejected_key = Some(key);
                break;
            }
            Err(error) => panic!("unexpected storage error: {error:?}"),
        }
    }
    let rejected_key = rejected_key.expect("scope quota should be reached");
    assert_eq!(
        storage.get_storage(&scope, "cart").expect("get baseline"),
        Some(json!("ok"))
    );
    assert_eq!(
        storage
            .get_storage(&scope, &rejected_key)
            .expect("get rejected key"),
        None
    );
}
