use js_runtime_quickjs::{ApiCall, ApiVm, ApiVmConfig, ApiVmError};
use mcp_schema::{ApiDeclaration, ComponentDeclaration, SkillManifest, ValidationReport};
use serde_json::json;
use skill_loader::{LoadedComponent, LoadedSkill, SourceFile};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[test]
fn middleware_runs_in_onion_order() {
    let skill = test_skill(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.use(async (ctx, next) => {
  ctx.arguments.events.push('outer-before')
  await next()
  ctx.arguments.events.push('outer-after')
})
skill.use(async (ctx, next) => {
  ctx.arguments.events.push('inner-before')
  await next()
  ctx.arguments.events.push('inner-after')
})
skill.registerAPI('ordered', async (ctx) => {
  ctx.arguments.events.push('handler')
  return {
    content: [{ type: 'text', text: ctx.arguments.events.join(',') }],
    structuredContent: { events: ctx.arguments.events }
  }
})
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["ordered"],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new(
            "skill",
            "session",
            "ordered",
            json!({ "events": [] }),
        ))
        .expect("call ordered");

    assert_eq!(
        result
            .structured_content
            .as_ref()
            .and_then(|content| content.get("events"))
            .and_then(|events| events.as_array())
            .cloned(),
        Some(vec![
            json!("outer-before"),
            json!("inner-before"),
            json!("handler"),
            json!("inner-after"),
            json!("outer-after"),
        ])
    );
}

#[test]
fn async_handler_promise_is_resolved() {
    let skill = test_skill(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('asyncValue', async (ctx) => {
  const suffix = await Promise.resolve(ctx.arguments.suffix)
  return { content: [{ type: 'text', text: 'async-' + suffix }] }
})
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["asyncValue"],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new(
            "skill",
            "session",
            "asyncValue",
            json!({ "suffix": "ok" }),
        ))
        .expect("call asyncValue");

    assert_eq!(result.content[0].text, "async-ok");
}

#[test]
fn wx_login_is_available_to_atomic_api_code() {
    let skill = test_skill(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('login', async () => {
  const login = await wx.login()
  return {
    content: [{ type: 'text', text: login.errMsg }],
    structuredContent: { code: login.code }
  }
})
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["login"],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new("skill", "session", "login", json!({})))
        .expect("call login");

    assert_eq!(result.content[0].text, "login:ok");
    assert_eq!(
        result
            .structured_content
            .as_ref()
            .and_then(|content| content.get("code"))
            .and_then(|code| code.as_str()),
        Some("dock-login-code-localhost")
    );
}

#[test]
fn model_context_get_session_id_is_available_to_atomic_api_code() {
    let skill = test_skill(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('session', () => {
  const sessionId = wx.modelContext.getSessionId()
  return {
    content: [{ type: 'text', text: sessionId }],
    structuredContent: { sessionId }
  }
})
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["session"],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new("skill", "session-123", "session", json!({})))
        .expect("call session");

    assert_eq!(result.content[0].text, "session-123");
    assert_eq!(
        result
            .structured_content
            .as_ref()
            .and_then(|content| content.get("sessionId"))
            .and_then(|session_id| session_id.as_str()),
        Some("session-123")
    );
}

#[test]
fn model_context_expire_all_cards_records_private_meta_event() {
    let skill = test_skill_with_components(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('expire', async () => {
  const outcome = await wx.modelContext.expireAllCards({
    componentPaths: ['components/result/index'],
    match: 'latest'
  })
  return {
    content: [{ type: 'text', text: outcome.errMsg }],
    structuredContent: { expiredCount: outcome.expiredCount }
  }
})
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["expire"],
        vec![("components/result/index", true)],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new("skill", "session", "expire", json!({})))
        .expect("call expire");

    assert_eq!(
        result
            .structured_content
            .as_ref()
            .and_then(|content| content.get("expiredCount"))
            .and_then(|expired_count| expired_count.as_u64()),
        Some(1)
    );
    assert_eq!(
        result
            .meta
            .as_ref()
            .and_then(|meta| meta.get("modelContext"))
            .and_then(|model_context| model_context.get("cardEvents"))
            .and_then(|events| events.as_array())
            .and_then(|events| events.first())
            .and_then(|event| event.get("type"))
            .and_then(|event_type| event_type.as_str()),
        Some("expireAllCards")
    );
    assert_eq!(
        result
            .meta
            .as_ref()
            .and_then(|meta| meta.get("modelContext"))
            .and_then(|model_context| model_context.get("cardEvents"))
            .and_then(|events| events.as_array())
            .and_then(|events| events.first())
            .and_then(|event| event.get("matchPolicy"))
            .and_then(|match_policy| match_policy.as_str()),
        Some("latest")
    );
    let model_visible = serde_json::to_value(result.model_visible()).expect("model visible");
    assert!(model_visible.get("_meta").is_none());
}

#[test]
fn model_context_expire_all_cards_defaults_to_all_expirable_components() {
    let skill = test_skill_with_components(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('expire', async () => {
  const outcome = await wx.modelContext.expireAllCards({ match: 'all' })
  return {
    content: [{ type: 'text', text: outcome.errMsg }],
    structuredContent: { expiredCount: outcome.expiredCount }
  }
})
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["expire"],
        vec![
            ("components/result/index", true),
            ("components/receipt/index", true),
            ("components/static/index", false),
        ],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new("skill", "session", "expire", json!({})))
        .expect("call expire");

    assert_eq!(
        result
            .structured_content
            .as_ref()
            .and_then(|content| content.get("expiredCount"))
            .and_then(|expired_count| expired_count.as_u64()),
        Some(2)
    );
    assert_eq!(
        result
            .meta
            .as_ref()
            .and_then(|meta| meta.get("modelContext"))
            .and_then(|model_context| model_context.get("cardEvents"))
            .and_then(|events| events.as_array())
            .and_then(|events| events.first())
            .and_then(|event| event.get("componentPaths"))
            .and_then(|paths| paths.as_array())
            .cloned(),
        Some(vec![
            json!("components/receipt/index"),
            json!("components/result/index"),
        ])
    );
    assert_eq!(
        result
            .meta
            .as_ref()
            .and_then(|meta| meta.get("modelContext"))
            .and_then(|model_context| model_context.get("cardEvents"))
            .and_then(|events| events.as_array())
            .and_then(|events| events.first())
            .and_then(|event| event.get("matchPolicy"))
            .and_then(|match_policy| match_policy.as_str()),
        Some("all")
    );
}

#[test]
fn model_context_expire_all_cards_rejects_invalid_component_path() {
    let skill = test_skill_with_components(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('expire', async () => {
  try {
    await wx.modelContext.expireAllCards({ componentPaths: ['../outside'] })
  } catch (error) {
    return {
      content: [{ type: 'text', text: error.errMsg }],
      structuredContent: { code: error.code }
    }
  }
  return { content: [{ type: 'text', text: 'unexpected' }] }
})
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["expire"],
        vec![("components/result/index", true)],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new("skill", "session", "expire", json!({})))
        .expect("call expire");

    assert!(result.content[0]
        .text
        .contains("modelContext.expireAllCards:fail"));
    assert_eq!(
        result
            .structured_content
            .as_ref()
            .and_then(|content| content.get("code"))
            .and_then(|code| code.as_str()),
        Some("invalid_options")
    );
    assert!(result.meta.is_none());
}

#[test]
fn model_context_expire_all_cards_requires_expirable_component() {
    let skill = test_skill_with_components(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('expire', async () => {
  try {
    await wx.modelContext.expireAllCards({ componentPaths: ['components/result/index'] })
  } catch (error) {
    return { content: [{ type: 'text', text: error.reason }] }
  }
  return { content: [{ type: 'text', text: 'unexpected' }] }
})
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["expire"],
        vec![("components/result/index", false)],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new("skill", "session", "expire", json!({})))
        .expect("call expire");

    assert!(result.content[0]
        .text
        .contains("component without expirable: true"));
    assert!(result.meta.is_none());
}

#[test]
fn model_context_notification_type_matches_component_runtime() {
    let skill = test_skill(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('notificationTypes', () => ({
  content: [{ type: 'text', text: [
    wx.modelContext.NotificationType.Input,
    wx.modelContext.NotificationType.Result,
    wx.modelContext.NotificationType.Expire,
    wx.modelContext.NotificationType.Overflow
  ].join(',') }]
}))
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["notificationTypes"],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new(
            "skill",
            "session",
            "notificationTypes",
            json!({}),
        ))
        .expect("call notificationTypes");

    assert_eq!(result.content[0].text, "input,result,expire,overflow");
}

#[test]
fn create_skill_rejects_path_outside_skill_package() {
    let skill = test_skill(
        r#"
const skill = wx.modelContext.createSkill('../outside')
skill.registerAPI('escape', () => ({ content: [{ type: 'text', text: 'never' }] }))
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["escape"],
    );

    let error = ApiVm::load_skill(skill).expect_err("unsafe skillPath must fail during load");
    assert!(
        matches!(error, ApiVmError::QuickJs(message) if message.contains("createSkill path outside skill package"))
    );
}

#[test]
fn timeout_interrupts_long_running_handler() {
    let skill = test_skill(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('loop', () => {
  while (true) {}
})
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["loop"],
    );
    let vm = ApiVm::load_skill_with_config(
        skill,
        ApiVmConfig {
            timeout: Duration::from_millis(20),
            ..Default::default()
        },
    )
    .expect("load VM");

    let error = vm
        .call(ApiCall::new("skill", "session", "loop", json!({})))
        .expect_err("loop should time out");

    assert!(matches!(error, ApiVmError::Timeout(name, _) if name == "loop"));
}

#[test]
fn require_parent_escape_is_rejected() {
    let mut modules = BTreeMap::new();
    modules.insert(
        "safe".to_owned(),
        r#"
module.exports = () => ({ content: [{ type: 'text', text: 'never' }] })
"#
        .to_owned(),
    );
    let skill = test_skill(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('escape', require('../secret'))
module.exports = skill
"#,
        modules,
        vec!["escape"],
    );

    let error = ApiVm::load_skill(skill).expect_err("escape require must fail");
    assert!(
        matches!(error, ApiVmError::QuickJs(message) if message.contains("outside skill package"))
    );
}

#[test]
fn sandbox_globals_are_not_available_to_skill_code() {
    let skill = test_skill(
        r#"
const skill = wx.modelContext.createSkill(__dirname)
skill.registerAPI('globals', () => ({
  content: [{ type: 'text', text: [
    typeof process,
    typeof fetch,
    typeof eval,
    typeof Function,
    typeof (() => {}).constructor,
    typeof (async function() {}).constructor,
    typeof (function* () {}).constructor,
    typeof (async function* () {}).constructor
  ].join(',') }]
}))
module.exports = skill
"#,
        BTreeMap::new(),
        vec!["globals"],
    );
    let vm = ApiVm::load_skill(skill).expect("load VM");

    let result = vm
        .call(ApiCall::new("skill", "session", "globals", json!({})))
        .expect("call globals");

    assert_eq!(
        result.content[0].text,
        "undefined,undefined,undefined,undefined,undefined,undefined,undefined,undefined"
    );
}

fn test_skill(
    entry_js: &str,
    api_modules: BTreeMap<String, String>,
    api_names: Vec<&str>,
) -> LoadedSkill {
    test_skill_with_components(entry_js, api_modules, api_names, Vec::new())
}

fn test_skill_with_components(
    entry_js: &str,
    api_modules: BTreeMap<String, String>,
    api_names: Vec<&str>,
    component_specs: Vec<(&str, bool)>,
) -> LoadedSkill {
    let root = test_root(&component_specs);
    let components = component_specs
        .iter()
        .map(|(path, expirable)| ComponentDeclaration {
            path: (*path).to_owned(),
            permissions: None,
            related_page: None,
            expirable: Some(*expirable),
            expired_text: None,
            meta: None,
            extra: BTreeMap::new(),
        })
        .collect::<Vec<_>>();
    let loaded_components = component_specs
        .iter()
        .map(|(path, _)| {
            (
                (*path).to_owned(),
                LoadedComponent {
                    route: (*path).to_owned(),
                    directory: component_directory(&root, path),
                    index_js: None,
                    index_wxml: None,
                    index_wxss: None,
                    index_json: None,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();

    LoadedSkill {
        root: root.clone(),
        skill_md: source_at(&root, "SKILL.md", "Test skill"),
        manifest: SkillManifest {
            apis: api_names
                .into_iter()
                .map(|name| ApiDeclaration {
                    name: name.to_owned(),
                    description: format!("{name} test API"),
                    input_schema: json!({ "type": "object" }),
                    output_schema: None,
                    meta: None,
                    extra: BTreeMap::new(),
                })
                .collect(),
            components,
            extra: BTreeMap::new(),
        },
        entry_js: source_at(&root, "index.js", entry_js),
        api_modules: api_modules
            .into_iter()
            .map(|(name, body)| {
                (
                    name.clone(),
                    source_at(&root, format!("apis/{name}.js"), body),
                )
            })
            .collect(),
        components: loaded_components,
        component_routes: BTreeMap::new(),
        validation: ValidationReport::ok(),
    }
}

fn test_root(component_specs: &[(&str, bool)]) -> PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!(
        "anp-miniapp-dock-js-runtime-tests-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let _ = fs::remove_dir_all(&root);
    let mut required_dirs = component_specs
        .iter()
        .map(|(path, _)| component_directory(&root, path))
        .collect::<BTreeSet<_>>();
    required_dirs.insert(root.clone());
    for dir in required_dirs {
        fs::create_dir_all(dir).expect("create test component dir");
    }
    root
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

fn component_directory(root: &Path, component_path: &str) -> PathBuf {
    let relative = Path::new(component_path);
    let dir = if relative.file_name().is_some_and(|name| name == "index") {
        relative.parent().unwrap_or(relative)
    } else {
        relative
    };
    root.join(dir)
}

fn source_at(root: &Path, path: impl AsRef<Path>, source: impl Into<String>) -> SourceFile {
    let relative_path = path.as_ref().to_path_buf();
    SourceFile {
        absolute_path: root.join(&relative_path),
        relative_path,
        source: source.into(),
    }
}
