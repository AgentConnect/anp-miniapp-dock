use anp::authentication::AuthMode;
use anp_adapter::{
    sign_challenge_proof, ChallengeLoginRequest, ChallengeLoginResponse, ChallengeProofError,
    ChallengeProofPayload, DidChallenge as AdapterDidChallenge, DidCredentialConfig,
    DidCredentialError, FileDidCredentialProvider, IdentitySession,
};
use card_spec::{fallback_from_result, FallbackReason};
use clap::{Parser, Subcommand};
use component_runtime::{
    ComponentEvent, ComponentInput, ComponentInstance, ComponentMetadata, ComponentPackage,
    ComponentRenderOutput, ComponentVmAction, RenderEventKind, RenderNode,
};
use consent_audit::{ConsentRequest, DEV_HEADLESS_CONSENT_PROVIDER, DEV_HEADLESS_DECISION_ACTOR};
use dock_core::{
    ApiCallContext, AuditEvent, AuditSink, ComponentRenderInput, ConsentDecision, ConsentGate,
    DockCoreError, ErrorCode, PermissionDecision, RenderOutcome, RenderRouter, RuntimeAuditReader,
    RuntimeCallRequest, RuntimeErrorResponse, RuntimeHost, RuntimeIpcRequest, RuntimeIpcResponse,
    RuntimeService, RuntimeSessionContext,
};
use js_runtime_quickjs::{ApiVm, HostDidAuthConfig};
use mcp_schema::{
    AtomicApiResult, ComponentDeclaration, ValidationIssueCategory, ValidationReport,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use skill_loader::{load_skill, resolve_component_path, LoadedSkill};
use std::cell::RefCell;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

const DEFAULT_SESSION_ID: &str = "session-cli";
const DEFAULT_SKILL_ID: &str = "coffee";
const DEFAULT_USER_DID: &str = "did:wba:user.example";
const DEFAULT_AGENT_DID: &str = "did:wba:agent.example";
const DEFAULT_MERCHANT_DID: &str = "did:wba:coffee-merchant.example";
const DEFAULT_IDENTITY_DIR: &str = "examples/identity";
const DEFAULT_DID_DOCUMENT_FILE: &str = "did_document.json";
const DEFAULT_PRIVATE_KEY_FILE: &str = "key-1-private.pem";

#[derive(Debug, Parser)]
#[command(name = "dock-cli", about = "MiniApp MCP Skill runtime developer CLI")]
pub struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Validate {
        skill: PathBuf,
    },
    CallApi {
        skill: PathBuf,
        api_name: String,
        json_args: String,
    },
    PreviewComponent {
        skill: PathBuf,
        component_path: String,
        json_input: String,
    },
    PreviewCard {
        result_json: String,
    },
    RuntimeJson {
        skill: PathBuf,
        request_json: String,
    },
    RunDemo {
        #[arg(long)]
        skill: PathBuf,
        #[arg(long)]
        server: String,
        #[arg(long)]
        did_document: Option<PathBuf>,
        #[arg(long)]
        private_key: Option<PathBuf>,
        #[arg(long)]
        user_did: Option<String>,
        #[arg(long)]
        agent_did: Option<String>,
        #[arg(long)]
        identity_handle: Option<String>,
        #[arg(long)]
        identity_root: Option<PathBuf>,
    },
}

pub fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    run_with_writer(cli, &mut std::io::stdout())
}

pub fn run_with_writer(mut cli: Cli, writer: &mut impl Write) -> Result<(), CliError> {
    let output = cli.execute()?;
    write_json(writer, &output)
}

impl Cli {
    pub fn try_parse_from_args<I, T>(args: I) -> Result<Self, CliError>
    where
        I: IntoIterator<Item = T>,
        T: Into<std::ffi::OsString> + Clone,
    {
        Self::try_parse_from(args).map_err(CliError::from)
    }

    fn execute(&mut self) -> Result<Value, CliError> {
        match &self.command {
            Command::Validate { skill } => validate(skill),
            Command::CallApi {
                skill,
                api_name,
                json_args,
            } => call_api(skill, api_name, json_args),
            Command::PreviewComponent {
                skill,
                component_path,
                json_input,
            } => preview_component(skill, component_path, json_input),
            Command::PreviewCard { result_json } => preview_card(result_json),
            Command::RuntimeJson {
                skill,
                request_json,
            } => runtime_json(skill, request_json),
            Command::RunDemo {
                skill,
                server,
                did_document,
                private_key,
                user_did,
                agent_did,
                identity_handle,
                identity_root,
            } => {
                let auth_config = DemoAuthConfig::from_inputs_optional(
                    did_document.clone(),
                    private_key.clone(),
                    user_did.clone(),
                    agent_did.clone(),
                    identity_handle.clone(),
                    identity_root.clone(),
                    EnvConfigSource,
                )?;
                run_demo(skill, server, auth_config.as_ref())
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{0}")]
    Args(#[from] clap::Error),

    #[error("failed to load skill: {0}")]
    Skill(#[from] skill_loader::SkillPackageError),

    #[error("failed to load component: {0}")]
    ComponentLoad(#[from] component_runtime::ComponentLoadError),

    #[error("component VM failed: {0}")]
    ComponentVm(#[from] component_runtime::ComponentVmError),

    #[error("API VM failed: {0}")]
    ApiVm(#[from] js_runtime_quickjs::ApiVmError),

    #[error("runtime call failed: {0}")]
    Core(#[from] DockCoreError),

    #[error("runtime call failed: {0}")]
    Runtime(#[from] Box<RuntimeErrorResponse>),

    #[error("invalid JSON for {label}: {source}")]
    Json {
        label: String,
        source: serde_json::Error,
    },

    #[error("I/O failed: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP request failed: {0}")]
    Http(String),

    #[error("demo flow failed: {0}")]
    Demo(String),

    #[error("DID credential configuration failed: {0}")]
    Credential(#[from] DidCredentialError),

    #[error("DID challenge proof failed: {0}")]
    ChallengeProof(#[from] ChallengeProofError),
}

#[derive(Clone, PartialEq, Eq)]
struct DemoAuthConfig {
    user_did: String,
    agent_did: Option<String>,
    credential: DidCredentialConfig,
}

impl fmt::Debug for DemoAuthConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DemoAuthConfig")
            .field("user_did", &self.user_did)
            .field("agent_did", &self.agent_did)
            .field("credential", &"[REDACTED]")
            .finish()
    }
}

impl DemoAuthConfig {
    fn from_inputs_optional(
        did_document: Option<PathBuf>,
        private_key: Option<PathBuf>,
        user_did: Option<String>,
        agent_did: Option<String>,
        identity_handle: Option<String>,
        identity_root: Option<PathBuf>,
        env: impl ConfigSource,
    ) -> Result<Option<Self>, DidCredentialError> {
        Self::from_inputs(
            did_document,
            private_key,
            user_did,
            agent_did,
            identity_handle,
            identity_root,
            env,
        )
        .map(Some)
    }

    fn from_inputs(
        did_document: Option<PathBuf>,
        private_key: Option<PathBuf>,
        user_did: Option<String>,
        agent_did: Option<String>,
        identity_handle: Option<String>,
        identity_root: Option<PathBuf>,
        env: impl ConfigSource,
    ) -> Result<Self, DidCredentialError> {
        let did_document = did_document.or_else(|| env.path("ANP_DOCK_DID_DOCUMENT"));
        let private_key = private_key.or_else(|| env.path("ANP_DOCK_PRIVATE_KEY"));
        let user_did = user_did.or_else(|| env.string("ANP_DOCK_USER_DID"));
        let agent_did = agent_did.or_else(|| env.string("ANP_DOCK_AGENT_DID"));
        let identity_handle = identity_handle.or_else(|| env.string("ANP_DOCK_IDENTITY_HANDLE"));
        let identity_root = identity_root.or_else(|| env.path("ANP_DOCK_IDENTITY_ROOT"));

        if did_document.is_some() || private_key.is_some() || user_did.is_some() {
            if did_document.is_none() || private_key.is_none() {
                return Err(DidCredentialError::InvalidIdentity);
            }
            if identity_handle.is_some() || identity_root.is_some() {
                return Err(DidCredentialError::InvalidIdentity);
            }
            return Self::from_credential_paths(
                did_document.expect("checked did document"),
                private_key.expect("checked private key"),
                user_did,
                agent_did,
            );
        }

        if identity_handle.is_some() || identity_root.is_some() {
            let handle = identity_handle.ok_or(DidCredentialError::Unavailable)?;
            let root = identity_root.ok_or(DidCredentialError::Unavailable)?;
            let resolved = resolve_identity_from_store(&root, &handle)?;
            resolved.credential.validate()?;
            return Ok(Self {
                agent_did,
                ..resolved
            });
        }

        let project_root = default_project_root()?;
        Self::from_default_project_identity(&project_root, agent_did)
    }

    fn from_default_project_identity(
        project_root: &Path,
        agent_did: Option<String>,
    ) -> Result<Self, DidCredentialError> {
        let (did_document, private_key) = default_identity_paths(project_root);
        Self::from_credential_paths(did_document, private_key, None, agent_did)
    }

    fn from_credential_paths(
        did_document: PathBuf,
        private_key: PathBuf,
        user_did: Option<String>,
        agent_did: Option<String>,
    ) -> Result<Self, DidCredentialError> {
        let user_did = match user_did {
            Some(user_did) => user_did,
            None => did_from_document_path(&did_document)?,
        };
        let credential = DidCredentialConfig::new(did_document, private_key)
            .without_private_key_permission_check();
        credential.validate()?;
        Ok(Self {
            user_did,
            agent_did,
            credential,
        })
    }

    fn redacted_summary(&self) -> Value {
        json!({
            "userDid": self.user_did,
            "agentDid": self.agent_did,
            "credential": {
                "didDocument": "[CONFIGURED]",
                "privateKey": "[REDACTED]"
            }
        })
    }
}

trait ConfigSource: Copy {
    fn string(self, name: &str) -> Option<String>;

    fn path(self, name: &str) -> Option<PathBuf> {
        self.string(name)
            .filter(|value| !value.trim().is_empty())
            .map(PathBuf::from)
    }
}

#[derive(Debug, Clone, Copy)]
struct EnvConfigSource;

impl ConfigSource for EnvConfigSource {
    fn string(self, name: &str) -> Option<String> {
        std::env::var(name)
            .ok()
            .map(|value| value.trim().to_owned())
            .filter(|value| !value.is_empty())
    }
}

fn resolve_identity_from_store(
    root: &Path,
    handle: &str,
) -> Result<DemoAuthConfig, DidCredentialError> {
    let root = identity_store_dir(root);
    let identity_dir = identity_dir_for_handle(&root, handle)?;
    let identity = read_json(identity_dir.join("identity.json"))?;
    let user_did = identity
        .get("did")
        .or_else(|| identity.get("userDid"))
        .and_then(Value::as_str)
        .ok_or(DidCredentialError::InvalidIdentity)?
        .to_owned();
    Ok(DemoAuthConfig {
        user_did,
        agent_did: None,
        credential: DidCredentialConfig::new(
            identity_dir.join("did_document.json"),
            identity_dir.join("key-1-private.pem"),
        ),
    })
}

fn default_project_root() -> Result<PathBuf, DidCredentialError> {
    let current_dir = std::env::current_dir().map_err(|_| DidCredentialError::Unavailable)?;
    find_project_root_from(&current_dir).ok_or(DidCredentialError::Unavailable)
}

fn find_project_root_from(start: &Path) -> Option<PathBuf> {
    let mut current = if start.is_file() {
        start.parent()?.to_path_buf()
    } else {
        start.to_path_buf()
    };
    loop {
        if current.join("Cargo.toml").is_file() && current.join("examples/coffee-skill").is_dir() {
            return Some(current);
        }
        if !current.pop() {
            return None;
        }
    }
}

fn default_identity_paths(project_root: &Path) -> (PathBuf, PathBuf) {
    let identity_dir = project_root.join(DEFAULT_IDENTITY_DIR);
    (
        identity_dir.join(DEFAULT_DID_DOCUMENT_FILE),
        identity_dir.join(DEFAULT_PRIVATE_KEY_FILE),
    )
}

fn identity_store_dir(root: &Path) -> PathBuf {
    if root.join("index.json").exists() || root.file_name().is_some_and(|name| name == "identities")
    {
        return root.to_path_buf();
    }
    root.join("identities")
}

fn identity_dir_for_handle(root: &Path, handle: &str) -> Result<PathBuf, DidCredentialError> {
    if let Some(path) = identity_dir_from_index(root, handle)? {
        return Ok(path);
    }
    let entries = std::fs::read_dir(root).map_err(|_| DidCredentialError::Unavailable)?;
    let mut matches = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|_| DidCredentialError::Unavailable)?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Ok(identity) = read_json(path.join("identity.json")) else {
            continue;
        };
        let matches_handle = identity
            .get("handle")
            .or_else(|| identity.get("name"))
            .and_then(Value::as_str)
            == Some(handle);
        if matches_handle {
            matches.push(path);
        }
    }
    match matches.len() {
        1 => Ok(matches.remove(0)),
        _ => Err(DidCredentialError::InvalidIdentity),
    }
}

fn identity_dir_from_index(
    root: &Path,
    handle: &str,
) -> Result<Option<PathBuf>, DidCredentialError> {
    let index_path = root.join("index.json");
    if !index_path.exists() {
        return Ok(None);
    }
    let index = read_json(index_path)?;
    let entry = index
        .get(handle)
        .or_else(|| index.get("handles").and_then(|handles| handles.get(handle)));
    let Some(entry) = entry else {
        return Ok(None);
    };
    let relative = entry
        .as_str()
        .or_else(|| entry.get("dir").and_then(Value::as_str))
        .or_else(|| entry.get("path").and_then(Value::as_str))
        .ok_or(DidCredentialError::InvalidIdentity)?;
    Ok(Some(root.join(relative)))
}

fn read_json(path: PathBuf) -> Result<Value, DidCredentialError> {
    let content = std::fs::read_to_string(&path).map_err(|_| DidCredentialError::Unavailable)?;
    serde_json::from_str(&content).map_err(|_| DidCredentialError::InvalidIdentity)
}

fn did_from_document_path(path: &Path) -> Result<String, DidCredentialError> {
    let document = read_json(path.to_path_buf())?;
    document
        .get("id")
        .and_then(Value::as_str)
        .filter(|did| !did.trim().is_empty())
        .map(str::to_owned)
        .ok_or(DidCredentialError::InvalidIdentity)
}

fn validate(skill_path: &Path) -> Result<Value, CliError> {
    let skill = load_skill(skill_path)?;
    let registration = validate_api_registration(&skill);
    let api_reports = validate_api_reports(&skill, registration.as_ref());
    let component_reports = validate_component_reports(&skill);
    let permissions = validate_permissions(&component_reports);
    let risks = validate_risks(&skill);
    let fallbacks = validate_fallbacks(&skill, &component_reports);
    let release_blockers = validate_release_blockers(&skill, registration.as_ref());
    let compatibility_level = compatibility_level(&skill.validation, &release_blockers);

    Ok(json!({
        "status": "ok",
        "compatibilityLevel": compatibility_level,
        "skillRoot": skill.root,
        "skillId": skill_id(&skill),
        "apis": skill.manifest.apis.iter().map(|api| api.name.as_str()).collect::<Vec<_>>(),
        "components": skill.components.keys().collect::<Vec<_>>(),
        "compatibilityReport": {
            "status": "ok",
            "compatibilityLevel": compatibility_level,
            "apis": api_reports,
            "components": component_reports,
            "permissions": permissions,
            "risks": risks,
            "supplyChain": supply_chain_report(&skill),
            "fallbacks": fallbacks,
            "releaseBlockers": release_blockers,
        },
        "validation": validation_summary(&skill.validation)
    }))
}

fn validate_api_registration(skill: &LoadedSkill) -> Result<Vec<String>, String> {
    ApiVm::load_skill(skill.clone())
        .map(|vm| {
            vm.registered_apis()
                .iter()
                .map(|api| api.name.clone())
                .collect()
        })
        .map_err(|error| redact_text(&error.to_string()))
}

fn validate_api_reports(
    skill: &LoadedSkill,
    registration: Result<&Vec<String>, &String>,
) -> Vec<Value> {
    skill
        .manifest
        .apis
        .iter()
        .map(|api| {
            let input_formats = api
                .input_formats()
                .into_iter()
                .map(|field| json!({ "path": field.path, "format": field.format }))
                .collect::<Vec<_>>();
            let registered = registration
                .as_ref()
                .is_ok_and(|registered| registered.iter().any(|name| name == &api.name));
            json!({
                "name": api.name,
                "registered": registered,
                "componentPath": api.component_path(),
                "inputFormats": input_formats,
                "hasOutputSchema": api.output_schema.is_some(),
                "risk": api.meta.as_ref().and_then(|meta| meta.anp.as_ref()).and_then(|anp| anp.get("risk")).cloned(),
                "status": if registered { "declared-and-registered" } else { "registration-unverified" },
            })
        })
        .collect()
}

fn validate_component_reports(skill: &LoadedSkill) -> Vec<Value> {
    skill
        .manifest
        .components
        .iter()
        .map(|component| component_report(component, skill))
        .collect()
}

fn component_report(component: &ComponentDeclaration, skill: &LoadedSkill) -> Value {
    let loaded = skill.components.contains_key(&component.path);
    let metadata = manifest_component_metadata(&skill.manifest, &component.path).ok();
    json!({
        "path": component.path,
        "loaded": loaded,
        "relatedPage": metadata.as_ref().and_then(|metadata| metadata.related_page.clone()),
        "permissions": {
            "dynamic": metadata.as_ref().is_some_and(|metadata| metadata.dynamic),
            "scopeDynamic": metadata.as_ref().and_then(|metadata| metadata.scope_dynamic.clone())
        },
        "expirable": metadata.as_ref().is_some_and(|metadata| metadata.expirable),
        "expiredText": metadata.as_ref().and_then(|metadata| metadata.expired_text.clone()),
        "runtimeMetadata": metadata,
        "fallback": if loaded { Value::Null } else { json!("component_load_failed") },
    })
}

fn validate_permissions(component_reports: &[Value]) -> Value {
    let dynamic_components = component_reports
        .iter()
        .filter_map(|component| {
            component
                .get("permissions")
                .and_then(|permissions| permissions.get("dynamic"))
                .and_then(Value::as_bool)
                .filter(|dynamic| *dynamic)
                .and_then(|_| component.get("path"))
                .and_then(Value::as_str)
        })
        .collect::<Vec<_>>();

    json!({
        "dynamicComponents": dynamic_components,
        "policy": "dynamic request/timer declarations require Step 02-05 runtime gate plus explicit Host production policy",
    })
}

fn validate_risks(skill: &LoadedSkill) -> Vec<Value> {
    skill
        .manifest
        .apis
        .iter()
        .filter_map(|api| {
            api.meta
                .as_ref()
                .and_then(|meta| meta.anp.as_ref())
                .and_then(|anp| anp.get("risk"))
                .map(|risk| {
                    json!({
                        "api": api.name,
                        "risk": risk,
                        "consentRequired": matches!(risk.as_str(), Some("order" | "payment" | "high" | "l3" | "l4")),
                    })
                })
        })
        .collect()
}

fn validate_fallbacks(skill: &LoadedSkill, component_reports: &[Value]) -> Vec<Value> {
    let mut fallbacks = Vec::new();
    for api in &skill.manifest.apis {
        if api.component_path().is_none() {
            fallbacks.push(json!({
                "api": api.name,
                "fallback": "card-spec",
                "reason": "no_component_path",
            }));
        }
    }
    for component in component_reports {
        if component.get("loaded").and_then(Value::as_bool) == Some(false) {
            fallbacks.push(json!({
                "componentPath": component.get("path"),
                "fallback": "card-spec",
                "reason": "component_load_failed",
            }));
        }
    }
    fallbacks
}

fn validate_release_blockers(
    skill: &LoadedSkill,
    registration: Result<&Vec<String>, &String>,
) -> Vec<Value> {
    let mut blockers = Vec::new();
    if let Err(error) = registration {
        blockers.push(json!({
            "code": "api_registration_mismatch",
            "message": error,
            "suggestion": "Keep apis[].name aligned with index.js registerAPI calls before production validation.",
        }));
    }

    for warning in &skill.validation.warnings {
        if warning.category == ValidationIssueCategory::Production {
            blockers.push(json!({
                "code": "production_warning",
                "path": warning.path.clone(),
                "message": warning.message.clone(),
                "suggestion": warning.suggestion.clone(),
            }));
        }
    }

    if !skill.integrity.production_ready {
        blockers.push(json!({
            "code": "supply_chain",
            "status": skill.integrity.status.as_str(),
            "issueCodes": skill.integrity.issue_codes,
            "message": "Skill package is not production-ready under the Step 03-06 supply-chain gate.",
            "suggestion": "Attach publisher DID, sha256 digest, package signature, and trusted publisher policy before production release.",
        }));
    }

    blockers
}

fn supply_chain_report(skill: &LoadedSkill) -> Value {
    json!({
        "digest": {
            "algorithm": skill.integrity.digest.algorithm,
            "value": skill.integrity.digest.value,
        },
        "status": skill.integrity.status.as_str(),
        "publisherDid": skill.integrity.publisher_did,
        "signature": {
            "algorithm": skill.integrity.signature_algorithm,
            "keyId": skill.integrity.signature_key_id,
        },
        "trustedPublisher": skill.integrity.trusted_publisher,
        "quarantine": skill.integrity.quarantine,
        "productionReady": skill.integrity.production_ready,
        "issueCodes": skill.integrity.issue_codes,
        "warnings": skill.integrity.warnings,
    })
}

fn compatibility_level(report: &ValidationReport, release_blockers: &[Value]) -> &'static str {
    if !report.is_valid() {
        "invalid"
    } else if !release_blockers.is_empty() {
        "demo-only"
    } else if report.warnings.is_empty() {
        "supported"
    } else {
        "compatible-with-warnings"
    }
}

fn call_api(skill_path: &Path, api_name: &str, json_args: &str) -> Result<Value, CliError> {
    let args = parse_json(json_args, "jsonArgs")?;
    let auth_config = if requires_remote_auth(&args) {
        DemoAuthConfig::from_inputs_optional(None, None, None, None, None, None, EnvConfigSource)?
    } else {
        None
    };
    let identity = auth_config
        .as_ref()
        .map(|auth_config| RuntimeIdentity {
            user_did: auth_config.user_did.clone(),
            agent_did: auth_config.agent_did.clone(),
            merchant_did: DEFAULT_MERCHANT_DID.to_owned(),
        })
        .unwrap_or_else(RuntimeIdentity::default_demo);
    let runtime = RuntimeHarness::load(skill_path, identity, auth_config.as_ref())?;
    let outcome = runtime.call(api_name, args)?;
    Ok(json!({
        "status": "ok",
        "apiName": api_name,
        "result": outcome.result,
        "modelVisible": outcome.model_visible,
        "render": render_outcome_json(outcome.render.as_ref()),
        "audit": audit_events_json(&runtime.audit_events())
    }))
}

fn requires_remote_auth(args: &Value) -> bool {
    args.get("serverUrl")
        .or_else(|| args.get("remoteBaseUrl"))
        .and_then(Value::as_str)
        .is_some_and(|url| !url.trim().is_empty())
}

fn preview_component(
    skill_path: &Path,
    component_path: &str,
    json_input: &str,
) -> Result<Value, CliError> {
    let input = parse_component_input(json_input)?;
    let package = load_component_package(skill_path, component_path)?;
    let mut instance = ComponentInstance::new(package)?;
    let metadata = load_component_metadata(skill_path, component_path)?;
    let input = ComponentInput {
        component_metadata: metadata,
        ..input
    };
    let outcome = instance.mount(input)?;
    Ok(json!({
        "status": "ok",
        "componentPath": component_path,
        "render": component_render_json(&outcome.render),
        "actions": outcome.actions,
        "metadata": outcome.metadata,
        "trace": outcome.trace,
        "state": outcome.state
    }))
}

fn preview_card(result_json: &str) -> Result<Value, CliError> {
    let result = parse_atomic_result(result_json)?;
    let reason = if result.is_error {
        FallbackReason::ApiError
    } else if result
        .structured_content
        .as_ref()
        .is_some_and(Map::is_empty)
        || result.structured_content.is_none()
    {
        FallbackReason::EmptyStructuredContent
    } else {
        FallbackReason::HostRendererUnavailable
    };
    let card = fallback_from_result(&result, reason);
    Ok(json!({
        "status": "ok",
        "card": card
    }))
}

fn runtime_json(skill_path: &Path, request_json: &str) -> Result<Value, CliError> {
    let request: RuntimeIpcRequest = match serde_json::from_str(request_json) {
        Ok(request) => request,
        Err(_) => {
            let response = RuntimeIpcResponse::error(
                "",
                "runtime.parseRequest",
                RuntimeErrorResponse::invalid_params(
                    "runtime IPC request JSON is invalid or does not match the envelope schema",
                ),
            );
            return serde_json::to_value(response).map_err(|source| CliError::Json {
                label: "runtime JSON response".to_owned(),
                source,
            });
        }
    };
    let runtime = RuntimeHarness::load(
        skill_path,
        RuntimeIdentity::default_demo(),
        Option::<&DemoAuthConfig>::None,
    )?;
    serde_json::to_value(runtime.handle_ipc_request(request)).map_err(|source| CliError::Json {
        label: "runtime JSON response".to_owned(),
        source,
    })
}

fn run_demo(
    skill_path: &Path,
    server: &str,
    auth_config: Option<&DemoAuthConfig>,
) -> Result<Value, CliError> {
    let auth_config = auth_config.ok_or(DidCredentialError::Unavailable)?;
    let auth = DemoHttpClient::new(server).login(auth_config)?;
    let server_business =
        DemoHttpClient::new(server).coffee_business_check(&auth.capability_token)?;
    let runtime = RuntimeHarness::load(
        skill_path,
        RuntimeIdentity {
            user_did: auth_config.user_did.clone(),
            agent_did: auth_config.agent_did.clone(),
            merchant_did: auth.merchant_did.clone(),
        },
        Some(auth_config),
    )?;

    let search_args = json!({"query": "latte", "serverUrl": server.trim_end_matches('/')});
    let search = runtime.call("searchDrinks", search_args.clone())?;
    let mut drink_component = mount_for_outcome(
        skill_path,
        runtime.skill(),
        "searchDrinks",
        search_args,
        search.result.clone(),
        required_component_path(&search, "searchDrinks")?,
    )?;
    let drink_event = find_tap_event(&drink_component.mount.render.root, "confirmDrink")
        .ok_or_else(|| CliError::Demo("drink-list confirmDrink event not found".to_owned()))?;
    let drink_action = dispatch_first_api_call(&mut drink_component.instance, &drink_event)?;

    let confirm_args = api_call_args(&drink_action, "confirmOrder")?;
    let confirm = runtime.call("confirmOrder", confirm_args.clone())?;
    let mut order_component = mount_for_outcome(
        skill_path,
        runtime.skill(),
        "confirmOrder",
        confirm_args,
        confirm.result.clone(),
        required_component_path(&confirm, "confirmOrder")?,
    )?;
    let pay_event = find_tap_event(&order_component.mount.render.root, "payOrder")
        .ok_or_else(|| CliError::Demo("order-confirm payOrder event not found".to_owned()))?;
    let pay_action = dispatch_first_api_call(&mut order_component.instance, &pay_event)?;

    let pay_args = api_call_args(&pay_action, "payOrder")?;
    let payment = runtime.call("payOrder", pay_args.clone())?;
    let mut payment_component = mount_for_outcome(
        skill_path,
        runtime.skill(),
        "payOrder",
        pay_args,
        payment.result.clone(),
        required_component_path(&payment, "payOrder")?,
    )?;
    let expire = payment_component
        .instance
        .expire(json!({"reason": "payment_completed"}))?;

    let server_health = DemoHttpClient::new(server).get_json("/health", None)?;
    Ok(json!({
        "status": "ok",
        "server": {
            "baseUrl": server.trim_end_matches('/'),
            "health": server_health,
            "auth": {
                "merchantDid": auth.merchant_did,
                "userDid": auth_config.user_did,
                "agentDid": auth_config.agent_did,
                "capabilityToken": "[REDACTED]",
                "tokenReceived": auth.token_received,
                "authProvider": "did-challenge",
                "challengeVerified": auth.token_received,
                "tokenScopes": [
                    "coffee:drinks:read",
                    "coffee:order:confirm",
                    "coffee:order:pay",
                    "coffee:order:read"
                ],
                "wxLoginStatus": "container-managed",
                "requestAuthMode": "container-attached-bearer",
                "credential": auth_config.redacted_summary()
            },
            "business": server_business
        },
        "flow": [
            step_summary("searchDrinks", &search.result, &drink_component.mount.render.root, &drink_component.mount.actions),
            step_summary("confirmOrder", &confirm.result, &order_component.mount.render.root, &order_component.mount.actions),
            step_summary("payOrder", &payment.result, &payment_component.mount.render.root, &payment_component.mount.actions),
            json!({
                "name": "expire",
                "state": expire.state,
                "actions": expire.actions,
                "trace": expire.trace
            })
        ],
        "componentActions": {
            "drinkList": drink_action,
            "orderConfirm": pay_action
        },
        "audit": audit_events_json(&runtime.audit_events())
    }))
}

struct RuntimeHarness {
    service: RuntimeService<
        AllowHost,
        ApproveConsent,
        js_runtime_quickjs::QuickJsApiExecutor,
        ComponentRenderRouter,
        CollectAudit,
        CollectAudit,
    >,
    audit: CollectAudit,
    identity: RuntimeIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeIdentity {
    user_did: String,
    agent_did: Option<String>,
    merchant_did: String,
}

impl RuntimeIdentity {
    fn default_demo() -> Self {
        Self {
            user_did: DEFAULT_USER_DID.to_owned(),
            agent_did: Some(DEFAULT_AGENT_DID.to_owned()),
            merchant_did: DEFAULT_MERCHANT_DID.to_owned(),
        }
    }
}

impl RuntimeHarness {
    fn load(
        skill_path: &Path,
        identity: RuntimeIdentity,
        auth_config: Option<&DemoAuthConfig>,
    ) -> Result<Self, CliError> {
        let skill = load_skill(skill_path)?;
        let api_vm = ApiVm::load_skill(skill.clone())?;
        let mut executor = api_vm.executor();
        if let Some(auth_config) = auth_config {
            executor = executor.with_host_did_auth(
                HostDidAuthConfig::new(
                    auth_config.credential.did_document_path.clone(),
                    auth_config.credential.private_key_path.clone(),
                )
                .without_private_key_permission_check(),
            );
        }
        let audit = CollectAudit::default();
        let service = RuntimeService::load_skill(
            skill.clone(),
            AllowHost,
            ApproveConsent,
            executor,
            ComponentRenderRouter {
                skill_root: skill.root,
                manifest: skill.manifest,
            },
            audit.clone(),
            audit.clone(),
        );
        Ok(Self {
            service,
            audit,
            identity,
        })
    }

    fn call(
        &self,
        api_name: impl Into<String>,
        arguments: Value,
    ) -> Result<dock_core::CallOutcome, CliError> {
        let response = self
            .service
            .call_api(RuntimeCallRequest {
                session: RuntimeSessionContext {
                    user_did: Some(self.identity.user_did.clone()),
                    agent_did: self.identity.agent_did.clone(),
                    merchant_did: Some(self.identity.merchant_did.clone()),
                    skill_id: DEFAULT_SKILL_ID.to_owned(),
                    session_id: DEFAULT_SESSION_ID.to_owned(),
                },
                api_name: api_name.into(),
                arguments,
                capability_token: None,
            })
            .map_err(CliError::Runtime)?;
        Ok(response.data.into_call_outcome())
    }

    fn skill(&self) -> &LoadedSkill {
        self.service.skill()
    }

    fn audit_events(&self) -> Vec<AuditEvent> {
        self.audit.events.borrow().clone()
    }

    fn handle_ipc_request(&self, request: RuntimeIpcRequest) -> dock_core::RuntimeIpcResponse {
        self.service.handle_ipc_request(request)
    }
}

#[derive(Clone)]
struct AllowHost;

impl RuntimeHost for AllowHost {
    fn check_permission(
        &self,
        _context: &ApiCallContext,
    ) -> Result<PermissionDecision, DockCoreError> {
        Ok(PermissionDecision::Allow)
    }
}

#[derive(Clone)]
struct ApproveConsent;

impl ConsentGate for ApproveConsent {
    fn check_consent(
        &self,
        _context: &ApiCallContext,
        _request: &ConsentRequest,
    ) -> Result<ConsentDecision, DockCoreError> {
        Ok(ConsentDecision::approved(
            DEV_HEADLESS_CONSENT_PROVIDER,
            DEV_HEADLESS_DECISION_ACTOR,
        ))
    }
}

#[derive(Clone)]
struct ComponentRenderRouter {
    skill_root: PathBuf,
    manifest: mcp_schema::SkillManifest,
}

impl RenderRouter for ComponentRenderRouter {
    fn render(
        &self,
        _context: &ApiCallContext,
        input: &ComponentRenderInput,
    ) -> Result<RenderOutcome, DockCoreError> {
        let component_input = ComponentInput {
            api_name: input.api_name.clone(),
            arguments: input.arguments.clone(),
            properties: Map::new(),
            content: input
                .content
                .iter()
                .map(|content| serde_json::to_value(content).unwrap_or(Value::Null))
                .collect(),
            structured_content: input.structured_content.clone(),
            meta: input.meta.clone(),
            component_metadata: ComponentMetadata::default(),
        };
        let package = load_component_package(&self.skill_root, &input.component_path)
            .map_err(|error| DockCoreError::core(ErrorCode::RenderFailed, error.to_string()))?;
        let mut instance = ComponentInstance::new(package)
            .map_err(|error| DockCoreError::core(ErrorCode::RenderFailed, error.to_string()))?;
        let metadata = manifest_component_metadata(&self.manifest, &input.component_path).map_err(
            |error| DockCoreError::core(ErrorCode::RenderFailed, redact_text(&error.to_string())),
        )?;
        let component_input = ComponentInput {
            component_metadata: metadata,
            ..component_input
        };
        let outcome = instance
            .mount(component_input)
            .map_err(|error| DockCoreError::core(ErrorCode::RenderFailed, error.to_string()))?;

        Ok(RenderOutcome {
            renderer: "component-runtime".to_owned(),
            component_path: Some(input.component_path.clone()),
            payload: json!({
                "render": component_render_json(&outcome.render),
                "actions": outcome.actions,
                "metadata": outcome.metadata,
                "trace": outcome.trace,
                "state": outcome.state
            }),
            fallback_reason: None,
        })
    }

    fn fallback(
        &self,
        _context: &ApiCallContext,
        result: &AtomicApiResult,
        reason: &str,
    ) -> RenderOutcome {
        let fallback_reason = fallback_reason_from_str(reason);
        let stable_reason = fallback_reason.as_str().to_owned();
        RenderOutcome {
            renderer: "card-spec".to_owned(),
            component_path: None,
            payload: json!(fallback_from_result(result, fallback_reason)),
            fallback_reason: Some(stable_reason),
        }
    }
}

#[derive(Clone, Default)]
struct CollectAudit {
    events: std::rc::Rc<RefCell<Vec<AuditEvent>>>,
}

impl AuditSink for CollectAudit {
    fn record(&self, event: AuditEvent) -> Result<(), DockCoreError> {
        self.events.borrow_mut().push(event);
        Ok(())
    }
}

impl RuntimeAuditReader for CollectAudit {
    fn runtime_audit_records(&self) -> Vec<AuditEvent> {
        self.events.borrow().clone()
    }
}

struct MountedComponent {
    instance: ComponentInstance,
    mount: component_runtime::ComponentOperationOutcome,
}

fn mount_for_outcome(
    skill_path: &Path,
    skill: &LoadedSkill,
    api_name: &str,
    arguments: Value,
    result: AtomicApiResult,
    component_path: &str,
) -> Result<MountedComponent, CliError> {
    let package = load_component_package(skill_path, component_path)?;
    let mut instance = ComponentInstance::new(package)?;
    let metadata = manifest_component_metadata(&skill.manifest, component_path)?;
    let input = ComponentInput {
        component_metadata: metadata,
        ..component_input(api_name, arguments, &result)
    };
    let mount = instance.mount(input)?;
    Ok(MountedComponent { instance, mount })
}

fn component_input(api_name: &str, arguments: Value, result: &AtomicApiResult) -> ComponentInput {
    ComponentInput {
        api_name: api_name.to_owned(),
        arguments,
        properties: Map::new(),
        content: result
            .content
            .iter()
            .map(|content| serde_json::to_value(content).unwrap_or(Value::Null))
            .collect(),
        structured_content: result.structured_content.clone(),
        meta: result.meta.clone(),
        component_metadata: ComponentMetadata::default(),
    }
}

fn load_component_metadata(
    skill_path: &Path,
    component_path: &str,
) -> Result<ComponentMetadata, CliError> {
    let skill = load_skill(skill_path)?;
    manifest_component_metadata(&skill.manifest, component_path)
}

fn manifest_component_metadata(
    manifest: &mcp_schema::SkillManifest,
    component_path: &str,
) -> Result<ComponentMetadata, CliError> {
    let component = manifest
        .components
        .iter()
        .find(|component| component_paths_match(&component.path, component_path))
        .ok_or_else(|| CliError::Demo(format!("component `{component_path}` is not declared")))?;
    let mut metadata = ComponentMetadata::new(component.path.clone());
    metadata.related_page = component.related_page.as_ref().and_then(safe_related_page);
    metadata.dynamic = component.dynamic_permission().is_some();
    metadata.scope_dynamic = component.dynamic_permission().map(redact_metadata_value);
    metadata.expirable = component.expirable.unwrap_or(false);
    metadata.expired_text = component.expired_text.as_deref().map(redact_metadata_text);
    Ok(metadata)
}

fn safe_related_page(related_page: &Value) -> Option<Value> {
    let object = related_page.as_object()?;
    let path = object.get("path")?.as_str()?.trim();
    if path.is_empty() || path.starts_with('/') || path.contains("..") || path.contains('\0') {
        return None;
    }

    let mut safe = Map::new();
    safe.insert("path".to_owned(), Value::String(path.to_owned()));
    if let Some(query) = object.get("query").and_then(Value::as_object) {
        safe.insert(
            "query".to_owned(),
            Value::Object(
                query
                    .iter()
                    .map(|(key, value)| {
                        if is_sensitive_metadata_key(key) {
                            (key.clone(), Value::String("[REDACTED]".to_owned()))
                        } else {
                            (key.clone(), redact_metadata_value(value))
                        }
                    })
                    .collect(),
            ),
        );
    }
    Some(Value::Object(safe))
}

fn component_paths_match(declared: &str, requested: &str) -> bool {
    declared == requested || strip_index_suffix(declared) == strip_index_suffix(requested)
}

fn strip_index_suffix(path: &str) -> &str {
    path.strip_suffix("/index").unwrap_or(path)
}

fn redact_metadata_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| {
                    if is_sensitive_metadata_key(key) {
                        (key.clone(), Value::String("[REDACTED]".to_owned()))
                    } else {
                        (key.clone(), redact_metadata_value(value))
                    }
                })
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(redact_metadata_value).collect()),
        Value::String(text) => Value::String(redact_metadata_text(text)),
        _ => value.clone(),
    }
}

fn redact_metadata_text(text: &str) -> String {
    if looks_sensitive_metadata_text(text) {
        "[REDACTED]".to_owned()
    } else {
        text.to_owned()
    }
}

fn is_sensitive_metadata_key(key: &str) -> bool {
    let normalized = key.to_ascii_lowercase();
    [
        "token",
        "authorization",
        "signature",
        "secret",
        "private",
        "credential",
        "password",
    ]
    .iter()
    .any(|needle| normalized.contains(needle))
}

fn looks_sensitive_metadata_text(text: &str) -> bool {
    text.starts_with('/')
        || text.starts_with("\\\\")
        || text.contains(":\\")
        || text.starts_with("file:")
}

fn load_component_package(
    skill_path: &Path,
    component_path: &str,
) -> Result<ComponentPackage, CliError> {
    ComponentPackage::load(component_directory(skill_path, component_path)?).map_err(Into::into)
}

fn component_directory(skill_path: &Path, component_path: &str) -> Result<PathBuf, CliError> {
    resolve_component_path(skill_path, component_path).map_err(Into::into)
}

fn find_tap_event(root: &RenderNode, method: &str) -> Option<ComponentEvent> {
    let binding = root
        .events
        .iter()
        .find(|event| event.event == RenderEventKind::Tap && event.method.as_str() == method);
    if let Some(binding) = binding {
        return Some(ComponentEvent::from_binding(binding));
    }
    root.children
        .iter()
        .find_map(|child| find_tap_event(child, method))
}

fn dispatch_first_api_call(
    instance: &mut ComponentInstance,
    event: &ComponentEvent,
) -> Result<ComponentVmAction, CliError> {
    let outcome = instance.dispatch_event(event)?;
    outcome
        .actions
        .into_iter()
        .find(|action| matches!(action, ComponentVmAction::ApiCall { .. }))
        .ok_or_else(|| CliError::Demo("component event did not emit api/call".to_owned()))
}

fn api_call_args(action: &ComponentVmAction, expected_name: &str) -> Result<Value, CliError> {
    match action {
        ComponentVmAction::ApiCall { name, arguments } if name == expected_name => {
            Ok(arguments.clone())
        }
        ComponentVmAction::ApiCall { name, .. } => Err(CliError::Demo(format!(
            "expected api/call `{expected_name}`, got `{name}`"
        ))),
        _ => Err(CliError::Demo("expected api/call action".to_owned())),
    }
}

fn required_component_path<'a>(
    outcome: &'a dock_core::CallOutcome,
    api_name: &str,
) -> Result<&'a str, CliError> {
    outcome
        .render
        .as_ref()
        .and_then(|render| render.component_path.as_deref())
        .ok_or_else(|| CliError::Demo(format!("API `{api_name}` did not render a component")))
}

fn step_summary(
    name: &str,
    result: &AtomicApiResult,
    root: &RenderNode,
    actions: &[ComponentVmAction],
) -> Value {
    json!({
        "name": name,
        "content": result.content,
        "structuredContent": result.structured_content,
        "renderRootKind": root.kind,
        "renderRootId": root.id,
        "actions": actions
    })
}

#[derive(Debug)]
struct DemoAuth {
    merchant_did: String,
    capability_token: String,
    token_received: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DemoDidChallenge {
    challenge_id: String,
    merchant_did: String,
    nonce: String,
    issued_at_ms: u64,
    expires_at_ms: Option<u64>,
    audience: String,
}

struct DemoHttpClient {
    base_url: String,
}

impl DemoHttpClient {
    fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_owned(),
        }
    }

    fn login(&self, auth_config: &DemoAuthConfig) -> Result<DemoAuth, CliError> {
        let challenge: DemoDidChallenge = serde_json::from_value(self.post_json(
            "/agents/coffee/auth/challenge",
            None,
            json!({
                "sessionId": DEFAULT_SESSION_ID,
                "skillId": DEFAULT_SKILL_ID,
                "userDid": auth_config.user_did,
                "agentDid": auth_config.agent_did
            }),
        )?)
        .map_err(|source| CliError::Json {
            label: "auth challenge response".to_owned(),
            source,
        })?;
        let session = IdentitySession::new(
            auth_config.user_did.clone(),
            auth_config.agent_did.clone(),
            challenge.merchant_did.clone(),
            DEFAULT_SKILL_ID,
            DEFAULT_SESSION_ID,
        );
        let payload = ChallengeProofPayload::from_challenge(
            &AdapterDidChallenge {
                challenge_id: challenge.challenge_id.clone(),
                merchant_did: challenge.merchant_did.clone(),
                nonce: challenge.nonce.clone(),
                expires_at_ms: challenge.expires_at_ms,
            },
            &session,
            challenge.audience.clone(),
            challenge.issued_at_ms,
        );
        let provider = FileDidCredentialProvider::from_config(auth_config.credential.clone())?;
        let proof = sign_challenge_proof(&payload, &provider, &session, AuthMode::HttpSignatures)?;
        let login_request = ChallengeLoginRequest {
            session_id: DEFAULT_SESSION_ID.to_owned(),
            skill_id: DEFAULT_SKILL_ID.to_owned(),
            user_did: auth_config.user_did.clone(),
            agent_did: auth_config.agent_did.clone(),
            merchant_did: challenge.merchant_did.clone(),
            challenge_id: challenge.challenge_id,
            signed_challenge: serde_json::to_value(proof).map_err(|source| CliError::Json {
                label: "signedChallenge".to_owned(),
                source,
            })?,
        };
        let login: ChallengeLoginResponse = serde_json::from_value(self.post_json(
            "/agents/coffee/auth/login",
            None,
            serde_json::to_value(login_request).map_err(|source| CliError::Json {
                label: "auth login request".to_owned(),
                source,
            })?,
        )?)
        .map_err(|source| CliError::Json {
            label: "auth login response".to_owned(),
            source,
        })?;
        Ok(DemoAuth {
            merchant_did: challenge.merchant_did,
            capability_token: login.capability_token.clone(),
            token_received: !login.capability_token.is_empty(),
        })
    }

    fn coffee_business_check(&self, token: &str) -> Result<Value, CliError> {
        let drinks = self.get_json("/api/drinks?query=latte", Some(token))?;
        let order = self.post_json(
            "/api/order/confirm",
            Some(token),
            json!({
                "drinkId": "latte",
                "size": "medium",
                "sugar": "less"
            }),
        )?;
        let order_id = order["orderId"]
            .as_str()
            .ok_or_else(|| CliError::Http("confirm order response missing orderId".to_owned()))?;
        let paid = self.post_json(
            "/api/order/pay",
            Some(token),
            json!({
                "orderId": order_id
            }),
        )?;

        Ok(json!({
            "firstDrinkId": drinks["drinks"].as_array().and_then(|items| items.first()).and_then(|item| item.get("id")).cloned().unwrap_or(Value::Null),
            "orderId": order["orderId"],
            "payable": order["payable"],
            "paymentStatus": paid["status"]
        }))
    }

    fn get_json(&self, path: &str, bearer: Option<&str>) -> Result<Value, CliError> {
        self.request_json("GET", path, bearer, None)
    }

    fn post_json(&self, path: &str, bearer: Option<&str>, body: Value) -> Result<Value, CliError> {
        self.request_json("POST", path, bearer, Some(body))
    }

    fn request_json(
        &self,
        method: &str,
        path: &str,
        bearer: Option<&str>,
        body: Option<Value>,
    ) -> Result<Value, CliError> {
        let (status, body) = http_request(&self.base_url, method, path, bearer, body)?;
        if !(200..300).contains(&status) {
            return Err(CliError::Http(format!(
                "{method} {path} returned {status}: {}",
                redact_text(&body)
            )));
        }
        serde_json::from_str(&body).map_err(|source| CliError::Json {
            label: format!("{method} {path} response"),
            source,
        })
    }
}

fn http_request(
    base_url: &str,
    method: &str,
    path: &str,
    bearer: Option<&str>,
    body: Option<Value>,
) -> Result<(u16, String), CliError> {
    let parsed = ParsedHttpUrl::parse(base_url)?;
    let body = body.map(|value| value.to_string()).unwrap_or_default();
    let mut stream = TcpStream::connect((parsed.host.as_str(), parsed.port))?;
    stream.set_read_timeout(Some(Duration::from_secs(5)))?;
    stream.set_write_timeout(Some(Duration::from_secs(5)))?;

    let full_path = if parsed.path_prefix == "/" {
        path.to_owned()
    } else {
        format!("{}{}", parsed.path_prefix.trim_end_matches('/'), path)
    };
    let auth = bearer
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let request = format!(
        "{method} {full_path} HTTP/1.1\r\nHost: {}\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth}\r\n{body}",
        parsed.host_header(),
        body.len()
    );
    stream.write_all(request.as_bytes())?;
    let response = read_http_response(&mut stream)?;
    let (head, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| CliError::Http("HTTP response missing header separator".to_owned()))?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|status| status.parse::<u16>().ok())
        .ok_or_else(|| CliError::Http("HTTP response missing status code".to_owned()))?;
    Ok((status, body.to_owned()))
}

fn read_http_response(stream: &mut TcpStream) -> Result<String, CliError> {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut buffer)?;
        if read == 0 {
            return Err(CliError::Http(
                "connection closed before response headers".to_owned(),
            ));
        }
        bytes.extend_from_slice(&buffer[..read]);
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            break index;
        }
    };
    let headers = String::from_utf8_lossy(&bytes[..header_end]).to_string();
    let body_start = header_end + 4;
    let content_length = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok())
            .flatten()
    });
    if let Some(content_length) = content_length {
        while bytes.len().saturating_sub(body_start) < content_length {
            let read = stream.read(&mut buffer)?;
            if read == 0 {
                return Err(CliError::Http(
                    "connection closed before full response body".to_owned(),
                ));
            }
            bytes.extend_from_slice(&buffer[..read]);
        }
    } else {
        loop {
            let read = stream.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            bytes.extend_from_slice(&buffer[..read]);
        }
    }
    String::from_utf8(bytes).map_err(|error| CliError::Http(error.to_string()))
}

#[derive(Debug)]
struct ParsedHttpUrl {
    host: String,
    port: u16,
    path_prefix: String,
}

impl ParsedHttpUrl {
    fn parse(url: &str) -> Result<Self, CliError> {
        let rest = url.strip_prefix("http://").ok_or_else(|| {
            CliError::Http("only http:// demo server URLs are supported".to_owned())
        })?;
        let (authority, path_prefix) = rest
            .split_once('/')
            .map(|(authority, path)| (authority, format!("/{path}")))
            .unwrap_or((rest, "/".to_owned()));
        let (host, port) = authority
            .rsplit_once(':')
            .map(|(host, port)| {
                let port = port
                    .parse::<u16>()
                    .map_err(|error| CliError::Http(format!("invalid server port: {error}")))?;
                Ok::<_, CliError>((host.to_owned(), port))
            })
            .transpose()?
            .unwrap_or_else(|| (authority.to_owned(), 80));
        if host.is_empty() {
            return Err(CliError::Http("server URL missing host".to_owned()));
        }
        Ok(Self {
            host,
            port,
            path_prefix,
        })
    }

    fn host_header(&self) -> String {
        if self.port == 80 {
            self.host.clone()
        } else {
            format!("{}:{}", self.host, self.port)
        }
    }
}

fn parse_json(source: &str, label: &str) -> Result<Value, CliError> {
    serde_json::from_str(source).map_err(|source| CliError::Json {
        label: label.to_owned(),
        source,
    })
}

fn parse_atomic_result(source: &str) -> Result<AtomicApiResult, CliError> {
    serde_json::from_str(source).map_err(|source| CliError::Json {
        label: "resultJson".to_owned(),
        source,
    })
}

fn parse_component_input(source: &str) -> Result<ComponentInput, CliError> {
    let value = parse_json(source, "jsonInput")?;
    match serde_json::from_value::<ComponentInput>(value.clone()) {
        Ok(input) => Ok(input),
        Err(_) => Ok(ComponentInput {
            api_name: value
                .get("apiName")
                .and_then(Value::as_str)
                .unwrap_or("preview")
                .to_owned(),
            arguments: value
                .get("arguments")
                .cloned()
                .unwrap_or_else(|| Value::Object(Map::new())),
            properties: value
                .get("properties")
                .and_then(Value::as_object)
                .cloned()
                .unwrap_or_default(),
            content: component_content_from_json(&value),
            structured_content: value
                .get("structuredContent")
                .or_else(|| value.get("structured_content"))
                .and_then(Value::as_object)
                .cloned(),
            meta: value
                .get("_meta")
                .or_else(|| value.get("meta"))
                .and_then(Value::as_object)
                .cloned(),
            component_metadata: ComponentMetadata::default(),
        }),
    }
}

fn component_content_from_json(value: &Value) -> Vec<Value> {
    value
        .get("content")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
}

fn write_json(writer: &mut impl Write, output: &impl Serialize) -> Result<(), CliError> {
    serde_json::to_writer_pretty(&mut *writer, output).map_err(|source| CliError::Json {
        label: "output".to_owned(),
        source,
    })?;
    writeln!(writer)?;
    Ok(())
}

fn validation_summary(report: &ValidationReport) -> Value {
    json!({
        "valid": report.is_valid(),
        "errors": report.errors,
        "warnings": report.warnings
    })
}

fn skill_id(skill: &LoadedSkill) -> String {
    skill
        .manifest
        .extra
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_SKILL_ID)
        .to_owned()
}

fn fallback_reason_from_str(reason: &str) -> FallbackReason {
    FallbackReason::normalize(reason)
}

fn render_outcome_json(render: Option<&RenderOutcome>) -> Value {
    let Some(render) = render else {
        return Value::Null;
    };
    json!({
        "renderer": render.renderer,
        "componentPath": render.component_path,
        "payload": render.payload,
        "fallbackReason": render.fallback_reason
    })
}

fn audit_events_json(events: &[AuditEvent]) -> Value {
    Value::Array(
        events
            .iter()
            .map(|event| {
                json!({
                    "userDid": event.user_did,
                    "agentDid": event.agent_did,
                    "merchantDid": event.merchant_did,
                    "sessionId": event.session_id,
                    "skillId": event.skill_id,
                    "apiName": event.api_name,
                    "riskLevel": event.risk_level,
                    "parameterSummary": event.parameter_summary,
                    "permissionDecision": event.permission_decision,
                    "consentProof": event.consent_proof,
                    "outcome": event.outcome
                })
            })
            .collect(),
    )
}

fn component_render_json(render: &ComponentRenderOutput) -> Value {
    json!({
        "schemaVersion": render.schema_version,
        "root": render.root,
        "warnings": render.warnings
    })
}

fn redact_text(value: &str) -> String {
    for marker in [
        "capabilityToken",
        "Authorization",
        "Signature",
        "token",
        "secret",
        "private",
    ] {
        if value
            .to_ascii_lowercase()
            .contains(&marker.to_ascii_lowercase())
        {
            return format!("{marker}=[REDACTED]");
        }
    }
    value.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn parses_validate_args() {
        let cli = Cli::try_parse_from_args(["dock-cli", "validate", "examples/coffee-skill"])
            .expect("args parse");
        assert!(matches!(cli.command, Command::Validate { .. }));
    }

    #[test]
    fn parses_runtime_json_args() {
        let cli = Cli::try_parse_from_args([
            "dock-cli",
            "runtime-json",
            "examples/coffee-skill",
            r#"{"apiVersion":"dock.runtime.v1","requestId":"req-1","method":"runtime.loadSkill","params":{}}"#,
        ])
        .expect("args parse");
        assert!(matches!(cli.command, Command::RuntimeJson { .. }));
    }

    #[test]
    fn preview_card_renders_fallback_card() {
        let output = preview_card(
            r#"{"content":[{"type":"text","text":"hello"}],"structuredContent":{"orderId":"1"}}"#,
        )
        .expect("preview card");

        assert_eq!(output["status"], "ok");
        assert_eq!(output["card"]["version"], "card-spec/v0");
        assert_eq!(
            output["card"]["fallbackReason"],
            "host_renderer_unavailable"
        );
    }

    #[test]
    fn redacts_http_errors() {
        let redacted = redact_text(r#"{"capabilityToken":"demo-token"}"#);
        assert_eq!(redacted, "capabilityToken=[REDACTED]");
    }

    #[test]
    fn validate_reports_api_registration_mismatch_as_release_blocker() {
        let fixture = SkillFixture::new();
        let output = validate(&fixture.root).expect("validate reports compatibility");

        assert_eq!(output["status"], "ok");
        assert_eq!(output["compatibilityLevel"], "demo-only");
        assert!(output["compatibilityReport"]["apis"]
            .as_array()
            .expect("api reports")
            .iter()
            .any(|api| {
                api["name"] == "missing"
                    && api["registered"] == false
                    && api["status"] == "registration-unverified"
            }));
        assert!(output["compatibilityReport"]["releaseBlockers"]
            .as_array()
            .expect("release blockers")
            .iter()
            .any(|blocker| {
                blocker["code"] == "api_registration_mismatch"
                    && blocker["message"]
                        .as_str()
                        .is_some_and(|message| message.contains("declared but not registered"))
            }));
        assert_eq!(
            output["compatibilityReport"]["supplyChain"]["status"],
            "demo-unsigned"
        );
        assert_eq!(
            output["compatibilityReport"]["supplyChain"]["productionReady"],
            false
        );
        assert!(output["compatibilityReport"]["releaseBlockers"]
            .as_array()
            .expect("release blockers")
            .iter()
            .any(
                |blocker| blocker["code"] == "supply_chain" && blocker["status"] == "demo-unsigned"
            ));
        assert!(!output.to_string().contains("fixture-signature-secret"));
    }

    #[test]
    fn validate_reports_component_runtime_metadata() {
        let dir = TempDir::new("dock-cli-metadata-fixture").expect("temp dir");
        let root = dir.path().to_path_buf();
        fs::create_dir_all(root.join("components/status-card")).expect("component dir");
        fs::write(root.join("SKILL.md"), "# Metadata Skill").expect("write SKILL.md");
        fs::write(
            root.join("index.js"),
            "const skill = wx.modelContext.createSkill(__dirname)\n\
             skill.registerAPI('status', async () => ({ content: [{ type: 'text', text: 'ok' }] }))\n\
             module.exports = skill\n",
        )
        .expect("write index.js");
        fs::write(
            root.join("components/status-card/index.wxml"),
            "<view><text>{{ apiName }}</text></view>",
        )
        .expect("write wxml");
        fs::write(
            root.join("mcp.json"),
            r#"{
              "apis": [{
                "name": "status",
                "description": "status API",
                "inputSchema": {},
                "_meta": { "ui": { "componentPath": "components/status-card/index" } }
              }],
              "components": [{
                "path": "components/status-card/index",
                "permissions": {
                  "scope.dynamic": { "desc": "refresh status" }
                },
                "relatedPage": {
                  "path": "pages/status/detail",
                  "query": {
                    "source": "card",
                    "secretToken": "should-not-leak"
                  }
                },
                "expirable": true,
                "expiredText": "Status expired"
              }]
            }"#,
        )
        .expect("write manifest");

        let output = validate(&root).expect("validate metadata");
        let component = output["compatibilityReport"]["components"][0].clone();

        assert_eq!(component["loaded"], true);
        assert_eq!(
            component["runtimeMetadata"]["componentPath"],
            "components/status-card/index"
        );
        assert_eq!(component["runtimeMetadata"]["dynamic"], true);
        assert_eq!(component["runtimeMetadata"]["expirable"], true);
        assert_eq!(
            component["runtimeMetadata"]["expiredText"],
            "Status expired"
        );
        assert_eq!(
            component["runtimeMetadata"]["relatedPage"]["path"],
            "pages/status/detail"
        );
        assert_eq!(
            component["runtimeMetadata"]["relatedPage"]["query"]["secretToken"],
            "[REDACTED]"
        );
        assert!(!component.to_string().contains("should-not-leak"));
    }

    #[test]
    fn validate_redacts_package_signature_value() {
        let fixture = SignedSkillFixture::new();
        let output = validate(&fixture.root).expect("validate signed fixture");

        assert_eq!(
            output["compatibilityReport"]["supplyChain"]["signature"]["keyId"],
            "did:wba:publisher.example#package-key-1"
        );
        assert_eq!(
            output["compatibilityReport"]["supplyChain"]["status"],
            "quarantined"
        );
        assert!(!output.to_string().contains("fixture-signature-secret"));
    }

    #[test]
    fn component_metadata_matches_index_path_alias() {
        let manifest: mcp_schema::SkillManifest = serde_json::from_value(json!({
            "apis": [],
            "components": [{
                "path": "components/status-card/index",
                "expirable": true
            }]
        }))
        .expect("manifest");

        let metadata =
            manifest_component_metadata(&manifest, "components/status-card").expect("metadata");

        assert_eq!(
            metadata.component_path.as_deref(),
            Some("components/status-card/index")
        );
        assert!(metadata.expirable);
    }

    #[test]
    fn finds_nested_tap_event() {
        let mut binding =
            component_runtime::RenderEventBinding::new(RenderEventKind::Tap, "confirmDrink");
        binding.dataset.insert("id".to_owned(), json!("latte"));
        let root = RenderNode::new("root", component_runtime::RenderNodeKind::View).with_child(
            RenderNode::new("button", component_runtime::RenderNodeKind::Button)
                .with_event(binding),
        );

        let event = find_tap_event(&root, "confirmDrink").expect("event");

        assert_eq!(event.kind, component_runtime::ComponentEventKind::Tap);
        assert_eq!(event.dataset["id"], "latte");
    }

    #[test]
    fn parses_run_demo_explicit_credential_config() {
        let fixture = CredentialFixture::new();
        let cli = Cli::try_parse_from_args([
            "dock-cli".to_owned(),
            "run-demo".to_owned(),
            "--skill".to_owned(),
            "examples/coffee-skill".to_owned(),
            "--server".to_owned(),
            "http://127.0.0.1:3000".to_owned(),
            "--did-document".to_owned(),
            fixture.did_path.display().to_string(),
            "--private-key".to_owned(),
            fixture.key_path.display().to_string(),
            "--user-did".to_owned(),
            "did:wba:user.example".to_owned(),
        ])
        .expect("CLI args parse");

        let Command::RunDemo {
            did_document,
            private_key,
            user_did,
            ..
        } = cli.command
        else {
            panic!("expected run-demo");
        };
        let config = DemoAuthConfig::from_inputs(
            did_document,
            private_key,
            user_did,
            None,
            None,
            None,
            EmptyConfigSource,
        )
        .expect("credential config parses");

        assert_eq!(config.user_did, "did:wba:user.example");
        assert_eq!(config.credential.did_document_path, fixture.did_path);
    }

    #[test]
    fn derives_user_did_from_did_document_for_path_credentials() {
        let fixture = CredentialFixture::new();
        let config = DemoAuthConfig::from_inputs(
            Some(fixture.did_path.clone()),
            Some(fixture.key_path),
            None,
            Some("did:wba:agent.example".to_owned()),
            None,
            None,
            EmptyConfigSource,
        )
        .expect("credential config parses");

        assert_eq!(config.user_did, "did:wba:user.example");
        assert_eq!(config.agent_did.as_deref(), Some("did:wba:agent.example"));
        assert_eq!(config.credential.did_document_path, fixture.did_path);
    }

    #[test]
    fn incomplete_run_demo_credential_config_fails_closed() {
        let fixture = CredentialFixture::new();

        let error = DemoAuthConfig::from_inputs(
            Some(fixture.did_path),
            None,
            Some("did:wba:user.example".to_owned()),
            None,
            None,
            None,
            EmptyConfigSource,
        )
        .expect_err("missing private key must fail");

        assert_eq!(error, DidCredentialError::InvalidIdentity);
    }

    #[test]
    fn resolves_identity_handle_from_store_without_exposing_key_content() {
        let fixture = IdentityStoreFixture::new("miniapp-test.awiki.ai");
        let config = DemoAuthConfig::from_inputs(
            None,
            None,
            None,
            None,
            Some("miniapp-test.awiki.ai".to_owned()),
            Some(fixture.root.clone()),
            EmptyConfigSource,
        )
        .expect("identity handle resolves");

        assert_eq!(config.user_did, "did:wba:miniapp-test.example");
        assert_eq!(config.credential.private_key_path, fixture.key_path);
        let summary = config.redacted_summary().to_string();
        assert!(!summary.contains("test-only-key"));
        assert!(!summary.contains("key-1-private.pem"));
        assert!(summary.contains("[REDACTED]"));
    }

    #[test]
    fn resolves_default_project_identity_from_examples_identity() {
        let fixture = ProjectIdentityFixture::new();
        let config = DemoAuthConfig::from_default_project_identity(&fixture.root, None)
            .expect("default project identity resolves");

        assert_eq!(config.user_did, "did:wba:default-user.example");
        assert_eq!(
            config.credential.did_document_path,
            fixture
                .root
                .join(DEFAULT_IDENTITY_DIR)
                .join(DEFAULT_DID_DOCUMENT_FILE)
        );
        assert_eq!(
            config.credential.private_key_path,
            fixture
                .root
                .join(DEFAULT_IDENTITY_DIR)
                .join(DEFAULT_PRIVATE_KEY_FILE)
        );
    }

    #[test]
    fn finds_project_root_from_child_directory() {
        let fixture = ProjectIdentityFixture::new();
        let child = fixture.root.join("crates/dock-cli/src");
        fs::create_dir_all(&child).expect("create child dir");

        assert_eq!(
            find_project_root_from(&child).as_deref(),
            Some(fixture.root.as_path())
        );
    }

    #[derive(Clone, Copy)]
    struct EmptyConfigSource;

    impl ConfigSource for EmptyConfigSource {
        fn string(self, _name: &str) -> Option<String> {
            None
        }
    }

    struct CredentialFixture {
        _dir: TempDir,
        did_path: PathBuf,
        key_path: PathBuf,
    }

    impl CredentialFixture {
        fn new() -> Self {
            let dir = TempDir::new("dock-cli-credential").expect("temp dir");
            let did_path = dir.path().join("did_document.json");
            let key_path = dir.path().join("key-1-private.pem");
            fs::write(&did_path, br#"{"id":"did:wba:user.example"}"#).expect("write DID");
            fs::write(&key_path, "test-only-key").expect("write key");
            set_private_key_permissions(&key_path);
            Self {
                _dir: dir,
                did_path,
                key_path,
            }
        }
    }

    struct SkillFixture {
        _dir: TempDir,
        root: PathBuf,
    }

    impl SkillFixture {
        fn new() -> Self {
            let dir = TempDir::new("dock-cli-skill-fixture").expect("temp dir");
            let root = dir.path().to_path_buf();
            fs::write(root.join("SKILL.md"), "# Test Skill").expect("write SKILL.md");
            fs::write(
                root.join("index.js"),
                "const skill = wx.modelContext.createSkill(__dirname)\n\
                 skill.registerAPI('declared', async () => ({ content: [{ type: 'text', text: 'ok' }] }))\n\
                 module.exports = skill\n",
            )
            .expect("write index.js");
            fs::write(
                root.join("mcp.json"),
                r#"{
                  "apis": [
                    {
                      "name": "declared",
                      "description": "registered API",
                      "inputSchema": {}
                    },
                    {
                      "name": "missing",
                      "description": "declared but not registered API",
                      "inputSchema": {}
                    }
                  ],
                  "components": []
                }"#,
            )
            .expect("write mcp.json");

            Self { _dir: dir, root }
        }
    }

    struct SignedSkillFixture {
        _dir: TempDir,
        root: PathBuf,
    }

    impl SignedSkillFixture {
        fn new() -> Self {
            let dir = TempDir::new("dock-cli-signed-skill-fixture").expect("temp dir");
            let root = dir.path().to_path_buf();
            fs::write(root.join("SKILL.md"), "# Signed Test Skill").expect("write SKILL.md");
            fs::write(
                root.join("index.js"),
                "const skill = wx.modelContext.createSkill(__dirname)\n\
                 skill.registerAPI('declared', async () => ({ content: [{ type: 'text', text: 'ok' }] }))\n\
                 module.exports = skill\n",
            )
            .expect("write index.js");
            fs::write(
                root.join("mcp.json"),
                r#"{
                  "_meta": {
                    "anp": {
                      "supplyChain": {
                        "publisherDid": "did:wba:publisher.example",
                        "digest": {
                          "algorithm": "sha256",
                          "value": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
                        },
                        "signature": {
                          "algorithm": "dock.package.dev-sha256.v1",
                          "keyId": "did:wba:publisher.example#package-key-1",
                          "value": "fixture-signature-secret"
                        }
                      }
                    }
                  },
                  "apis": [
                    {
                      "name": "declared",
                      "description": "registered API",
                      "inputSchema": {}
                    }
                  ],
                  "components": []
                }"#,
            )
            .expect("write mcp.json");

            Self { _dir: dir, root }
        }
    }

    struct IdentityStoreFixture {
        _dir: TempDir,
        root: PathBuf,
        key_path: PathBuf,
    }

    impl IdentityStoreFixture {
        fn new(handle: &str) -> Self {
            let dir = TempDir::new("dock-cli-identity-store").expect("temp dir");
            let identities = dir.path().join("identities");
            let identity_dir = identities.join("e1_test");
            fs::create_dir_all(&identity_dir).expect("create identity dir");
            fs::write(
                identity_dir.join("identity.json"),
                format!(r#"{{"handle":"{handle}","did":"did:wba:miniapp-test.example"}}"#),
            )
            .expect("write identity");
            fs::write(
                identity_dir.join("did_document.json"),
                br#"{"id":"did:wba:miniapp-test.example"}"#,
            )
            .expect("write DID document");
            let key_path = identity_dir.join("key-1-private.pem");
            fs::write(&key_path, "test-only-key").expect("write key");
            set_private_key_permissions(&key_path);
            Self {
                root: dir.path().to_path_buf(),
                key_path,
                _dir: dir,
            }
        }
    }

    struct ProjectIdentityFixture {
        _dir: TempDir,
        root: PathBuf,
    }

    impl ProjectIdentityFixture {
        fn new() -> Self {
            let dir = TempDir::new("dock-cli-project-identity").expect("temp dir");
            let root = dir.path().join("repo");
            let identity = root.join(DEFAULT_IDENTITY_DIR);
            fs::create_dir_all(&identity).expect("create default identity dir");
            fs::create_dir_all(root.join("examples/coffee-skill")).expect("create skill dir");
            fs::write(root.join("Cargo.toml"), b"[workspace]\n").expect("write manifest");
            fs::write(
                identity.join(DEFAULT_DID_DOCUMENT_FILE),
                br#"{"id":"did:wba:default-user.example"}"#,
            )
            .expect("write DID document");
            let key_path = identity.join(DEFAULT_PRIVATE_KEY_FILE);
            fs::write(&key_path, "test-only-key").expect("write key");
            set_private_key_permissions(&key_path);
            Self { _dir: dir, root }
        }
    }

    #[cfg(unix)]
    fn set_private_key_permissions(path: &Path) {
        use std::os::unix::fs::PermissionsExt;

        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("set key permissions");
    }

    #[cfg(not(unix))]
    fn set_private_key_permissions(_path: &Path) {}

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(prefix: &str) -> std::io::Result<Self> {
            let path = std::env::temp_dir().join(format!(
                "{}-{}-{}",
                prefix,
                std::process::id(),
                unique_suffix()
            ));
            fs::create_dir(&path)?;
            Ok(Self { path })
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn unique_suffix() -> String {
        use std::sync::atomic::{AtomicU64, Ordering};

        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let counter = COUNTER.fetch_add(1, Ordering::Relaxed);
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        format!("{nanos}-{counter}")
    }
}
