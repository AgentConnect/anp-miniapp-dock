# Phase 4：生产运行时与 Host 接入实施计划

## 1. 阶段目标

Phase 4 将 CLI/demo 形态升级为可被真实宿主集成和线上部署的容器。核心产物是稳定 Runtime API、可选 IPC 边界、Skill registry/cache、持久化和 Host adapter contract。

## 2. 涉及模块

| 模块 | 方向 |
|---|---|
| `dock-core` | public runtime service、orchestrator facade、session/action manager |
| `dock-cli` | 改为 Runtime API 的调用者，而不是第二套流程 |
| `skill-loader` | registry/cache/package zip/digest |
| `anp-adapter` | Agent discovery、DID resolver、trusted merchant policy |
| `component-runtime` | Render IR service endpoint |
| `consent-audit` | persistent audit sink |
| Host apps | Mac/Flutter/Web/headless adapters |

## 3. 开发顺序

### 3.1 Runtime API 稳定化

定义稳定 API：

```text
validate_skill(path_or_package)
load_skill(skill_ref)
call_api(session, skill_id, api_name, arguments)
render_component(session, component_path, input)
dispatch_component_action(session, render_id, action)
expire_cards(session, filters)
get_audit_records(filters)
close_session(session)
```

要求：

- 输入输出 JSON 可序列化；
- 错误码稳定；
- API version 可协商；
- CLI 和 Host 共用同一 API。

当前 Step 04-01 已冻结 Rust facade 的初版 contract：

| API | 当前 contract | Phase 4 后续边界 |
|---|---|---|
| `validate_skill(path_or_package)` | `validate_skill_path()` / `RuntimeService::validate_skill()` 返回 `RuntimeResponse<RuntimeValidateSkillResponse>` | 不执行生产 registry download；真实 registry 来源由 04-03 接入。 |
| `load_skill(skill_ref)` | `load_skill_path()` / `RuntimeService::load_skill_response()` 返回 `RuntimeResponse<RuntimeLoadSkillResponse>` | production policy、cache pin 和 rollback 由 04-03 继续收敛。 |
| `call_api(session, skill_id, api_name, arguments)` | `RuntimeService::call_api(RuntimeCallRequest)`；返回 `RuntimeCallResponse`，内部仍走 Orchestrator、Permission、Consent、Audit、RenderRouter | `capabilityToken` 不序列化、不进入 Debug；Host/IPC 形态由 04-02 复用。 |
| `render_component(session, component_path, input)` | `RuntimeService::render_component(RuntimeRenderComponentRequest)`；返回 `RenderOutcome` | 真实 Host renderer conformance 由 04-09 补齐。 |
| `dispatch_component_action(session, render_id, action)` | `RuntimeService::dispatch_component_action(RuntimeDispatchComponentActionRequest)`；`api/call` 回 Orchestrator | unknown/high-risk action protocol 和 Host adapter conformance 由 04-09 补齐。 |
| `expire_cards(session, filters)` | 返回稳定 `RuntimeExpireCardsResponse`，边界标记为 `host-managed-card-store` | 持久 card/session store 不在 04-01 实现，后续由 04-09/04-10 承接。 |
| `get_audit_records(filters)` | 通过 `RuntimeAuditReader` 返回 `RuntimeAuditEvent`，参数和 proof summary 二次脱敏 | 持久化 retention/export 由 04-07 承接。 |
| `close_session(session)` | 返回稳定 `RuntimeCloseSessionResponse`，边界标记为 `stateless-runtime-facade` | token/cache/session 清理由 04-05/04-10 承接。 |

版本策略：

- 当前版本常量为 `dock.runtime.v1`，所有 `RuntimeResponse<T>` / `RuntimeErrorResponse` 都包含 `version` 和 `status`。
- `negotiate_runtime_version()` 对未知版本返回稳定 `unsupported_version` error code。
- `RuntimeErrorResponse` 使用 `ErrorCode::as_str()` 或 runtime 专用 code，并对 token、Authorization、signature、private key、secret、credential 等文本做 redaction。
- `RuntimeSkillSummary.packageRef` 使用 digest ref 或 `local-dev-package`，不输出本机 skill root 绝对路径。
- `dock-cli call-api` 已改为通过 `RuntimeService::call_api()`，但保留原 CLI JSON 字段；`run-demo` 继续复用同一 `RuntimeHarness` 调用 facade。CLI JSON 仍不是 IPC/Host 生产协议，04-02 会定义 headless/IPC envelope。

### 3.2 IPC / SDK 形态

候选：

1. Rust library embedding；
2. local HTTP / JSON-RPC sidecar；
3. gRPC sidecar；
4. headless CLI JSON mode。

建议顺序：先稳定 Rust facade，再做 local HTTP/JSON-RPC。这样 Mac/Flutter/Web host 都能接入，且不会把 CLI 输出当生产协议。

### 3.3 Skill Registry / Cache

开发项：

- merchant Agent manifest；
- skill package URL；
- package digest/signature；
- local cache；
- version pinning；
- rollback；
- cache eviction；
- package.zip 从 demo no-op 变为真实候选路径。

### 3.4 Runtime Config 与 Secret Store

先冻结配置和 secret 边界，再分别实现具体持久化 backend，避免一个 Step 同时跨 token、storage、audit、cache 和 migration。

配置范围：

| 配置 | 策略 |
|---|---|
| runtime profile | dev/demo/production 明确区分 |
| identity/resolver | 保存 provider reference，不保存 private key material |
| token issuer | 保存 secret reference，不保存 token 或 issuer secret |
| storage/audit/cache path | 保存 workspace/runtime 相对或配置路径，输出时 redacted |
| Host providers | 声明 provider handle、capability 和 mock/dev 标记 |

要求：

- config 文件只允许 non-secret value 和 secret reference；
- secret material 只能来自 env、Secret Store 或 Host credential provider；
- production profile 缺少 required provider 时 fail closed 或 release blocked。

### 3.5 持久化

持久化范围：

| 数据 | 策略 |
|---|---|
| token cache | secure store / encrypted SQLite，短期 TTL |
| scoped storage | SQLite，按 DID/merchant/Skill scope，quota |
| audit | append-only 或 SQLite，retention |
| skill cache | digest-keyed directory，read-only after verify |
| runtime config | 只由 3.4 负责 schema、secret boundary 和 provider reference |

拆分顺序：

1. token cache 持久化与恢复；
2. scoped storage 持久化与 quota；
3. persistent audit sink retention/export；
4. Skill cache cleanup 与版本清理。

### 3.6 Host Adapter Contract

Host 必须实现或声明不支持：

- Render IR renderer；
- CardSpec fallback renderer；
- consent prompt；
- phone/address/media/file/location/payment providers；
- openDetailPage fallback；
- event dispatch；
- secure identity provider。

Host 不允许：

- 直接把组件 action 变成高风险系统调用；
- 向 Skill JS 暴露 token/private key；
- 绕过 Runtime audit。

### 3.7 并发、取消与幂等

开发项：

- session manager；
- cancellation token；
- per-session lock for high-risk transaction；
- idempotency key for order/payment；
- retry policy；
- dynamic component cleanup。

## 4. 测试计划

| 测试 | 内容 |
|---|---|
| runtime API tests | load/call/render/action/expire/audit |
| IPC tests | JSON schema、错误码、version |
| persistence tests | restart 后 storage/token/audit 行为 |
| multi-session tests | user/merchant/skill 隔离 |
| host contract tests | mock Host provider success/fail |
| rollback tests | skill version pin/rollback |

## 5. 阶段完成检查

- [ ] CLI 使用同一 Runtime API。
- [ ] 至少一个 Host 通过稳定协议接入。
- [ ] Skill package 可下载/缓存/校验/回滚。
- [ ] runtime config 与 secret store 边界冻结，且 storage/token/audit/cache 分别有 focused production candidate 或 release blocker。
- [ ] 多 session 隔离和高风险串行策略有测试。
