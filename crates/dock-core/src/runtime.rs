use crate::error::{DockCoreError, ErrorCode};
use crate::host::{ApiExecutor, AuditEvent, AuditSink, ConsentGate, RenderOutcome, RenderRouter};
use crate::orchestrator::{
    ApiCallContext, CallOutcome, ComponentAction, ComponentRenderInput, Orchestrator,
};
use crate::RuntimeHost;
use consent_audit::{redact_value, AuditOutcome, AuditRecord, AuditRecordInput};
use mcp_schema::{
    AtomicApiResult, ModelVisibleApiResult, TextContent, ValidationIssue, ValidationReport,
};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use skill_loader::{load_skill, LoadedSkill, SkillPackageError};
use std::error::Error;
use std::fmt;
use std::path::Path;
use std::sync::Arc;

pub const RUNTIME_API_VERSION: &str = "dock.runtime.v1";
pub const RUNTIME_IPC_TRANSPORT: &str = "headless-cli-json";
pub const RUNTIME_IPC_BINDING: &str = "local-process-stdio";

pub type RuntimeError = Box<RuntimeErrorResponse>;
pub type RuntimeResult<T> = Result<RuntimeResponse<T>, RuntimeError>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeVersion {
    pub current: String,
    pub supported: Vec<String>,
}

impl Default for RuntimeVersion {
    fn default() -> Self {
        Self {
            current: RUNTIME_API_VERSION.to_owned(),
            supported: vec![RUNTIME_API_VERSION.to_owned()],
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeIpcRequest {
    pub api_version: String,
    pub request_id: String,
    pub method: String,
    #[serde(default = "empty_object")]
    pub params: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeIpcResponse {
    pub api_version: String,
    pub request_id: String,
    pub method: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RuntimeErrorDto>,
    pub redaction: RuntimeIpcRedaction,
    pub transport: RuntimeIpcTransport,
}

impl RuntimeIpcResponse {
    pub fn ok<T: Serialize>(
        request_id: impl Into<String>,
        method: impl Into<String>,
        response: RuntimeResponse<T>,
    ) -> Self {
        match serde_json::to_value(response) {
            Ok(result) => Self {
                api_version: RUNTIME_API_VERSION.to_owned(),
                request_id: request_id.into(),
                method: method.into(),
                status: "ok".to_owned(),
                result: Some(result),
                error: None,
                redaction: RuntimeIpcRedaction::default(),
                transport: RuntimeIpcTransport::default(),
            },
            Err(error) => Self::error(
                request_id,
                method,
                RuntimeErrorResponse::new(
                    "serialization_failed",
                    format!("runtime response serialization failed: {error}"),
                    None,
                ),
            ),
        }
    }

    pub fn error(
        request_id: impl Into<String>,
        method: impl Into<String>,
        error: RuntimeErrorResponse,
    ) -> Self {
        Self {
            api_version: RUNTIME_API_VERSION.to_owned(),
            request_id: request_id.into(),
            method: method.into(),
            status: "error".to_owned(),
            result: None,
            error: Some(error.error),
            redaction: RuntimeIpcRedaction::default(),
            transport: RuntimeIpcTransport::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeIpcRedaction {
    pub marker: String,
    pub policy: String,
    pub applied_by_default: bool,
}

impl Default for RuntimeIpcRedaction {
    fn default() -> Self {
        Self {
            marker: "[REDACTED]".to_owned(),
            policy: "dock.runtime.redaction.v1".to_owned(),
            applied_by_default: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeIpcTransport {
    pub mode: String,
    pub binding: String,
}

impl Default for RuntimeIpcTransport {
    fn default() -> Self {
        Self {
            mode: RUNTIME_IPC_TRANSPORT.to_owned(),
            binding: RUNTIME_IPC_BINDING.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeIpcVersionParams {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub requested_version: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeResponse<T> {
    pub version: String,
    pub status: String,
    pub data: T,
}

impl<T> RuntimeResponse<T> {
    pub fn ok(data: T) -> Self {
        Self {
            version: RUNTIME_API_VERSION.to_owned(),
            status: "ok".to_owned(),
            data,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeErrorResponse {
    pub version: String,
    pub status: String,
    pub error: RuntimeErrorDto,
}

impl RuntimeErrorResponse {
    pub fn from_core(error: DockCoreError) -> Self {
        let validation = match &error {
            DockCoreError::Validation { report, .. } => Some(redacted_validation_report(report)),
            DockCoreError::Core { .. } => None,
        };
        let code = error.code().as_str().to_owned();
        let message = match error {
            DockCoreError::Core { message, .. } | DockCoreError::Validation { message, .. } => {
                redact_text(&message)
            }
        };
        Self::new(code, message, validation)
    }

    pub fn from_skill_load(error: SkillPackageError) -> Self {
        Self::new("skill_load_failed", redact_text(&error.to_string()), None)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self::new("unsupported", redact_text(&message.into()), None)
    }

    pub fn invalid_method(message: impl Into<String>) -> Self {
        Self::new("invalid_method", redact_text(&message.into()), None)
    }

    pub fn invalid_params(message: impl Into<String>) -> Self {
        Self::new("invalid_params", redact_text(&message.into()), None)
    }

    pub fn boxed(self) -> RuntimeError {
        Box::new(self)
    }

    fn new(
        code: impl Into<String>,
        message: impl Into<String>,
        validation: Option<ValidationReport>,
    ) -> Self {
        Self {
            version: RUNTIME_API_VERSION.to_owned(),
            status: "error".to_owned(),
            error: RuntimeErrorDto {
                code: code.into(),
                message: message.into(),
                validation,
            },
        }
    }
}

impl fmt::Display for RuntimeErrorResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.error.code, self.error.message)
    }
}

impl Error for RuntimeErrorResponse {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeErrorDto {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub validation: Option<ValidationReport>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSkillSummary {
    pub skill_id: String,
    pub package_ref: String,
    pub api_names: Vec<String>,
    pub component_paths: Vec<String>,
    pub validation: ValidationReport,
    pub supply_chain_status: String,
    pub production_ready: bool,
}

impl RuntimeSkillSummary {
    pub fn from_loaded(skill: &LoadedSkill) -> Self {
        Self {
            skill_id: skill_id(skill),
            package_ref: package_ref(skill),
            api_names: skill
                .manifest
                .apis
                .iter()
                .map(|api| api.name.clone())
                .collect(),
            component_paths: skill.components.keys().cloned().collect(),
            validation: skill.validation.clone(),
            supply_chain_status: skill.integrity.status.as_str().to_owned(),
            production_ready: skill.integrity.production_ready,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeValidateSkillResponse {
    pub skill: RuntimeSkillSummary,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLoadSkillResponse {
    pub skill: RuntimeSkillSummary,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeSessionContext {
    pub user_did: Option<String>,
    pub agent_did: Option<String>,
    pub merchant_did: Option<String>,
    pub skill_id: String,
    pub session_id: String,
}

impl RuntimeSessionContext {
    fn to_api_context(
        &self,
        api_name: impl Into<String>,
        arguments: Value,
        capability_token: Option<String>,
    ) -> ApiCallContext {
        ApiCallContext {
            user_did: self.user_did.clone(),
            agent_did: self.agent_did.clone(),
            merchant_did: self.merchant_did.clone(),
            skill_id: self.skill_id.clone(),
            session_id: self.session_id.clone(),
            api_name: api_name.into(),
            arguments,
            capability_token,
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCallRequest {
    pub session: RuntimeSessionContext,
    pub api_name: String,
    #[serde(default)]
    pub arguments: Value,
    #[serde(default, skip_serializing)]
    pub capability_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCallResponse {
    pub api_name: String,
    pub result: AtomicApiResult,
    pub model_visible: ModelVisibleApiResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub render: Option<RenderOutcome>,
}

impl RuntimeCallResponse {
    pub fn from_outcome(api_name: impl Into<String>, outcome: CallOutcome) -> Self {
        let model_visible = outcome.result.model_visible();
        Self {
            api_name: api_name.into(),
            result: outcome.result,
            model_visible,
            render: outcome.render,
        }
    }

    pub fn into_call_outcome(self) -> CallOutcome {
        let model_visible = serde_json::to_value(&self.model_visible).unwrap_or(Value::Null);
        CallOutcome {
            result: self.result,
            model_visible,
            render: self.render,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRenderComponentRequest {
    pub session: RuntimeSessionContext,
    pub api_name: String,
    #[serde(default)]
    pub arguments: Value,
    #[serde(default)]
    pub content: Vec<TextContent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub structured_content: Option<Map<String, Value>>,
    #[serde(default, rename = "_meta", skip_serializing_if = "Option::is_none")]
    pub meta: Option<Map<String, Value>>,
    pub component_path: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeRenderComponentResponse {
    pub render: RenderOutcome,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum RuntimeComponentAction {
    SendFollowUpMessage {
        content: Vec<TextContent>,
    },
    ApiCall {
        name: String,
        #[serde(default)]
        arguments: Value,
    },
    OpenDetailPage {
        url: String,
    },
    ExpirePreviousCards {
        #[serde(default)]
        component_paths: Vec<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        match_policy: Option<String>,
    },
}

impl From<RuntimeComponentAction> for ComponentAction {
    fn from(action: RuntimeComponentAction) -> Self {
        match action {
            RuntimeComponentAction::SendFollowUpMessage { content } => {
                ComponentAction::SendFollowUpMessage { content }
            }
            RuntimeComponentAction::ApiCall { name, arguments } => {
                ComponentAction::ApiCall { name, arguments }
            }
            RuntimeComponentAction::OpenDetailPage { url } => {
                ComponentAction::OpenDetailPage { url }
            }
            RuntimeComponentAction::ExpirePreviousCards {
                component_paths,
                match_policy,
            } => ComponentAction::ExpirePreviousCards {
                component_paths,
                match_policy,
            },
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDispatchComponentActionRequest {
    pub session: RuntimeSessionContext,
    pub source_api_name: String,
    #[serde(default)]
    pub source_arguments: Value,
    pub action: RuntimeComponentAction,
    #[serde(default, skip_serializing)]
    pub capability_token: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeDispatchComponentActionResponse {
    pub handled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call: Option<RuntimeCallResponse>,
    pub boundary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeExpireCardsRequest {
    pub session: RuntimeSessionContext,
    #[serde(default)]
    pub filters: Value,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeExpireCardsResponse {
    pub accepted: bool,
    pub expired_count: usize,
    pub boundary: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAuditRecordsRequest {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAuditRecordsResponse {
    pub records: Vec<RuntimeAuditEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAuditEvent {
    pub user_did: Option<String>,
    pub agent_did: Option<String>,
    pub merchant_did: Option<String>,
    pub session_id: String,
    pub skill_id: String,
    pub api_name: String,
    pub risk_level: String,
    pub parameter_summary: Value,
    pub permission_decision: crate::host::PermissionDecisionSummary,
    pub consent_proof: Option<consent_audit::ConsentProof>,
    pub outcome: String,
}

impl RuntimeAuditEvent {
    pub fn from_event(event: AuditEvent) -> Self {
        let mut consent_proof = event.consent_proof;
        if let Some(proof) = &mut consent_proof {
            proof.parameter_summary = redact_value(&proof.parameter_summary);
        }
        Self {
            user_did: event.user_did,
            agent_did: event.agent_did,
            merchant_did: event.merchant_did,
            session_id: event.session_id,
            skill_id: event.skill_id,
            api_name: event.api_name,
            risk_level: event.risk_level.to_string(),
            parameter_summary: redact_value(&event.parameter_summary),
            permission_decision: event.permission_decision,
            consent_proof,
            outcome: event.outcome,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCloseSessionRequest {
    pub session: RuntimeSessionContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeCloseSessionResponse {
    pub session_id: String,
    pub closed: bool,
    pub boundary: String,
}

pub trait RuntimeAuditReader {
    fn runtime_audit_records(&self) -> Result<Vec<AuditEvent>, DockCoreError>;
}

impl RuntimeAuditReader for () {
    fn runtime_audit_records(&self) -> Result<Vec<AuditEvent>, DockCoreError> {
        Ok(Vec::new())
    }
}

#[derive(Debug, Clone)]
pub struct RuntimePersistentAuditSink<S> {
    sink: Arc<S>,
}

impl<S> RuntimePersistentAuditSink<S> {
    pub fn new(sink: S) -> Self {
        Self {
            sink: Arc::new(sink),
        }
    }

    pub fn sink(&self) -> &S {
        self.sink.as_ref()
    }
}

impl<S> AuditSink for RuntimePersistentAuditSink<S>
where
    S: consent_audit::AuditSink,
{
    fn ensure_available(&self) -> Result<(), DockCoreError> {
        self.sink.ensure_available().map_err(|_| {
            DockCoreError::core(
                ErrorCode::AuditUnavailable,
                "audit sink unavailable for runtime audit",
            )
        })
    }

    fn record(&self, event: AuditEvent) -> Result<(), DockCoreError> {
        self.sink
            .record(audit_record_from_event(event))
            .map_err(|_| {
                DockCoreError::core(
                    ErrorCode::AuditUnavailable,
                    "audit sink unavailable for runtime audit",
                )
            })
    }
}

impl<S> RuntimeAuditReader for RuntimePersistentAuditSink<S>
where
    S: consent_audit::AuditSink + PersistentAuditRecordReader,
{
    fn runtime_audit_records(&self) -> Result<Vec<AuditEvent>, DockCoreError> {
        Ok(self
            .sink
            .records()
            .map_err(|_| {
                DockCoreError::core(
                    ErrorCode::AuditUnavailable,
                    "audit sink unavailable for runtime audit",
                )
            })?
            .into_iter()
            .map(AuditEvent::from)
            .collect())
    }
}

pub trait PersistentAuditRecordReader {
    fn records(&self) -> Result<Vec<AuditRecord>, consent_audit::AuditError>;
}

impl PersistentAuditRecordReader for consent_audit::FileAuditSink {
    fn records(&self) -> Result<Vec<AuditRecord>, consent_audit::AuditError> {
        consent_audit::FileAuditSink::records(self)
    }
}

fn audit_record_from_event(event: AuditEvent) -> AuditRecord {
    let permission_decision = serde_json::to_value(&event.permission_decision)
        .map_or(Value::Null, |value| redact_value(&value));
    let mut record = AuditRecord::new(AuditRecordInput {
        user_did: event.user_did,
        agent_did: event.agent_did,
        merchant_did: event.merchant_did,
        session_id: event.session_id,
        skill_id: event.skill_id,
        api_name: event.api_name,
        risk_level: event.risk_level,
        arguments: &event.parameter_summary,
        consent_proof: event.consent_proof,
        outcome: AuditOutcome::from_label(&event.outcome),
    });
    record.permission_decision = Some(permission_decision);
    record
}

impl From<AuditRecord> for AuditEvent {
    fn from(record: AuditRecord) -> Self {
        let parameter_summary = redact_value(&record.parameter_summary);
        let permission_decision = permission_decision_from_audit_record(&record);
        Self {
            user_did: record.user_did,
            agent_did: record.agent_did,
            merchant_did: record.merchant_did,
            session_id: record.session_id,
            skill_id: record.skill_id,
            api_name: record.api_name,
            risk_level: record.risk_level,
            parameter_summary,
            permission_decision,
            consent_proof: record.consent_proof,
            outcome: record.outcome.to_string(),
        }
    }
}

fn permission_decision_from_audit_record(
    record: &AuditRecord,
) -> crate::host::PermissionDecisionSummary {
    record
        .permission_decision
        .as_ref()
        .and_then(|value| {
            serde_json::from_value::<crate::host::PermissionDecisionSummary>(redact_value(value))
                .ok()
        })
        .unwrap_or_else(|| crate::host::PermissionDecisionSummary {
            decision: "persisted_audit".to_owned(),
            reason_code: "persistent_audit_record".to_owned(),
            reason: "record restored from persistent audit sink".to_owned(),
            dev_only: false,
        })
}

pub fn negotiate_runtime_version(requested: Option<&str>) -> RuntimeResult<RuntimeVersion> {
    match requested {
        Some(version) if version != RUNTIME_API_VERSION => Err(RuntimeErrorResponse::new(
            "unsupported_version",
            format!("runtime API version `{version}` is not supported"),
            None,
        )
        .boxed()),
        _ => Ok(RuntimeResponse::ok(RuntimeVersion::default())),
    }
}

pub fn validate_skill_path(
    skill_ref: impl AsRef<Path>,
) -> RuntimeResult<RuntimeValidateSkillResponse> {
    let skill = load_skill(skill_ref)
        .map_err(|error| RuntimeErrorResponse::from_skill_load(error).boxed())?;
    Ok(RuntimeResponse::ok(RuntimeValidateSkillResponse {
        skill: RuntimeSkillSummary::from_loaded(&skill),
    }))
}

pub fn load_skill_path(skill_ref: impl AsRef<Path>) -> RuntimeResult<RuntimeLoadSkillResponse> {
    let skill = load_skill(skill_ref)
        .map_err(|error| RuntimeErrorResponse::from_skill_load(error).boxed())?;
    Ok(RuntimeResponse::ok(RuntimeLoadSkillResponse {
        skill: RuntimeSkillSummary::from_loaded(&skill),
    }))
}

pub struct RuntimeService<H, C, E, R, A, Q = ()> {
    orchestrator: Orchestrator<H, C, E, R, A>,
    audit_reader: Q,
}

impl<H, C, E, R, A, Q> RuntimeService<H, C, E, R, A, Q>
where
    H: RuntimeHost,
    C: ConsentGate,
    E: ApiExecutor,
    R: RenderRouter,
    A: AuditSink,
    Q: RuntimeAuditReader,
{
    pub fn load_skill(
        skill: LoadedSkill,
        host: H,
        consent: C,
        executor: E,
        renderer: R,
        audit: A,
        audit_reader: Q,
    ) -> Self {
        Self {
            orchestrator: Orchestrator::load_skill(skill, host, consent, executor, renderer, audit),
            audit_reader,
        }
    }

    pub fn from_orchestrator(orchestrator: Orchestrator<H, C, E, R, A>, audit_reader: Q) -> Self {
        Self {
            orchestrator,
            audit_reader,
        }
    }

    pub fn skill(&self) -> &LoadedSkill {
        self.orchestrator.skill()
    }

    pub fn validate_skill(&self) -> RuntimeResponse<RuntimeValidateSkillResponse> {
        RuntimeResponse::ok(RuntimeValidateSkillResponse {
            skill: RuntimeSkillSummary::from_loaded(self.skill()),
        })
    }

    pub fn load_skill_response(&self) -> RuntimeResponse<RuntimeLoadSkillResponse> {
        RuntimeResponse::ok(RuntimeLoadSkillResponse {
            skill: RuntimeSkillSummary::from_loaded(self.skill()),
        })
    }

    pub fn call_api(&self, request: RuntimeCallRequest) -> RuntimeResult<RuntimeCallResponse> {
        let api_name = request.api_name.clone();
        self.orchestrator
            .call_api(request.session.to_api_context(
                request.api_name,
                request.arguments,
                request.capability_token,
            ))
            .map(|outcome| {
                RuntimeResponse::ok(RuntimeCallResponse::from_outcome(api_name, outcome))
            })
            .map_err(|error| RuntimeErrorResponse::from_core(error).boxed())
    }

    pub fn render_component(
        &self,
        request: RuntimeRenderComponentRequest,
    ) -> RuntimeResult<RuntimeRenderComponentResponse> {
        let context = request.session.to_api_context(
            request.api_name.clone(),
            request.arguments.clone(),
            None,
        );
        let input = ComponentRenderInput {
            api_name: request.api_name,
            arguments: request.arguments,
            content: request.content,
            structured_content: request.structured_content,
            meta: request.meta,
            component_path: request.component_path,
        };
        self.orchestrator
            .render_component(&context, input)
            .map(|render| RuntimeResponse::ok(RuntimeRenderComponentResponse { render }))
            .map_err(|error| RuntimeErrorResponse::from_core(error).boxed())
    }

    pub fn dispatch_component_action(
        &self,
        request: RuntimeDispatchComponentActionRequest,
    ) -> RuntimeResult<RuntimeDispatchComponentActionResponse> {
        let call_api_name = match &request.action {
            RuntimeComponentAction::ApiCall { name, .. } => name.clone(),
            RuntimeComponentAction::SendFollowUpMessage { .. }
            | RuntimeComponentAction::OpenDetailPage { .. }
            | RuntimeComponentAction::ExpirePreviousCards { .. } => request.source_api_name.clone(),
        };
        let base_context = request.session.to_api_context(
            request.source_api_name.clone(),
            request.source_arguments,
            request.capability_token,
        );
        let action = request.action;
        self.orchestrator
            .handle_component_action(&base_context, action.into())
            .map(|outcome| {
                RuntimeResponse::ok(RuntimeDispatchComponentActionResponse {
                    handled: outcome.is_some(),
                    call: outcome
                        .map(|outcome| RuntimeCallResponse::from_outcome(call_api_name, outcome)),
                    boundary: "orchestrator".to_owned(),
                })
            })
            .map_err(|error| RuntimeErrorResponse::from_core(error).boxed())
    }

    pub fn expire_cards(
        &self,
        _request: RuntimeExpireCardsRequest,
    ) -> RuntimeResult<RuntimeExpireCardsResponse> {
        Ok(RuntimeResponse::ok(RuntimeExpireCardsResponse {
            accepted: true,
            expired_count: 0,
            boundary: "host-managed-card-store".to_owned(),
        }))
    }

    pub fn get_audit_records(
        &self,
        request: RuntimeAuditRecordsRequest,
    ) -> RuntimeResult<RuntimeAuditRecordsResponse> {
        let records = self
            .audit_reader
            .runtime_audit_records()
            .map_err(|error| RuntimeErrorResponse::from_core(error).boxed())?
            .into_iter()
            .filter(|event| {
                request
                    .session_id
                    .as_ref()
                    .is_none_or(|session_id| &event.session_id == session_id)
                    && request
                        .skill_id
                        .as_ref()
                        .is_none_or(|skill_id| &event.skill_id == skill_id)
            })
            .map(RuntimeAuditEvent::from_event)
            .collect();
        Ok(RuntimeResponse::ok(RuntimeAuditRecordsResponse { records }))
    }

    pub fn close_session(
        &self,
        request: RuntimeCloseSessionRequest,
    ) -> RuntimeResult<RuntimeCloseSessionResponse> {
        Ok(RuntimeResponse::ok(RuntimeCloseSessionResponse {
            session_id: request.session.session_id,
            closed: true,
            boundary: "stateless-runtime-facade".to_owned(),
        }))
    }

    pub fn handle_ipc_request(&self, request: RuntimeIpcRequest) -> RuntimeIpcResponse {
        if request.api_version != RUNTIME_API_VERSION {
            return RuntimeIpcResponse::error(
                request.request_id,
                request.method,
                RuntimeErrorResponse::new(
                    "unsupported_version",
                    "runtime API version is not supported",
                    None,
                ),
            );
        }

        let request_id = request.request_id;
        let method = request.method;
        match method.as_str() {
            "runtime.negotiateVersion" => {
                let params = match parse_ipc_params::<RuntimeIpcVersionParams>(&request.params) {
                    Ok(params) => params,
                    Err(error) => {
                        return RuntimeIpcResponse::error(request_id, method, *error);
                    }
                };
                runtime_ipc_response(
                    request_id,
                    method,
                    negotiate_runtime_version(params.requested_version.as_deref()),
                )
            }
            "runtime.validateSkill" => {
                RuntimeIpcResponse::ok(request_id, method, self.validate_skill())
            }
            "runtime.loadSkill" => {
                RuntimeIpcResponse::ok(request_id, method, self.load_skill_response())
            }
            "runtime.callApi" => runtime_ipc_response(
                request_id,
                method,
                parse_ipc_params(&request.params).and_then(|params| self.call_api(params)),
            ),
            "runtime.renderComponent" => runtime_ipc_response(
                request_id,
                method,
                parse_ipc_params(&request.params).and_then(|params| self.render_component(params)),
            ),
            "runtime.dispatchComponentAction" => runtime_ipc_response(
                request_id,
                method,
                parse_ipc_params(&request.params)
                    .and_then(|params| self.dispatch_component_action(params)),
            ),
            "runtime.expireCards" => runtime_ipc_response(
                request_id,
                method,
                parse_ipc_params(&request.params).and_then(|params| self.expire_cards(params)),
            ),
            "runtime.getAuditRecords" => runtime_ipc_response(
                request_id,
                method,
                parse_ipc_params(&request.params).and_then(|params| self.get_audit_records(params)),
            ),
            "runtime.closeSession" => runtime_ipc_response(
                request_id,
                method,
                parse_ipc_params(&request.params).and_then(|params| self.close_session(params)),
            ),
            _ => RuntimeIpcResponse::error(
                request_id,
                method,
                RuntimeErrorResponse::invalid_method("runtime method is not supported"),
            ),
        }
    }
}

fn skill_id(skill: &LoadedSkill) -> String {
    skill
        .manifest
        .extra
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("unknown")
        .to_owned()
}

fn package_ref(skill: &LoadedSkill) -> String {
    if skill.integrity.digest.value.is_empty() {
        "local-dev-package".to_owned()
    } else {
        format!("sha256:{}", skill.integrity.digest.value)
    }
}

fn redact_text(value: &str) -> String {
    let lower = value.to_ascii_lowercase();
    for marker in [
        "authorization",
        "signature",
        "capabilitytoken",
        "capability_token",
        "token",
        "secret",
        "private",
        "password",
        "cookie",
        "credential",
        "key-",
        "/home/",
        "/users/",
        "\\users\\",
        "c:\\",
        "file:",
    ] {
        if lower.contains(marker) {
            return "[REDACTED]".to_owned();
        }
    }
    value.to_owned()
}

fn redacted_validation_report(report: &ValidationReport) -> ValidationReport {
    ValidationReport {
        errors: report
            .errors
            .iter()
            .map(redacted_validation_issue)
            .collect(),
        warnings: report
            .warnings
            .iter()
            .map(redacted_validation_issue)
            .collect(),
    }
}

fn redacted_validation_issue(issue: &ValidationIssue) -> ValidationIssue {
    ValidationIssue {
        level: issue.level,
        category: issue.category,
        path: redact_text(&issue.path),
        message: redact_text(&issue.message),
        suggestion: issue.suggestion.as_deref().map(redact_text),
    }
}

fn runtime_ipc_response<T: Serialize>(
    request_id: String,
    method: String,
    result: RuntimeResult<T>,
) -> RuntimeIpcResponse {
    match result {
        Ok(response) => RuntimeIpcResponse::ok(request_id, method, response),
        Err(error) => RuntimeIpcResponse::error(request_id, method, *error),
    }
}

fn parse_ipc_params<T: DeserializeOwned>(params: &Value) -> Result<T, RuntimeError> {
    let params = if params.is_null() {
        empty_object()
    } else {
        params.clone()
    };
    serde_json::from_value(params).map_err(|error| {
        RuntimeErrorResponse::invalid_params(format!("runtime IPC params are invalid: {error}"))
            .boxed()
    })
}

fn empty_object() -> Value {
    Value::Object(Map::new())
}

#[allow(dead_code)]
fn core_error(code: ErrorCode, message: impl Into<String>) -> RuntimeErrorResponse {
    RuntimeErrorResponse::from_core(DockCoreError::core(code, message))
}
