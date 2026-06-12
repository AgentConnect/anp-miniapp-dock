# Release Gates Runbook

> 状态：Phase 0 发布门槛初版
> 日期：2026-06-12
> 范围：定义 `anp-miniapp-dock` 每次进入 production-readiness milestone、release branch 或 production deployment 前需要执行或明确记录的验证、Review、红线和回滚条件。
> 上游计划：[`../plan/production-readiness-roadmap.md`](../plan/production-readiness-roadmap.md) Step 00-04。

## 1. 使用规则

每个 Step 仍按 roadmap 的小 Plan 执行；本 runbook 是跨 Step、跨 Phase 的 release gate 汇总。执行 release gates 时必须记录：

- 执行日期、分支和 commit range；
- 每个命令的 pass/fail/skip；
- skip 原因、影响和替代证据；
- Review 发现、修复或残余风险；
- 最终 `git status --short --branch`。

任何 gate 失败不得靠“文档说明”绕过。只有当 gate 标为 planned 且当前 Phase 尚未实现时，才能记录为 planned gap。

## 2. 基础命令 Gate

这些命令来自仓库 `AGENTS.md` 和 README，是当前 release 前基础 gate。

```bash
cargo metadata --format-version 1 --no-deps
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p dock-cli --test coffee_order_flow
cargo run -p dock-cli -- validate examples/coffee-skill
```

通过标准：

- `cargo metadata` 成功，workspace 成员与文档一致。
- `cargo fmt --check` 无格式 diff。
- `cargo clippy` 无 warning。
- `cargo test --workspace` 全部通过。
- coffee CLI E2E 通过，并继续断言 capability token、Authorization、Signature、Signature-Input、private key path/material 不出现在 JSON 输出。
- `dock-cli validate` 输出 JSON，包含 `compatibilityLevel` 和 `compatibilityReport.apis/components/permissions/risks/fallbacks/releaseBlockers`；当前 coffee Skill 仍应因 demo-only localhost DID/request metadata 被标为 `demo-only`，不得误标 `supported`。

如果环境无法运行全量命令，必须记录原因、失败命令、影响范围和替代检查；不能把未运行命令写成通过。

## 3. 文档 Gate

当前文档 gate：

```bash
git diff --check -- README.md AGENTS.md docs/architecture docs/runbook docs/security docs/plan
```

手工检查：

| 检查项 | 当前要求 | planned 提升 |
|---|---|---|
| Markdown 链接 | 新增或修改文档的相对链接必须指向存在文件 | Phase 5 引入自动 link checker |
| 兼容矩阵状态 | `supported`、`host-boundary`、`planned-p1`、`planned-p2`、`demo-only`、`unsupported-by-design` 不得混用或写成未知 | Phase 5 引入矩阵 schema checker |
| Step 台账 | 每个 Step 有 status、Review evidence、verification evidence、commit hash | Codex Goal 长跑每步必填 |
| Validate 报告 | `dock-cli validate examples/coffee-skill` 的 `compatibilityReport` 包含 API 注册、组件加载、权限、风险、fallback 和 release blocker 字段 | Phase 5 引入报告 schema checker |
| Plan 变更 | 改范围、顺序、验收、安全边界或验证策略前先更新 Plan 变更记录 | Phase 6 可纳入 PR checklist |
| README 索引 | 新增架构、安全、runbook 文档必须有入口链接 | 当前手工检查 |

## 4. 安全 Gate

### 4.1 当前必须执行

当前已有自动化测试覆盖以下安全主线，随 `cargo test --workspace` 执行：

| Gate | 证据 |
|---|---|
| Skill package path escape / absolute path deny | [`coffee_skill_load.rs`](../../crates/skill-loader/tests/coffee_skill_load.rs) |
| Manifest component metadata、input `format:image/file`、production warning 分层 | [`mcp_validation.rs`](../../crates/mcp-schema/tests/mcp_validation.rs) |
| `dock-cli validate` 兼容报告、API 注册 mismatch blocker、demo-only release blocker | [`commands.rs`](../../crates/dock-cli/src/commands.rs)、[`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) |
| Atomic API sandbox、unsafe require、timeout | [`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs)、[`bridge.rs`](../../crates/js-runtime-quickjs/src/bridge.rs) |
| Component sandbox、default no network/timer、expire 后事件失败 | [`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs) |
| Component profile 默认 deny request/timer，dynamic 才可表达 request boundary | [`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs) |
| Request allowlist deny by default / miss deny without transport | [`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs) |
| token scope isolation、HTTP Signature fallback、401 retry | [`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs) |
| DID auth session cache 隔离、过期 refresh、clear/revoke 语义 | [`session.rs`](../../crates/anp-adapter/src/session.rs) |
| Atomic API `wx.login` receipt、`wx.checkSession`、JS auth header fail closed、response header redaction | [`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) |
| deterministic unsupported API stub、sync throw、nested `wx.cloud.*`、unknown root fallback、safe reason/suggestion | [`unsupported.rs`](../../crates/wx-compat/src/unsupported.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) |
| Atomic API scoped storage JS bridge、sync/async shape、DID/merchant/skill scope、JSON-safe validation、model-visible 隔离 | [`storage.rs`](../../crates/wx-compat/src/storage.rs)、[`scoped_storage.rs`](../../crates/wx-compat/tests/scoped_storage.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) |
| Atomic API 与 Component VM device/app info 最小字段、防指纹和 shared default 防漂移 | [`model_context.rs`](../../crates/wx-compat/src/model_context.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs) |
| Atomic API 高风险 Host boundary：无 provider fail closed、未 consent 不执行 provider、dev-only mock 标识、opaque handle、本地路径拒绝、payment 不收集密码 | [`high_risk.rs`](../../crates/wx-compat/src/high_risk.rs)、[`high_risk_provider.rs`](../../crates/wx-compat/tests/high_risk_provider.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) |
| L3 payment consent 和 audit redaction | [`payment_requires_consent.rs`](../../crates/consent-audit/tests/payment_requires_consent.rs)、[`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs) |
| CLI/demo redaction | [`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) |

补充红线抽样：

```bash
rg -n "token|Authorization|signature|private key|ConsentGate|audit|sandbox|allowlist|fail closed" docs/security docs/runbook/release-gates.md
```

预期：命中安全文档、runbook、测试说明和 redaction 规则；不得发现真实 secret、真实 DID private key、真实 bearer token 或生产凭据。

### 4.2 Planned security gates

以下 gate 是 release blocker 的目标状态，但当前尚未全部自动化：

| Gate | 启用阶段 | 当前处理 |
|---|---|---|
| production Host RequestBroker transport、registry allowlist、request audit persistence | Phase 4 | Step 01-04 已把 Atomic API `wx.request` 收敛到 `wx-compat::RequestBroker` trait 的 loopback DID broker；demo-only localhost transport 仍不得 production release |
| sandbox escape 专项回归集：constructor/prototype/process/fetch/WebSocket/timer/result size/console size | Phase 3 | 当前由分散测试覆盖基础项 |
| token refresh/revoke/logout、jti replay、DID resolver trust anchor | Phase 3/4 | 当前 only TTL/scope/challenge proof |
| persistent audit sink、retention、redacted export | Phase 3/4 | 当前 in-memory/mock |
| Skill package digest/signature、publisher DID、trusted publisher allowlist | Phase 3/5 | 当前 path/manifest validation |
| Host provider conformance：phone/address/location/file/payment | Phase 3/4 | 当前 host-boundary/fail closed 策略 |

## 5. 兼容矩阵 Gate

每次新增或改变 `wx.*`、`wx.modelContext`、Component Runtime、Render IR、Host provider 或 fallback 行为时，必须同步更新：

- [`wx-api-compatibility-matrix.md`](../architecture/wx-api-compatibility-matrix.md)
- [`component-compatibility-matrix.md`](../architecture/component-compatibility-matrix.md)
- 相关 Step 文档和 roadmap 执行台账

检查项：

| 检查项 | 通过标准 |
|---|---|
| 状态准确 | demo-only、host-boundary、planned 和 supported 与源码/测试一致 |
| owner 明确 | 每项 planned/supported 有 owner crate 或 Host adapter |
| high-risk 明确 | L3/L4 API 或 action 写明 ConsentGate/audit/fail closed |
| unsupported 明确 | 不支持能力有 reason/suggestion，不出现 silent success |
| Phase 决策点 | callback/Promise、Render IR schema、dynamic、Host provider 等待决策项可追踪 |

## 6. Fixture 与 Render IR Gate

当前必须执行：

```bash
cargo test -p component-runtime
cargo test -p dock-cli --test coffee_order_flow
```

当前通过标准：

- coffee 三组件能 mount、render、dispatch tap、触发 `api/call` 和 expire。
- `preview-component` 和 `call-api` render payload 输出 `schemaVersion: "dock.render-ir.v1"`。
- component manifest `relatedPage`、`scope.dynamic`、`expirable`、`expiredText` 进入 redacted runtime metadata / validate report，且不进入 JS state 或 model-visible result。
- render failure 可以 fallback 到 CardSpec，并输出稳定 fallback reason enum string。

Planned gates：

| Gate | 启用阶段 |
|---|---|
| Render IR golden snapshots | Phase 2 |
| address-form、media-review、dynamic-status、location-map-preview fixtures | Phase 2 |
| Host renderer unknown node/action conformance | Phase 4 |

## 7. Demo-only 禁止项

以下能力不得作为 production-ready 发布：

- localhost `wx.request` bridge。
- 无 Host DID 配置时 `wx.login` 返回 `dock-login-code-localhost` 的 fallback。
- mock coffee payment / mock merchant data。
- CLI auto approval / `DecisionConsentProvider::approved()` 作为生产 consent。
- in-memory token/storage/audit 作为生产持久化。
- FastAPI/Rust demo 默认 token issuer secret。
- Mac app `dock-cli` process boundary 作为稳定 Host SDK。

若 release 范围仍包含这些能力，release notes 必须写成 demo/dev only，且不得面向生产用户启用。

## 8. 回滚与失败处理

| 失败类型 | 处理 |
|---|---|
| 基础命令失败 | 停止 release；修复代码或明确环境问题后重跑。 |
| redaction 失败 | 立即阻塞；删除泄露输出，补回归测试，再重跑全量安全相关测试。 |
| consent bypass | 阻塞；修复 Orchestrator/permission/Host provider，补 L3/L4 测试。 |
| allowlist / network deny 失败 | 阻塞；禁止发布任何网络相关变更。 |
| matrix 与实现不一致 | 阻塞当前 Step；先修矩阵或实现，再 Review。 |
| planned gate 未实现 | 不阻塞 Phase 0 文档 release，但必须记录为 planned gap；进入对应 Phase 前升级为 required。 |
| demo-only 被误标 production-ready | 阻塞；修正文档、配置或代码路径。 |

## 9. Release Review Checklist

发布前 Review 必须确认：

- 当前 commit range 中每个 Step 都有 focused commit、Review evidence、verification evidence 和 commit hash。
- `git status --short --branch` 干净，或所有剩余改动都有明确归属和风险记录。
- 所有新增 API/组件/Host/provider 文档已同步到兼容矩阵。
- 所有 L3/L4 行为有 ConsentGate/audit/fail closed 证据。
- 所有 demo-only/mock 行为不会进入 production profile。
- 用户可见输出、日志、audit、Render IR 和 model-visible result 已检查敏感字段。
- 若最终全局 Review 修改文件，必须再执行相关验证并创建单独最终集成 commit。
