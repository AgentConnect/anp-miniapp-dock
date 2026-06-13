use crate::error::DockCoreError;
use crate::error::ErrorCode;
use crate::orchestrator::{ApiCallContext, ComponentRenderInput};
use consent_audit::RiskLevel;
use consent_audit::{
    ConsentError, ConsentProof, ConsentRequest, ConsentStatus, HostConsentAdapter,
};
use mcp_schema::AtomicApiResult;
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait RuntimeHost {
    fn check_permission(
        &self,
        context: &ApiCallContext,
    ) -> Result<PermissionDecision, DockCoreError>;
}

pub trait ConsentGate {
    fn check_consent(
        &self,
        context: &ApiCallContext,
        request: &ConsentRequest,
    ) -> Result<ConsentDecision, DockCoreError>;
}

pub trait ApiExecutor {
    fn execute(
        &self,
        context: &ApiCallContext,
        component_path: Option<&str>,
    ) -> Result<AtomicApiResult, DockCoreError>;
}

pub trait RenderRouter {
    fn render(
        &self,
        context: &ApiCallContext,
        input: &ComponentRenderInput,
    ) -> Result<RenderOutcome, DockCoreError>;

    fn fallback(
        &self,
        context: &ApiCallContext,
        result: &AtomicApiResult,
        reason: &str,
    ) -> RenderOutcome;
}

pub trait AuditSink {
    fn record(&self, event: AuditEvent) -> Result<(), DockCoreError>;
}

#[derive(Debug, Clone)]
pub struct HostConsentGateAdapter<A> {
    adapter: A,
}

impl<A> HostConsentGateAdapter<A> {
    pub fn new(adapter: A) -> Self {
        Self { adapter }
    }

    pub fn adapter(&self) -> &A {
        &self.adapter
    }
}

impl<A> ConsentGate for HostConsentGateAdapter<A>
where
    A: HostConsentAdapter,
{
    fn check_consent(
        &self,
        _context: &ApiCallContext,
        request: &ConsentRequest,
    ) -> Result<ConsentDecision, DockCoreError> {
        match self.adapter.request_host_consent(request) {
            Ok(decision) if decision.status == ConsentStatus::Approved => Ok(
                ConsentDecision::approved(decision.provider, decision.decision_actor),
            ),
            Ok(decision) => Ok(ConsentDecision::Required(format!(
                "consent denied by {}",
                decision.provider
            ))),
            Err(ConsentError::Denied { .. }) => {
                Ok(ConsentDecision::Required("consent denied".to_owned()))
            }
            Err(ConsentError::ProviderUnavailable { .. }) => Err(DockCoreError::core(
                ErrorCode::ConsentRequired,
                "host consent provider unavailable",
            )),
            Err(ConsentError::Provider(_)) => Err(DockCoreError::core(
                ErrorCode::ConsentRequired,
                "host consent provider failed",
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PermissionDecision {
    Allow,
    Deny { reason: String, reason_code: String },
    Prompt { reason: String, reason_code: String },
    MockAllowedDevOnly { reason: String, reason_code: String },
}

impl PermissionDecision {
    pub fn deny(reason: impl Into<String>, reason_code: impl Into<String>) -> Self {
        Self::Deny {
            reason: reason.into(),
            reason_code: reason_code.into(),
        }
    }

    pub fn prompt(reason: impl Into<String>, reason_code: impl Into<String>) -> Self {
        Self::Prompt {
            reason: reason.into(),
            reason_code: reason_code.into(),
        }
    }

    pub fn mock_allowed_dev_only(
        reason: impl Into<String>,
        reason_code: impl Into<String>,
    ) -> Self {
        Self::MockAllowedDevOnly {
            reason: reason.into(),
            reason_code: reason_code.into(),
        }
    }

    pub fn summary(&self) -> PermissionDecisionSummary {
        match self {
            Self::Allow => PermissionDecisionSummary {
                decision: "allow".to_owned(),
                reason_code: "host_allow".to_owned(),
                reason: "Host allowed this API call".to_owned(),
                dev_only: false,
            },
            Self::Deny {
                reason,
                reason_code,
            } => PermissionDecisionSummary {
                decision: "deny".to_owned(),
                reason_code: reason_code.clone(),
                reason: redact_permission_reason(reason),
                dev_only: false,
            },
            Self::Prompt {
                reason,
                reason_code,
            } => PermissionDecisionSummary {
                decision: "prompt".to_owned(),
                reason_code: reason_code.clone(),
                reason: redact_permission_reason(reason),
                dev_only: false,
            },
            Self::MockAllowedDevOnly {
                reason,
                reason_code,
            } => PermissionDecisionSummary {
                decision: "mock_allowed".to_owned(),
                reason_code: reason_code.clone(),
                reason: redact_permission_reason(reason),
                dev_only: true,
            },
        }
    }
}

fn redact_permission_reason(reason: &str) -> String {
    let lower = reason.to_ascii_lowercase();
    for marker in [
        "authorization",
        "signature",
        "token",
        "secret",
        "private",
        "password",
        "cookie",
        "credential",
    ] {
        if lower.contains(marker) {
            return format!("{marker}=[REDACTED]");
        }
    }
    reason.to_owned()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionDecisionSummary {
    pub decision: String,
    pub reason_code: String,
    pub reason: String,
    pub dev_only: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConsentDecision {
    Approved {
        provider: String,
        decision_actor: String,
    },
    Required(String),
}

impl ConsentDecision {
    pub fn approved(provider: impl Into<String>, decision_actor: impl Into<String>) -> Self {
        Self::Approved {
            provider: provider.into(),
            decision_actor: decision_actor.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RenderOutcome {
    pub renderer: String,
    pub component_path: Option<String>,
    pub payload: Value,
    pub fallback_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AuditEvent {
    pub user_did: Option<String>,
    pub agent_did: Option<String>,
    pub merchant_did: Option<String>,
    pub session_id: String,
    pub skill_id: String,
    pub api_name: String,
    pub risk_level: RiskLevel,
    pub parameter_summary: Value,
    pub permission_decision: PermissionDecisionSummary,
    pub consent_proof: Option<ConsentProof>,
    pub outcome: String,
}
