#![doc = "wx Compatibility Layer host capability and scoped storage crate."]

pub mod model_context;
pub mod permissions;
pub mod request;
pub mod storage;
pub mod unsupported;

pub use model_context::{
    notification_type_js_literal, CardEvent, CardEventSink, DeviceInfo, InMemoryCardEventSink,
    ModelContext, RelatedPage, NOTIFICATION_TYPE_EXPIRE, NOTIFICATION_TYPE_INPUT,
    NOTIFICATION_TYPE_OVERFLOW, NOTIFICATION_TYPE_RESULT,
};
pub use permissions::{Capability, CapabilityProfile, PermissionDecision, WxEnvironmentKind};
pub use request::{
    RequestBroker, UnsupportedRequestBroker, WxMethod, WxRequest, WxRequestError, WxResponse,
};
pub use storage::{InMemoryScopedStorage, ScopedStorage, StorageError, StorageScope};
pub use unsupported::{
    unsupported_api, unsupported_api_registry, unsupported_api_registry_js_literal, UnsupportedApi,
    UnsupportedApiKind,
};
