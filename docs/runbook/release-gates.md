# Release Gates Runbook

> 状态：Phase 4 runtime/Host gates 进行中；Step 03-02 sandbox/resource、Step 03-03 permission/allowlist、Step 03-04 DID/token lifecycle、Step 03-05 consent/audit persistence、Step 03-06 Skill package integrity/supply-chain、Step 04-03 本地 registry/cache/version/rollback contract、Step 04-04 runtime config / secret boundary、Step 04-05 token cache persistence contract / restore policy、Step 04-06 scoped storage persistence contract / quota / restore / delete-scope、Step 04-07 audit persistence profile / retention-export report / unavailable fail-closed 已有本地 release gate 证据
> 日期：2026-06-13
> 范围：定义 `anp-miniapp-dock` 每次进入 production-readiness milestone、release branch 或 production deployment 前需要执行或明确记录的验证、Review、红线和回滚条件。
> 上游计划：[`../plan/production-readiness-roadmap.md`](../plan/production-readiness-roadmap.md) Step 03-01。

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

本节把安全 gate 分成三类：

- **当前必须执行**：当前代码库已有自动化或手工 gate，任何 Step / release 都必须执行或记录无法执行原因。
- **Phase 3 required**：Phase 3 的 Step 03-02 至 03-06 必须实现或升级为 required 的 gate；在对应 Step 完成前不得写成“已通过”。Step 03-02 已升级为本地 required release gate；CI 自动化仍待 Phase 6。
- **Phase 4/5 后续 gate**：依赖 production Host、registry、持久化 backend、开发者工具或 CI 自动化的 gate；当前仍是 production release blocker，但不作为 03-01 的已实现证据。

### 4.1 当前必须执行

当前已有自动化测试覆盖以下安全主线，随 `cargo test --workspace` 执行：

| Gate | 证据 |
|---|---|
| Skill package path escape / absolute path / outside symlink / zip slip deny | [`coffee_skill_load.rs`](../../crates/skill-loader/tests/coffee_skill_load.rs) |
| Skill package digest/signature contract、trusted publisher allowlist、quarantine、validate supply-chain report | [`integrity.rs`](../../crates/skill-loader/src/integrity.rs)、[`coffee_skill_load.rs`](../../crates/skill-loader/tests/coffee_skill_load.rs)、[`mcp_validation.rs`](../../crates/mcp-schema/tests/mcp_validation.rs)、[`commands.rs`](../../crates/dock-cli/src/commands.rs) |
| Skill registry/cache contract：local/package URL/registry id reference、digest-keyed cache、source/cache digest verify、readonly cache、latest/pinned/prerelease/rollback、rollback pin eviction、cache audit redaction | [`registry.rs`](../../crates/skill-loader/src/registry.rs)、[`skill_registry_cache.rs`](../../crates/skill-loader/tests/skill_registry_cache.rs) |
| Runtime config / secret boundary：`dock.runtime.config.v1` schema、profile、load priority、provider handle、secret reference、production blockers、redacted diagnostics | [`config.rs`](../../crates/dock-core/src/config.rs)、[`runtime_config.rs`](../../crates/dock-core/tests/runtime_config.rs) |
| Token cache persistence contract：backend profile、restart restore、expiry/revocation/replay/scope/trust rejection、redacted restore report、dev-only in-memory backend | [`token.rs`](../../crates/anp-adapter/src/token.rs) |
| Scoped storage persistence contract：DID/merchant/Skill/namespace scope、backend profile、restart restore、quota rejection、invalid entry cleanup、remove/clear/delete scope、redacted restore report、local file backend dev-only | [`storage.rs`](../../crates/wx-compat/src/storage.rs)、[`scoped_storage.rs`](../../crates/wx-compat/tests/scoped_storage.rs) |
| Persistent audit sink contract：backend profile、redacted export/retention report、Runtime persistent reader、corrupt backend `audit_unavailable`、L3/L4 executor 前 audit unavailable fail closed、local JSONL backend dev-only | [`audit.rs`](../../crates/consent-audit/src/audit.rs)、[`payment_requires_consent.rs`](../../crates/consent-audit/tests/payment_requires_consent.rs)、[`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs)、[`runtime_facade.rs`](../../crates/dock-core/tests/runtime_facade.rs) |
| Manifest component metadata、input `format:image/file`、production warning 分层 | [`mcp_validation.rs`](../../crates/mcp-schema/tests/mcp_validation.rs) |
| `dock-cli validate` 兼容报告、API 注册 mismatch blocker、demo-only release blocker | [`commands.rs`](../../crates/dock-cli/src/commands.rs)、[`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) |
| Atomic API sandbox、unsafe require、timeout、WebSocket/timer globals deny、Promise job drain、console/result size limit | [`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs)、[`bridge.rs`](../../crates/js-runtime-quickjs/src/bridge.rs)、[`api_vm.rs`](../../crates/js-runtime-quickjs/src/api_vm.rs) |
| Component sandbox、default no network/timer、native bridge hidden、snapshot size limit、expire 后事件失败 | [`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs)、[`component_vm.rs`](../../crates/component-runtime/src/component_vm.rs) |
| Component profile 默认 deny request/timer，dynamic 才可表达 request/timer boundary | [`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs) |
| Request allowlist deny by default / miss deny without transport | [`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs) |
| token scope isolation、HTTP Signature fallback、401 retry | [`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs) |
| DID auth session cache 隔离、过期 refresh、clear/revoke 语义 | [`session.rs`](../../crates/anp-adapter/src/session.rs) |
| Atomic API `wx.login` receipt、`wx.checkSession`、JS auth header fail closed、response header redaction | [`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) |
| deterministic unsupported API stub、sync throw、nested `wx.cloud.*`、unknown root fallback、safe reason/suggestion | [`unsupported.rs`](../../crates/wx-compat/src/unsupported.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) |
| Atomic API scoped storage JS bridge、sync/async shape、DID/merchant/skill scope、JSON-safe validation、model-visible 隔离 | [`storage.rs`](../../crates/wx-compat/src/storage.rs)、[`scoped_storage.rs`](../../crates/wx-compat/tests/scoped_storage.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) |
| Atomic API 与 Component VM device/app info 最小字段、防指纹和 shared default 防漂移 | [`model_context.rs`](../../crates/wx-compat/src/model_context.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs) |
| Atomic API 高风险 Host boundary：无 provider fail closed、未 consent 不执行 provider、dev-only mock 标识、opaque handle、本地路径拒绝、payment 不收集密码 | [`high_risk.rs`](../../crates/wx-compat/src/high_risk.rs)、[`high_risk_provider.rs`](../../crates/wx-compat/tests/high_risk_provider.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) |
| Component dynamic request/timer 最小 gate：`scope.dynamic` 驱动注入、默认 deny、auth header deny、response header redaction、timer limit/clear/expire cleanup | [`component_vm.rs`](../../crates/component-runtime/src/component_vm.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs) |
| L3 payment consent、Host consent adapter、provider unavailable audit、persistent audit redaction | [`payment_requires_consent.rs`](../../crates/consent-audit/tests/payment_requires_consent.rs)、[`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs) |
| CLI/demo redaction | [`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) |

补充红线抽样：

```bash
rg -n "token|Authorization|signature|private key|ConsentGate|audit|sandbox|allowlist|fail closed" docs/security docs/runbook/release-gates.md
```

预期：命中安全文档、runbook、测试说明和 redaction 规则；不得发现真实 secret、真实 DID private key、真实 bearer token 或生产凭据。

### 4.2 Phase 3 required security gates

以下 gate 是 Phase 3 必须补齐的 release blocker。完成对应 Step 后，必须把实际命令、测试文件和残余风险回填到本 runbook、Threat Model、主 Plan 台账和 Step 文档。

| Gate | Step | 当前处理 | 完成后必须记录 |
|---|---|---|---|
| sandbox escape 专项回归集：constructor/prototype/process/fetch/WebSocket/timer/result size/console size | 03-02 | 已升级为本地 required release gate：Atomic API VM 禁用 WebSocket/timer globals，限制 Promise job drain、console 和 result size；Component VM 扩展 escape regression 并限制 snapshot output size；dynamic 例外仍只由 component capability profile 开放 | `cargo test -p js-runtime-quickjs sandbox`、`cargo test -p js-runtime-quickjs limit`、`cargo test -p js-runtime-quickjs console`、`cargo test -p js-runtime-quickjs invalid_atomic`、`cargo test -p js-runtime-quickjs pending_job`、`cargo test -p component-runtime sandbox`、`cargo test -p component-runtime dynamic`、`cargo test -p component-runtime snapshot_size`、`cargo test -p js-runtime-quickjs`、`cargo test -p component-runtime`；CI 自动化和 resource metrics 仍待 Phase 6 |
| permission policy engine：`Allow` / `Deny` / `Prompt` / `MockAllowed(dev_only)`，Host deny override，manifest permission，dynamic scope，merchant trust policy | 03-03 | 已补齐本地 required gate：`wx-compat::PermissionPolicyEngine` 统一 decision，Host deny override 优先，Host allow 不能替代 manifest/meta/dynamic 声明，mock provider 只能 dev/headless；`dock-core::AuditEvent` 记录脱敏 `permissionDecision` | `cargo test -p wx-compat permission`、`cargo test -p dock-core permission`；生产 Host policy UI/config、provider conformance 和 persistent audit 仍待 Phase 4/03-05 |
| network allowlist：scheme、host、port、path prefix、method、scope，默认 deny | 03-03 | 已补齐本地 required gate：`anp-adapter::NetworkAllowlistRule` 支持 scheme/host/port/path prefix/method/scope，默认空 policy deny，mismatch 在 transport 前失败 | `cargo test -p anp-adapter allowlist`；Host registry 配置来源、生产 transport 和 persistent request audit 仍待 Phase 4/03-05 |
| DID/token lifecycle：token claims version、refresh、revoke/logout、cache eviction、jti replay、challenge nonce 一次性、resolver cache/trust anchor | 03-04 | 已补齐本地 required gate：`CapabilityTokenLifecycleStore` / `InMemoryTokenLifecycleStore` 支持 revoke、expired prune 和 high-risk `ConsumeOnce` jti gate；`DidAuthSessionManager` 支持 revoke/logout 和 expired eviction；`ChallengeNonceStore`、`TrustedDidDocumentResolver` 覆盖 nonce 一次性、cache TTL、trust anchor、unknown/mismatch fail closed；demo-server 登录尝试开始即消费 challenge，服务端 bearer 校验检查 revoked jti | `cargo test -p anp-adapter token`、`cargo test -p anp-adapter session`、`cargo test -p anp-adapter challenge`、`cargo test -p anp-adapter`、`cargo test -p demo-server token`、`cargo test -p demo-server`、`cargo test -p js-runtime-quickjs wx_login`、`cargo test -p dock-cli --test coffee_order_flow`；生产 token cache/revocation restore、跨进程 replay store、DID network/rotation、secret store 待 Phase 4/6 |
| Host consent adapter、ConsentProof policy version/prompt digest/decision actor/timestamp/parameter digest | 03-05 | 已补齐本地 required gate：`consent-audit::HostConsentAdapter`、`dock-core::HostConsentGateAdapter`、`ConsentProof` policy/prompt/actor/digest 字段；provider unavailable fail closed 并记录 blocked consent audit | `cargo test -p consent-audit consent`、`cargo test -p dock-core consent`；真实 Host UI/conformance 待 Phase 4 |
| persistent audit sink、retention、query/export redaction | 03-05 / 04-07 | 03-05 已补齐本地 `FileAuditSink` JSONL 持久化；04-07 已补齐 `AuditPersistenceProfile`、`AuditExportReport`、`AuditRetentionReport`、Runtime persistent reader、corrupt backend `audit_unavailable` 和 L3/L4 executor 前 unavailable fail-closed gate；`localFileJsonl` 明确 `productionReady = false` | `cargo test -p consent-audit audit`、`cargo test -p dock-core audit`；部署级 Host/encrypted audit backend、backend config、migration、access control、export approval、privacy deletion 和 durability/alerting 待 Phase 4/6 |
| Skill package digest/signature、publisher DID、trusted publisher allowlist、quarantine、remote require/path/symlink/zip slip deny | 03-06 | 已补齐本地 required gate：`skill-loader` 计算 normalized package `sha256` digest，`mcp-schema` 校验 `_meta.anp.supplyChain` contract，production integrity policy 对 unsigned、digest mismatch、signature mismatch、unknown publisher fail closed/quarantine，QuickJS CommonJS 拒绝 remote require，`dock-cli validate` 输出 redacted supply-chain report 和 release blocker；未签名本地包仍是 dev/demo-only | `cargo test -p skill-loader package`、`cargo test -p mcp-schema -p dock-cli validate`、`cargo test -p js-runtime-quickjs remote_require_is_rejected`、`cargo run -p dock-cli -- validate examples/coffee-skill`；真实 registry/cache、生产签名 verifier、publisher allowlist 配置来源和 CI 自动化仍待 Phase 4/6 |

### 4.3 Phase 4/5 后续 security gates

以下 gate 依赖生产 Host、registry/cache、持久化配置、开发者工具或 CI 自动化。它们仍是 production release blocker，但不应在 Phase 3 前半段误标为已自动化。

| Gate | 启用阶段 | 当前处理 |
|---|---|---|
| production Host RequestBroker transport、registry allowlist、request audit persistence | Phase 4 | Step 01-04 已把 Atomic API `wx.request` 收敛到 `wx-compat::RequestBroker` trait 的 loopback DID broker；Step 02-05 已给 dynamic component request 接入 injected broker boundary；Step 04-03 已冻结本地 registry/cache contract，但真实远端 registry download、Host registry allowlist 和 request audit persistence 仍未完成；demo-only/unsupported transport 仍不得 production release |
| 真实远端 registry download、生产签名 verifier、publisher trust policy 配置来源 | Phase 4/6 | Step 03-06 已完成 package integrity gate；Step 04-03 已完成本地 registry/cache/version/rollback gate。真实 HTTPS/DID registry discovery/download、生产签名算法 verifier、publisher allowlist 配置来源、CI release report 仍是 production release blocker。 |
| Host provider conformance：phone/address/location/file/payment/scan/phone call/share/detail page | Phase 4 | 当前只有 host-boundary/fail closed 策略和 mock/dev-only fixtures；真实 provider UI 与 least-privilege field shape 待 Host adapter contract |
| secret store、token cache 持久化、scoped storage 持久化、audit retention/export 配置化 | Phase 4 | Step 04-04 已冻结 runtime config schema、secret reference、provider/path handle、production profile release blockers 和 redacted diagnostics；Step 04-05 已冻结 token cache persistence trait、restore policy、redacted report 和 dev-only in-memory backend gate；Step 04-06 已冻结 scoped storage persistence trait、namespace scope、quota/restore/delete-scope/redaction gate 和未加密 local file dev/test backend；Step 04-07 已冻结 audit profile、redacted export/retention report、Runtime persistent reader 和 L3/L4 audit unavailable fail-closed gate。真实 secret resolve、生产 Host secure store/encrypted token/storage/audit backend、storage/audit migration/access control、export approval、privacy deletion、Skill cache cleanup 仍由 04-08 和后续 Host/ops gate 分别承接 |
| CLI compatibility / inspect / import 报告 schema、developer self-certification | Phase 5 | 当前 `dock-cli validate` 已输出 demo-only compatibility report；完整 migration/import/report schema 待 Phase 5 |
| CI/CD 自动 gate runner、link checker、matrix schema checker、snapshot gate、privacy deletion runbook | Phase 6 | 当前手工和本地命令执行；自动化 release report 待 Phase 6 |

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
cargo test -p component-runtime snapshot
cargo test -p dock-cli fixture
cargo test -p dock-cli --test coffee_order_flow
```

当前通过标准：

- coffee 三组件能 mount、render、dispatch tap、触发 `api/call` 和 expire。
- `preview-component` 和 `call-api` render payload 输出 `schemaVersion: "dock.render-ir.v1"`。
- component manifest `relatedPage`、`scope.dynamic`、`expirable`、`expiredText` 进入 redacted runtime metadata / validate report，且不进入 JS state 或 model-visible result。
- render failure 可以 fallback 到 CardSpec，并输出稳定 fallback reason enum string。
- dynamic 组件只有声明 `scope.dynamic` 后才注入受限 request/timer；默认 deny、auth header deny、timer limit/clear/expire cleanup 已有 focused tests。
- address-form、media-review、dynamic-status、location-map-preview fixture packages 可被 `dock-cli validate` / `preview-component` 读取。
- `testdata/render-ir/*.json` golden snapshots 包含 render、actions、warnings、metadata、state 和 audit summary，且不含真实 token、Authorization、signature、private key path、本机路径、手机号、真实地址、经纬度。

Planned gates：

| Gate | 启用阶段 |
|---|---|
| Host renderer unknown node/action conformance | Phase 4 |

## 7. Demo-only 禁止项

以下能力不得作为 production-ready 发布：

- localhost `wx.request` bridge。
- 无 Host DID 配置时 `wx.login` 返回 `dock-login-code-localhost` 的 fallback。
- mock coffee payment / mock merchant data。
- CLI auto approval / `DecisionConsentProvider::approved()` 作为生产 consent。
- in-memory token/storage/audit 作为生产持久化；token cache 的 `inMemoryDev` profile、scoped storage 的 `localFileUnencrypted` profile、audit 的 `localFileJsonl` profile 只能作为 dev/test/local evidence backend。
- production profile 中启用 mock provider、dev-only Host provider、inline secret 或缺失 required provider。
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
| 后续阶段 gate 未实现 | 不阻塞当前 Step 的文档基线，但必须记录为 planned gap 或后续阶段 release blocker；进入对应 Phase 前升级为 required。 |
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
