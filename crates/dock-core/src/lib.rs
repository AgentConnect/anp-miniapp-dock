#![doc = "Core orchestrator, API registry, host boundary, and shared error crate."]

pub mod api_registry;
pub mod config;
pub mod error;
pub mod host;
pub mod orchestrator;
pub mod runtime;

pub use api_registry::{ApiRegistry, RegisteredApi};
pub use config::{
    redact_runtime_config_text, ConfigReference, HostProviderConfig, NetworkAllowlistReference,
    PathReference, ProviderReference, RuntimeAllowlistConfig, RuntimeConfig, RuntimeConfigIssue,
    RuntimeConfigIssueSeverity, RuntimeConfigLoadError, RuntimeConfigReleaseBlocker,
    RuntimeConfigSource, RuntimeConfigValidation, RuntimeDataBackendConfig, RuntimeDataBackendKind,
    RuntimeIdentityConfig, RuntimeMockProviderFlags, RuntimeObservabilityConfig,
    RuntimeObservabilityLevel, RuntimeProfile, RuntimeResolverConfig, RuntimeTokenIssuerConfig,
    SecretReference, RUNTIME_CONFIG_LOAD_PRIORITY, RUNTIME_CONFIG_REDACTION_MARKER,
    RUNTIME_CONFIG_REDACTION_POLICY, RUNTIME_CONFIG_SCHEMA_VERSION,
};
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
    RuntimeExpireCardsRequest, RuntimeExpireCardsResponse, RuntimeIpcRedaction, RuntimeIpcRequest,
    RuntimeIpcResponse, RuntimeIpcTransport, RuntimeIpcVersionParams, RuntimeLoadSkillResponse,
    RuntimePersistentAuditSink, RuntimeRenderComponentRequest, RuntimeRenderComponentResponse,
    RuntimeResponse, RuntimeResult, RuntimeService, RuntimeSessionContext, RuntimeSkillSummary,
    RuntimeValidateSkillResponse, RuntimeVersion, RUNTIME_API_VERSION, RUNTIME_IPC_BINDING,
    RUNTIME_IPC_TRANSPORT,
};
