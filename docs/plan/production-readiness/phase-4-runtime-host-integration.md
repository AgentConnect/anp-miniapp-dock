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

当前 Step 04-02 选择的首个可测试 Host 接入形态是 `dock-cli runtime-json`：

- transport mode：`headless-cli-json`；
- binding：`local-process-stdio`；
- 目标：给非 Rust Host 一个本地进程级、machine-readable 的 JSON envelope，同时继续复用 04-01 的 `RuntimeService` facade；
- 非目标：不在本 Step 启动 HTTP/gRPC sidecar，不把旧的 developer CLI JSON 当作生产 IPC 协议。

请求 envelope：

```json
{
  "apiVersion": "dock.runtime.v1",
  "requestId": "req-1",
  "method": "runtime.callApi",
  "params": {}
}
```

响应 envelope 必须包含：

- `apiVersion`、`requestId`、`method`、`status`；
- `result` 或 `error`；
- `redaction.marker = "[REDACTED]"`、`redaction.policy = "dock.runtime.redaction.v1"`；
- `transport.mode = "headless-cli-json"`、`transport.binding = "local-process-stdio"`。

当前支持的 IPC method：

| Method | Runtime facade |
|---|---|
| `runtime.negotiateVersion` | `negotiate_runtime_version()` |
| `runtime.validateSkill` | `RuntimeService::validate_skill()` |
| `runtime.loadSkill` | `RuntimeService::load_skill_response()` |
| `runtime.callApi` | `RuntimeService::call_api()` |
| `runtime.renderComponent` | `RuntimeService::render_component()` |
| `runtime.dispatchComponentAction` | `RuntimeService::dispatch_component_action()` |
| `runtime.expireCards` | `RuntimeService::expire_cards()` |
| `runtime.getAuditRecords` | `RuntimeService::get_audit_records()` |
| `runtime.closeSession` | `RuntimeService::close_session()` |

安全边界：

- IPC 只负责传输，不允许绕过 permission、ConsentGate、audit、redaction 或 package integrity gate；
- `capabilityToken` 可作为请求参数进入 Runtime facade，但 request DTO 不派生 `Debug` 且响应默认不序列化该字段；
- invalid method、invalid params、version mismatch 和 request parse/schema error 都返回 redacted envelope；
- 当前 binding 是本地进程 stdio，后续 HTTP/JSON-RPC sidecar 必须单独补 loopback/socket binding、安全配置和 conformance tests。

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

当前 Step 04-03 已在 `skill-loader` 内冻结本地 registry/cache/version/rollback contract：

| 能力 | 当前 contract | 边界 |
|---|---|---|
| Skill reference | `SkillReference` 支持 local path、package URL 和 registry id；保留 `skillId`、`publisherDid`、`version`、`digest` | 真实远端 marketplace/discovery 仍未接入。 |
| Registry trait | `SkillRegistry::resolve_skill()` 和 `LocalSkillRegistry` 可用本地 manifest/fixture 模拟 merchant registry | 不执行生产 HTTP download；真实网络必须后续接入 allowlist/HTTPS/DID policy。 |
| Cache key | `SkillCacheKey = publisher DID + skill id + version + digest`，目录名经过 sanitize | cache 命中也会重新用 registry digest 和 package integrity policy 验证。 |
| Verified cache | `SkillCache::load_or_insert()` 先验证源包 digest、签名、publisher allowlist，再复制到 digest-keyed cache 并设置 readonly | readonly 是本地文件权限 gate，不等同部署级防篡改或加密存储。 |
| Version selection | `Latest`、`Pinned(version)`、`Rollback { beforeVersion }`，默认不选 prerelease，显式 `registry_id_with_prerelease()` 才允许 | 版本比较为 semver-like numeric order，仍不替代生产 release policy。 |
| Rollback / eviction | 支持 rollback pin，`evict_unpinned()` 不删除 retain set 或 rollback pin | cache cleanup、quarantine 生命周期和 privacy/delete hooks 后续由 04-08 承接。 |
| Audit summary | `CachedSkillMetadata::audit_summary()` 输出 source type、package ref、publisher、skill、version、digest、supply-chain status、cache flags，并脱敏 package URL secret/query | 不输出本机 cache root 或 private path。 |

04-03 不声明真实 ANP Agent registry、远端 zip 下载、生产签名算法 verifier 或生产 publisher trust policy 配置已完成；这些仍是 Phase 4/6 的后续 release blocker。

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

当前 Step 04-04 已在 `dock-core` 冻结 `dock.runtime.config.v1` contract：

| 能力 | 当前 contract | 边界 |
|---|---|---|
| profile | `development`、`demo`、`production` 明确区分；默认是 development | production 缺少 required provider 时只产生 release blocker，不自动创建真实 provider。 |
| config load priority | `builtInDefault -> configFile -> environment -> cliArgument -> hostOverride` | 本 Step 冻结优先级枚举，不实现完整文件/环境合并器。 |
| secret reference | 只允许 `env`、`secretStore`、`hostCredentialProvider` reference | 不解析、不读取、不缓存真实 secret material。 |
| provider/path handle | identity、resolver、allowlist、token issuer、storage、audit、cache、Host provider 均使用 reference/handle | storage/audit/cache 的实际 backend 由 04-05 至 04-08 分别实现。 |
| diagnostics | `redactedDiagnostics()` 只输出 configured 状态、profile、capability summary 和 redaction metadata | 不输出 secret store key、private key path、本机绝对路径、token、Authorization、signature 或 raw secret。 |
| production release blockers | 缺少 identity/resolver/allowlist/token issuer secret、使用 in-memory backend、缺少 render/consent Host provider、启用 mock/dev-only provider 均 block release | 不把 demo/mock/in-memory 配置写成 production-ready。 |

### 3.5 持久化

持久化范围：

| 数据 | 策略 |
|---|---|
| token cache | secure store / encrypted SQLite，短期 TTL |
| scoped storage | SQLite，按 DID/merchant/Skill scope，quota |
| audit | append-only 或 SQLite，retention |
| skill cache | digest-keyed directory，read-only after verify |
| runtime config | 只由 3.4 负责 schema、secret boundary 和 provider reference |

当前 Step 04-05 已在 `anp-adapter` 冻结 capability token cache persistence contract：

| 能力 | 当前 contract | 边界 |
|---|---|---|
| persistence backend | `TokenCachePersistenceBackend` trait 支持 load/replace entry；profile 区分 `inMemoryDev`、`hostSecureStore`、`encryptedBackend` | 当前只提供 `InMemoryTokenCachePersistenceBackend` 作为 dev/test backend；生产必须接 Host secure store 或 encrypted backend。 |
| persisted entry | entry 绑定 scope、issuer、audience、jti、expiry 和 raw token secure boundary | `PersistentCapabilityTokenEntry` Debug redacted，且不作为公开 JSON diagnostics 输出。 |
| restart restore | `PersistentCapabilityTokenCache::restore()` 只恢复未过期、signature/trust 有效、metadata/claims/scope 匹配、未 revoked、未 consumed once 的 token | rejected entry 会从 backend snapshot 清掉，并只在 report 中暴露 scope summary 与 reason。 |
| redacted report | `TokenCacheRestoreReport` 只输出 backend profile、production-ready flag、计数、rejection reason 和 redaction metadata | 不输出 raw token、Authorization、signature、private key path 或 secret。 |
| failure policy | fallible `try_put()` / `try_clear()` 先写 persistence snapshot，成功后才更新内存 cache；`CapabilityTokenCache` trait 方法复用 fail-closed 路径 | 若 production backend 不可用，写入/清理不会先污染内存状态；调用方若需要错误详情应使用 fallible API。 |

当前 Step 04-06 已在 `wx-compat` 冻结 scoped storage persistence contract：

| 能力 | 当前 contract | 边界 |
|---|---|---|
| scope | `StorageScope = user DID + merchant DID + Skill id + namespace`，默认 namespace 为 `default` | Atomic API VM 当前仍使用默认 namespace；更多 Host-managed namespace 需后续显式 wiring。 |
| persistence backend | `ScopedStoragePersistenceBackend` trait 支持 load/restore snapshot 和 replace entries；profile 区分 `inMemoryDev`、`localFileUnencrypted`、`hostEncryptedStore`、`encryptedSqlite` | 只有 Host encrypted store / encrypted SQLite profile 标记 production-ready；local file JSON backend 未加密，只能 dev/test/local evidence。 |
| restart restore | `PersistentScopedStorage::restore()` 恢复合法 entry，拒绝 invalid 或超 quota entry，并重写 backend snapshot 清理 rejected entry | corrupt JSON snapshot 整体 fail closed，需 Host repair/reset；生产 repair/backup 策略后续补。 |
| quota / cleanup | `try_set_storage()` 先校验 aggregate quota 并写 backend snapshot，成功后才更新内存；支持 remove、clear 和 delete scope 持久化 | 生产 privacy deletion 仍需 Host/ops runbook 串联 token/audit/cache 清理。 |
| redacted report | `StorageRestoreReport` 只输出 backend profile、production-ready flag、计数、scope summary、key/value bytes、reason 和 redaction metadata | 不输出 raw key、raw value、token、Authorization、private path 或隐私原文。 |

拆分顺序：

1. token cache 持久化与恢复；已由 04-05 完成本地 contract 和 dev-only backend gate；
2. scoped storage 持久化与 quota；已由 04-06 完成本地 contract、quota/restore/delete-scope gate 和 dev-only local file backend gate；
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
