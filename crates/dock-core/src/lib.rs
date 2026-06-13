#![doc = "Core orchestrator, API registry, host boundary, and shared error crate."]

pub mod api_registry;
pub mod error;
pub mod host;
pub mod orchestrator;
pub mod runtime;

pub use api_registry::{ApiRegistry, RegisteredApi};
pub use error::{DockCoreError, ErrorCode};
pub use host::{
    ApiExecutor, AuditEvent, AuditSink, ConsentDecision, ConsentGate, HostConsentGateAdapter,
    PermissionDecision, PermissionDecisionSummary, RenderOutcome, RenderRouter, RuntimeHost,
};
pub use orchestrator::{
    ApiCallContext, CallOutcome, ComponentAction, ComponentRenderInput, Orchestrator,
};
pub use runtime::{
    load_skill_path, negotiate_runtime_version, validate_skill_path, RuntimeAuditEvent,
    RuntimeAuditReader, RuntimeAuditRecordsRequest, RuntimeAuditRecordsResponse,
    RuntimeCallRequest, RuntimeCallResponse, RuntimeCloseSessionRequest,
    RuntimeCloseSessionResponse, RuntimeComponentAction, RuntimeDispatchComponentActionRequest,
    RuntimeDispatchComponentActionResponse, RuntimeErrorDto, RuntimeErrorResponse,
    RuntimeExpireCardsRequest, RuntimeExpireCardsResponse, RuntimeLoadSkillResponse,
    RuntimeRenderComponentRequest, RuntimeRenderComponentResponse, RuntimeResponse, RuntimeResult,
    RuntimeService, RuntimeSessionContext, RuntimeSkillSummary, RuntimeValidateSkillResponse,
    RuntimeVersion, RUNTIME_API_VERSION,
};
