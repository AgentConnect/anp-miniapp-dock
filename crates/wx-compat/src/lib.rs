#![doc = "wx Compatibility Layer host capability and scoped storage crate."]

pub mod model_context;
pub mod permissions;
pub mod request;
pub mod storage;
pub mod unsupported;

pub use model_context::{
    default_app_base_info_js_literal, default_device_info_js_literal, notification_type_js_literal,
    AppBaseInfo, CardEvent, CardEventSink, DeviceInfo, InMemoryCardEventSink, ModelContext,
    RelatedPage, NOTIFICATION_TYPE_EXPIRE, NOTIFICATION_TYPE_INPUT, NOTIFICATION_TYPE_OVERFLOW,
    NOTIFICATION_TYPE_RESULT,
};
pub use permissions::{Capability, CapabilityProfile, PermissionDecision, WxEnvironmentKind};
pub use request::{
    RequestBroker, UnsupportedRequestBroker, WxMethod, WxRequest, WxRequestError, WxResponse,
};
pub use storage::{
    InMemoryScopedStorage, ScopedStorage, StorageError, StorageScope,
    DEFAULT_MAX_STORAGE_KEY_BYTES, DEFAULT_MAX_STORAGE_SCOPE_BYTES,
    DEFAULT_MAX_STORAGE_VALUE_BYTES,
};
pub use unsupported::{
    unsupported_api, unsupported_api_registry, unsupported_api_registry_js_literal, UnsupportedApi,
    UnsupportedApiKind,
};
