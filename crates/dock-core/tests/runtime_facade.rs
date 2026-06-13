use consent_audit::{ConsentRequest, DEV_HEADLESS_CONSENT_PROVIDER, DEV_HEADLESS_DECISION_ACTOR};
use dock_core::{
    negotiate_runtime_version, ApiCallContext, ApiExecutor, AuditEvent, AuditSink,
    ComponentRenderInput, ConsentDecision, ConsentGate, DockCoreError, ErrorCode,
    PermissionDecision, RenderOutcome, RenderRouter, RuntimeAuditReader,
    RuntimeAuditRecordsRequest, RuntimeCallRequest, RuntimeCloseSessionRequest,
    RuntimeComponentAction, RuntimeDispatchComponentActionRequest, RuntimeExpireCardsRequest,
    RuntimeHost, RuntimePersistentAuditSink, RuntimeRenderComponentRequest, RuntimeService,
    RuntimeSessionContext, RUNTIME_API_VERSION,
};
use mcp_schema::{AtomicApiResult, TextContent, ValidationReport};
use serde_json::{json, Map};
use skill_loader::load_skill;
use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("crate lives under crates/dock-core")
        .to_path_buf()
}

fn coffee_skill_root() -> PathBuf {
    repo_root().join("examples/coffee-skill")
}

fn session() -> RuntimeSessionContext {
    RuntimeSessionContext {
        user_did: Some("did:wba:user.example".to_owned()),
        agent_did: Some("did:wba:agent.example".to_owned()),
        merchant_did: Some("did:wba:merchant.example".to_owned()),
        skill_id: "coffee".to_owned(),
        session_id: "session-runtime".to_owned(),
    }
}

fn runtime_service(
    executor: MockExecutor,
) -> RuntimeService<AllowHost, ApproveConsent, MockExecutor, MockRenderer, MockAudit, MockAudit> {
    let skill = load_skill(coffee_skill_root()).expect("coffee skill loads");
    let audit = MockAudit::default();
    RuntimeService::load_skill(
        skill,
        AllowHost,
        ApproveConsent,
        executor,
        MockRenderer,
        audit.clone(),
        audit,
    )
}

#[test]
fn runtime_version_negotiation_reports_stable_version() {
    let version = negotiate_runtime_version(Some(RUNTIME_API_VERSION))
        .expect("current version is accepted")
        .data;

    assert_eq!(version.current, RUNTIME_API_VERSION);
    assert_eq!(version.supported, vec![RUNTIME_API_VERSION]);

    let error =
        negotiate_runtime_version(Some("dock.runtime.v0")).expect_err("unsupported version fails");
    assert_eq!(error.error.code, "unsupported_version");
    assert_eq!(error.version, RUNTIME_API_VERSION);
}

#[test]
fn runtime_facade_call_render_action_expire_audit_and_close_are_versioned() {
    let executor = MockExecutor::with_result(AtomicApiResult {
        is_error: false,
        content: vec![TextContent::text("confirmed")],
        structured_content: Some(Map::from_iter([
            ("orderId".to_owned(), json!("order_demo_001")),
            ("payable".to_owned(), json!(18)),
        ])),
        meta: Some(Map::from_iter([(
            "private".to_owned(),
            json!("component-only"),
        )])),
        extra: Default::default(),
    });
    let calls = executor.calls.clone();
    let service = runtime_service(executor);

    let validate = service.validate_skill();
    assert_eq!(validate.version, RUNTIME_API_VERSION);
    assert_eq!(validate.data.skill.skill_id, "unknown");
    assert!(validate
        .data
        .skill
        .api_names
        .iter()
        .any(|api| api == "confirmOrder"));

    let loaded = service.load_skill_response();
    assert_eq!(loaded.version, RUNTIME_API_VERSION);
    assert!(loaded.data.skill.component_paths.iter().any(|path| {
        path == "components/order-confirm/index" || path == "components/order-confirm"
    }));

    let call = service
        .call_api(RuntimeCallRequest {
            session: session(),
            api_name: "confirmOrder".to_owned(),
            arguments: json!({"drinkId": "latte"}),
            capability_token: Some("capability-secret-token".to_owned()),
        })
        .expect("facade call succeeds");
    assert_eq!(call.version, RUNTIME_API_VERSION);
    assert_eq!(call.data.api_name, "confirmOrder");
    assert_eq!(call.data.result.content[0].text, "confirmed");
    let model_visible_json =
        serde_json::to_value(&call.data.model_visible).expect("model-visible serializes");
    assert!(model_visible_json.get("_meta").is_none());
    assert_eq!(
        call.data
            .render
            .as_ref()
            .and_then(|render| render.component_path.as_deref()),
        Some("components/order-confirm/index")
    );

    let render = service
        .render_component(RuntimeRenderComponentRequest {
            session: session(),
            api_name: "confirmOrder".to_owned(),
            arguments: json!({"drinkId": "latte"}),
            content: vec![TextContent::text("confirmed")],
            structured_content: Some(Map::from_iter([("ok".to_owned(), json!(true))])),
            meta: None,
            component_path: "components/order-confirm/index".to_owned(),
        })
        .expect("explicit render succeeds");
    assert_eq!(render.data.render.renderer, "mock-renderer");

    let action = service
        .dispatch_component_action(RuntimeDispatchComponentActionRequest {
            session: session(),
            source_api_name: "searchDrinks".to_owned(),
            source_arguments: json!({"query": "latte"}),
            action: RuntimeComponentAction::ApiCall {
                name: "confirmOrder".to_owned(),
                arguments: json!({"drinkId": "latte"}),
            },
            capability_token: None,
        })
        .expect("component api/call routes through facade");
    assert!(action.data.handled);
    assert_eq!(
        action.data.call.as_ref().map(|call| call.api_name.as_str()),
        Some("confirmOrder")
    );
    assert_eq!(*calls.borrow(), 2);

    let expire = service
        .expire_cards(RuntimeExpireCardsRequest {
            session: session(),
            filters: json!({"componentPath": "components/order-confirm/index"}),
        })
        .expect("expire boundary returns stable response");
    assert!(expire.data.accepted);
    assert_eq!(expire.data.boundary, "host-managed-card-store");

    let audit = service
        .get_audit_records(RuntimeAuditRecordsRequest {
            session_id: Some("session-runtime".to_owned()),
            skill_id: Some("coffee".to_owned()),
        })
        .expect("audit reader returns records");
    assert_eq!(audit.version, RUNTIME_API_VERSION);
    assert_eq!(audit.data.records.len(), 2);
    assert!(audit
        .data
        .records
        .iter()
        .all(|record| record.parameter_summary["drinkId"] == "latte"));

    let close = service
        .close_session(RuntimeCloseSessionRequest { session: session() })
        .expect("close response is stable");
    assert!(close.data.closed);
    assert_eq!(close.data.boundary, "stateless-runtime-facade");
}

#[test]
fn runtime_persistent_audit_sink_records_and_reads_redacted_events() {
    let fixture = TempDir::new("dock-core-runtime-audit");
    let audit = RuntimePersistentAuditSink::new(consent_audit::FileAuditSink::new(
        fixture.path().join("audit").join("records.jsonl"),
    ));
    let skill = load_skill(coffee_skill_root()).expect("coffee skill loads");
    let executor = MockExecutor::with_result(AtomicApiResult {
        is_error: false,
        content: vec![TextContent::text("paid")],
        structured_content: Some(Map::from_iter([
            ("orderId".to_owned(), json!("order_demo_001")),
            ("status".to_owned(), json!("paid")),
        ])),
        meta: None,
        extra: Default::default(),
    });
    let service = RuntimeService::load_skill(
        skill,
        AllowHost,
        ApproveConsent,
        executor,
        MockRenderer,
        audit.clone(),
        audit,
    );

    service
        .call_api(RuntimeCallRequest {
            session: session(),
            api_name: "payOrder".to_owned(),
            arguments: json!({
                "orderId": "order_demo_001",
                "capabilityToken": "capability-secret-token",
                "deliveryAddress": "1 Private Road"
            }),
            capability_token: Some("capability-secret-token".to_owned()),
        })
        .expect("payment call succeeds with persistent audit");

    let audit = service
        .get_audit_records(RuntimeAuditRecordsRequest {
            session_id: Some("session-runtime".to_owned()),
            skill_id: Some("coffee".to_owned()),
        })
        .expect("persistent audit reader returns records");
    assert_eq!(audit.data.records.len(), 1);
    let record = &audit.data.records[0];
    assert_eq!(record.api_name, "payOrder");
    assert_eq!(record.outcome, "ok");
    assert_eq!(record.permission_decision.decision, "allow");
    assert_eq!(record.parameter_summary["capabilityToken"], "[REDACTED]");
    assert_eq!(record.parameter_summary["deliveryAddress"], "[REDACTED]");

    let rendered = serde_json::to_string(&audit).expect("audit serializes");
    assert!(!rendered.contains("capability-secret-token"));
    assert!(!rendered.contains("1 Private Road"));
}

#[test]
fn runtime_persistent_audit_reader_reports_unavailable_when_backend_is_corrupt() {
    let fixture = TempDir::new("dock-core-runtime-audit-corrupt");
    let audit_path = fixture.path().join("audit").join("records.jsonl");
    fs::create_dir_all(audit_path.parent().expect("audit dir")).expect("create audit parent dir");
    fs::write(&audit_path, "{not-json}\n").expect("write corrupt audit file");
    let audit = RuntimePersistentAuditSink::new(consent_audit::FileAuditSink::new(audit_path));
    let skill = load_skill(coffee_skill_root()).expect("coffee skill loads");
    let service = RuntimeService::load_skill(
        skill,
        AllowHost,
        ApproveConsent,
        MockExecutor::with_result(AtomicApiResult {
            is_error: false,
            content: vec![TextContent::text("ok")],
            structured_content: None,
            meta: None,
            extra: Default::default(),
        }),
        MockRenderer,
        MockAudit::default(),
        audit,
    );

    let error = service
        .get_audit_records(RuntimeAuditRecordsRequest {
            session_id: None,
            skill_id: None,
        })
        .expect_err("corrupt persistent audit backend should surface an error");

    assert_eq!(error.error.code, "audit_unavailable");
    assert!(!error.error.message.contains("{not-json}"));
}

#[test]
fn runtime_facade_errors_are_json_serializable_and_redacted() {
    let service = runtime_service(MockExecutor::fail(
        ErrorCode::VmFailed,
        "Authorization Bearer capability-secret-token",
    ));

    let error = service
        .call_api(RuntimeCallRequest {
            session: session(),
            api_name: "searchDrinks".to_owned(),
            arguments: json!({"query": "latte"}),
            capability_token: Some("capability-secret-token".to_owned()),
        })
        .expect_err("executor failure is wrapped as runtime error");

    let json = serde_json::to_value(&error).expect("runtime error serializes");
    assert_eq!(json["version"], RUNTIME_API_VERSION);
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"]["code"], "vm_failed");
    let rendered = json.to_string();
    assert!(!rendered.contains("capability-secret-token"));
    assert!(!rendered.contains("Authorization Bearer"));
    assert!(rendered.contains("[REDACTED]"));
}

#[test]
fn runtime_facade_validation_errors_redact_reports() {
    let mut report = ValidationReport::ok();
    report.push_error(
        "/home/user/private/key-1-private.pem",
        "Authorization Bearer capability-secret-token",
    );
    report.push_compatibility_warning(
        "credential.path",
        "file:/Users/demo/private-key.pem",
        Some("remove secret token"),
    );

    let error = dock_core::RuntimeErrorResponse::from_core(DockCoreError::validation(
        "private validation error",
        report,
    ));

    let rendered = serde_json::to_string(&error).expect("runtime error serializes");
    assert_eq!(error.error.code, "validation_failed");
    assert!(!rendered.contains("/home/user/private/key-1-private.pem"));
    assert!(!rendered.contains("capability-secret-token"));
    assert!(!rendered.contains("/Users/demo/private-key.pem"));
    assert!(!rendered.contains("secret token"));
    assert!(rendered.contains("[REDACTED]"));
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
struct MockExecutor {
    result: AtomicApiResult,
    error: Option<(ErrorCode, String)>,
    calls: std::rc::Rc<RefCell<usize>>,
}

impl MockExecutor {
    fn with_result(result: AtomicApiResult) -> Self {
        Self {
            result,
            error: None,
            calls: Default::default(),
        }
    }

    fn fail(code: ErrorCode, message: &str) -> Self {
        Self {
            result: AtomicApiResult {
                is_error: false,
                content: vec![TextContent::text("unused")],
                structured_content: None,
                meta: None,
                extra: Default::default(),
            },
            error: Some((code, message.to_owned())),
            calls: Default::default(),
        }
    }
}

impl ApiExecutor for MockExecutor {
    fn execute(
        &self,
        _context: &ApiCallContext,
        _component_path: Option<&str>,
    ) -> Result<AtomicApiResult, DockCoreError> {
        *self.calls.borrow_mut() += 1;
        if let Some((code, message)) = &self.error {
            return Err(DockCoreError::core(*code, message.clone()));
        }
        Ok(self.result.clone())
    }
}

#[derive(Clone)]
struct MockRenderer;

impl RenderRouter for MockRenderer {
    fn render(
        &self,
        _context: &ApiCallContext,
        input: &ComponentRenderInput,
    ) -> Result<RenderOutcome, DockCoreError> {
        Ok(RenderOutcome {
            renderer: "mock-renderer".to_owned(),
            component_path: Some(input.component_path.clone()),
            payload: json!({
                "apiName": input.api_name,
                "structuredContent": input.structured_content,
                "meta": input.meta
            }),
            fallback_reason: None,
        })
    }

    fn fallback(
        &self,
        _context: &ApiCallContext,
        _result: &AtomicApiResult,
        reason: &str,
    ) -> RenderOutcome {
        RenderOutcome {
            renderer: "mock-fallback".to_owned(),
            component_path: None,
            payload: json!({}),
            fallback_reason: Some(reason.to_owned()),
        }
    }
}

#[derive(Clone, Default)]
struct MockAudit {
    events: std::rc::Rc<RefCell<Vec<AuditEvent>>>,
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(prefix: &str) -> Self {
        let path = std::env::temp_dir().join(format!("{prefix}-{}", unique_suffix()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir(&path).expect("create temp dir");
        Self { path }
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
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time after epoch")
        .as_nanos();
    format!("{}-{nanos}", std::process::id())
}

impl AuditSink for MockAudit {
    fn record(&self, event: AuditEvent) -> Result<(), DockCoreError> {
        self.events.borrow_mut().push(event);
        Ok(())
    }
}

impl RuntimeAuditReader for MockAudit {
    fn runtime_audit_records(&self) -> Result<Vec<AuditEvent>, dock_core::DockCoreError> {
        Ok(self.events.borrow().clone())
    }
}
