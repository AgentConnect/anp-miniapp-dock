# Phase 6：观测、性能与发布运营实施计划

## 1. 阶段目标

Phase 6 让容器具备线上运行的可观测性、性能边界、发布门禁、灰度和回滚能力。完成后，线上问题能定位到具体 session、Skill、API、Host provider 或 merchant Agent，而不需要查看敏感 payload。

## 2. 观测模型

### 2.1 结构化事件

必须记录但默认脱敏：

- `skill_load_start/end`
- `api_call_start/end`
- `wx_api_call_start/end`
- `request_start/end`
- `consent_prompt/decision`
- `component_render_start/end`
- `component_event`
- `fallback_used`
- `audit_record_written`
- `sandbox_limit_hit`

Step 06-01 已在 `dock-core` 固化事件 schema `dock.observability.event.v1`、脱敏策略 `dock.observability.redaction.v1`、`NoopObservabilitySink` 和 `InMemoryObservabilitySink`，并在 `RuntimeService` 的 Skill load、API call、ConsentGate、Render IR render、component event、Host action、fallback、audit written 和 timeout/limit 边界发出结构化事件。`wx_api_call_*` 与 `request_*` 已作为稳定事件类型保留，后续 Step 06-02 可在 QuickJS / RequestBroker / metrics bridge 中复用同一事件模型接入更细粒度链路。

公共字段：

```text
traceId
sessionId
skillId
apiName?
componentPath?
merchantDid?
userDid? (可 hash)
runtimeVersion
renderIrVersion
outcome
latencyMs
```

公共字段落地规则：

- `userDid` 不直接记录，默认写入 `userDidHash = sha256:<hex>`。
- `merchantDid` 可按 policy 作为定位字段输出；涉及 token、Authorization、Signature、private key、手机号、地址、文件内容、精确位置、本机绝对路径的字段必须在 emit 前替换为 `[REDACTED]`。
- `fields` 只能放定位 metadata、计数、状态、risk level、release/readiness 信息，不放 raw arguments、HTTP body、文件内容或 Host provider 原始 payload。
- Runtime 默认使用 no-op sink；Host、CLI 或测试可以注入 sink。Step 06-01 已完成 structured event sink；Step 06-02 已补 `dock.observability.metric.v1`、`dock.observability.trace.v1`、in-memory/test metrics recorder、RuntimeService metrics hooks 和 QuickJS executor / `wx.request` / token path metrics hooks。当前不接入外部日志、Prometheus 或 OpenTelemetry vendor exporter。

### 2.2 Metrics

| 指标 | 目的 |
|---|---|
| API latency | 识别慢接口 |
| VM execution time | sandbox 性能 |
| render latency | 组件渲染性能 |
| request status | merchant/网络错误 |
| fallback rate | 兼容性质量 |
| consent approve/deny rate | 风控与 UX |
| unsupported API count | 迁移阻塞点 |
| sandbox timeout/memory hit | 恶意或低质量 Skill |
| token refresh/fail count | DID/auth 健康度 |

Step 06-02 当前实现的本地指标契约：

- `dock.skill_load.total`：Skill load outcome 与 supply-chain status。
- `dock.api_call.total`、`dock.api_latency_ms`：Runtime API call start/end、outcome、risk level 和 bounded API name。
- `dock.api_vm_execution_ms`：QuickJS Atomic API VM execution time，按 API name 和 outcome 记录。
- `dock.render_latency_ms`：Render IR render latency，按 manifest bounded component path 和 outcome 记录。
- `dock.request.total`、`dock.request_latency_ms`：QuickJS `wx.request` status，HTTP status 只记录 `2xx`/`3xx`/`4xx`/`5xx`/`other` 或低基数失败码，不记录 URL、query、headers 或 body。
- `dock.fallback.total`：fallback reason enum。
- `dock.consent.total`：prompt / decision outcome 和 risk level。
- `dock.unsupported_api.total`：Runtime API registry miss count。
- `dock.sandbox_limit.total`：timeout / sandbox limit 命中。
- `dock.token_refresh.total`、`dock.token_refresh_latency_ms`：`wx.login` / `wx.checkSession` token/session path outcome，不记录 raw token。
- `dock.audit_record.total`、`dock.audit_latency_ms`：audit record outcome、risk level 和 trace correlation。

所有 label 必须低 cardinality；`ObservabilityMetric` 和 `TraceSpan` 会对 URL query、header/token marker、DID 原文、本机绝对路径和过长 label 执行 redaction。Runtime 和 QuickJS 默认 no-op；测试与 Host 集成可注入 `InMemoryMetricsSink` 或未来 exporter sink。

### 2.3 Tracing

一条用户请求应串起：

```text
Host message
  -> model/intent decision
  -> Skill/API call
  -> wx.login/checkSession/request
  -> merchant response
  -> component render
  -> user action
  -> follow-up api/call
  -> audit
```

Step 06-02 当前 trace propagation：

- `RuntimeOperationOptions.trace` 可接收 Host/IPC 层传入的 `traceId` 和 parent span。
- `ApiCallContext.trace` 把 Runtime API trace 传入 Orchestrator、API executor、component action 和 nested `api/call`。
- RuntimeService 为 API call、render、component action、audit 和 IPC 记录 span。
- QuickJS executor 为 API VM、`wx.request`、`wx.login` / `wx.checkSession` 记录 span，并继承 `ApiCallContext.trace`。
- 显式 `runtime.renderComponent` 当前没有 operation trace 输入字段，因此独立生成 root trace；Host envelope 若需要把 render 与上一跳强绑定，应在后续 Host/IPC contract 扩展中为 render request 增加 operation trace 字段。

## 3. 性能基线

建议基准：

| 基准 | 初始目标 |
|---|---|
| Skill load | 本地 P50/P95 |
| API VM cold call | P50/P95 |
| API VM warm-ish call | P50/P95 |
| Component render | P50/P95 |
| Render IR size | P50/P95 |
| token lookup | P50/P95 |
| storage read/write | P50/P95 |
| memory per VM | max |

具体数值应在实现基准测试后写入 release notes，不在计划文档中凭空承诺。

## 4. CI/CD Gates

基础 gates：

```bash
cargo metadata --format-version 1 --no-deps
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

产品 gates：

- compatibility matrix coverage；
- sandbox escape regression；
- redaction regression；
- DID/token replay/scope tests；
- Render IR snapshots；
- fixture E2E；
- markdown link check；
- release notes completeness。

## 5. 发布策略

### 5.1 版本化对象

- Runtime API version；
- Render IR schema version；
- capability token version；
- Skill package contract version；
- Host adapter contract version。

### 5.2 灰度流程

1. headless fixture 全量通过；
2. internal Host canary；
3. allowlisted merchant Skill；
4. expand by publisher DID / skill version；
5. monitor fallback/error/consent/token metrics；
6. rollback on gate breach。

### 5.3 回滚条件

- token leakage regression；
- consent bypass；
- sandbox escape；
- fallback rate 超阈值；
- auth failure rate 激增；
- Host crash / Render IR incompatible；
- audit write failure。

## 6. 运维 Runbook

需要覆盖：

- DID 验签失败；
- token scope mismatch；
- allowlist deny；
- component render failed；
- sandbox timeout；
- storage quota exceeded；
- audit sink unavailable；
- Host provider unavailable；
- merchant Agent unavailable；
- Skill package signature mismatch；
- rollback and cache purge。

## 7. 阶段完成检查

- [x] 结构化日志和 metrics 默认脱敏。
- [x] trace 能串起 Runtime API、QuickJS VM、`wx.login` / `wx.checkSession` / `wx.request`、render、action、nested `api/call` 和 audit 的本地测试链路；完整 Host message / model decision 侧 trace 仍需真实 Host adapter 注入。
- [ ] 性能基准有自动化脚本。
- [ ] CI gates 覆盖安全、兼容、snapshot、文档。
- [ ] canary/rollback runbook 可执行。
- [ ] release notes 包含版本、兼容变化、风险和回滚方式。
