# Threat Model 与安全控制矩阵

> 状态：Phase 3 控制矩阵基线
> 日期：2026-06-13
> 范围：覆盖 `anp-miniapp-dock` 从 Skill 包加载、QuickJS VM、wx Compatibility Layer、ANP DID / capability token、ConsentGate、audit、Render IR 到 Host provider 的主要威胁、风险分级、控制措施、测试 gate 和残余风险。
> 上游计划：[`../plan/production-readiness-roadmap.md`](../plan/production-readiness-roadmap.md) Step 03-01 至 03-02。
> 执行说明：本文冻结 Phase 3 安全 contract。Step 03-02 已把 sandbox/resource gate 升级为本地 required release gate；Step 03-03 至 03-06 仍是必须实现或补强的 Phase 3 release blocker。CI 自动化仍以后续 Phase 6 gate 为准。

## 1. 安全目标

`anp-miniapp-dock` 的安全目标是让 MiniApp MCP Skill 能在 Agent 对话中执行，同时不让 Skill 获得宿主私钥、capability token、任意网络、任意文件、真实支付、隐私数据或未授权 Host 能力。

核心原则：

- Skill JS 默认不可信。
- 高风险能力默认 fail closed。
- ANP DID、capability token、HTTP Signature 和 Host provider 均在容器/Host 边界内，不暴露给 Skill。
- `api/call`、支付、隐私、文件、位置、电话和外部链接必须经过 Orchestrator、Host provider、ConsentGate 和 audit。
- CLI JSON、日志、audit export、Render IR 和模型可见输出不得包含 token、Authorization、HTTP Signature、private key path、手机号、地址、文件内容或未脱敏位置。

## 2. 风险等级与默认发布门槛

风险等级沿用 `consent-audit` 的 L0-L4 分层。任何新增或改动 API、组件 action、Host provider、network capability、storage、DID/token 或 package loading 行为时，必须先归入下表，再补齐对应 gate。

| 等级 | 能力范围 | 默认处理 | 必需控制 | 必需测试 / Gate | Release blocker |
|---|---|---|---|---|---|
| L0 | 公开读、常量、无状态 runtime 信息、Render IR 普通节点 | 可默认允许，但仍不得泄露 `_meta` 或 Host 私密字段 | schema validation、redaction、unsupported fail shape | unit / snapshot / docs gate | 否，除非泄露敏感信息或破坏公开契约 |
| L1 | 登录/session 标识、最小设备/应用信息、DID 绑定但不含 secret | 只允许最小字段；禁止真实设备指纹、credential path 和 token | DID/session binding、字段最小化、redaction | DID/session tests、device/app info tests、redaction tests | 是，若输出 secret、token、private key path 或真实指纹 |
| L2 | 普通写、card expiration、storage、follow-up、业务状态变化 | 需要 scope、input validation、audit summary；失败必须 stable fail | scoped storage、input schema、permission decision、audit summary | storage/card/action tests、audit redaction | 是，若越 scope、静默成功或绕过 audit |
| L3 | 交易、支付、退款、订单确认、分享外发、外部交易跳转 | 默认 Prompt 或 Deny；无 consent proof 不执行 | PermissionDecision、ConsentGate、Host provider boundary、audit、idempotency/replay plan | consent bypass tests、provider unavailable fail-closed tests、audit tests | 是 |
| L4 | 手机号、地址、身份、位置、文件、媒体、相册、扫码、电话、生物识别、crypto private operation | 默认 Deny 或 Prompt；无 Host provider、allowlist、consent 和 redaction 不执行 | least-privilege Host provider、opaque handle、no raw data output、audit redaction、retention/export policy | L4 provider tests、redaction regression、file/path deny、location precision tests | 是 |

### 2.1 L3/L4 能力控制矩阵

下表是 Phase 3 的高风险验收口径。`当前 gate` 只记录已存在的自动化或手工检查；`Phase 3 required gate` 是后续 Step 必须补齐或升级的 release blocker。

| 能力 / Action | 风险 | 当前状态 | Owner | 当前 gate | Phase 3 required gate | 残余风险 |
|---|---|---|---|---|---|---|
| `wx.requestPayment`、`wx.requestVirtualPayment`、`wx.requestJointPayment` | L3 | `host-boundary`，默认 provider unavailable；mock payment 仅 demo/dev | `wx-compat`、`js-runtime-quickjs`、`consent-audit`、`dock-core`、Host adapter | `cargo test -p wx-compat high_risk`、`cargo test -p consent-audit -p dock-core consent` | Step 03-03 permission decision、Step 03-05 ConsentProof / persistent audit、Host provider conformance planned for Phase 4 | 真实支付 provider 和幂等策略未完成，production release 前阻塞 |
| merchant `api/call` 中的下单、支付、退款、确认类业务 API | L3 | Component action 回 Orchestrator；coffee payment 是 mock business API | `dock-core`、`component-runtime`、merchant adapter、`consent-audit` | `cargo test -p dock-core consent`、`cargo test -p dock-cli --test coffee_order_flow` | Step 03-03 must audit permission decision；Step 03-05 must persist redacted action audit | merchant API contract 和真实交易 provider 仍待 Phase 4/5 |
| `wx.shareAppMessage`、外部交易/详情页跳转 | L3/L4 | `host-boundary` 或 planned；Render IR 只记录 action | Host adapter、`component-runtime`、`consent-audit` | 组件矩阵手工 gate；unknown action fail closed requirement | Step 03-03 allowlist/permission decision；Phase 4 Host action conformance | 真实 Host renderer 未冻结，production release 前阻塞 |
| `wx.getPhoneNumber`、`wx.getRealtimePhoneNumber` | L4 | `getPhoneNumber` 已进入 high-risk provider boundary；实时手机号 planned | `wx-compat`、Host phone provider、`consent-audit` | `cargo test -p wx-compat high_risk`、`cargo test -p js-runtime-quickjs high_risk` | Step 03-03 deny-by-default policy；Step 03-05 proof digest、redacted persistent audit/export | 真实手机号 provider/conformance 仍待 Phase 4 |
| `wx.chooseAddress` | L4 | host-boundary；dev-only mock 只返回 opaque handle；address fixture 不含真实地址 | `wx-compat`、Host address provider、`component-runtime`、`consent-audit` | high-risk tests、address-form snapshot sensitive scan | Step 03-05 audit persistence/export redaction；provider conformance planned for Phase 4 | 真实地址最小字段和 retention policy 未完成 |
| `wx.getLocation`、`wx.getFuzzyLocation`、`wx.chooseLocation`、`wx.openLocation` | L4 | host-boundary；fixtures 只使用 opaque token / static preview | `wx-compat`、Host location/map provider、`component-runtime`、`consent-audit` | high-risk tests、location-map-preview snapshot sensitive scan | Step 03-03 allowlist/policy；Step 03-05 audit redaction；Phase 4 Host provider conformance | 精确位置 UI、最小精度和 retention 仍待 Host contract |
| `wx.chooseMedia`、`wx.chooseMessageFile`、`wx.uploadFile`、`wx.downloadFile`、`wx.openDocument`、`wx.getImageInfo` | L4 | choose 系列 host-boundary；upload/download/openDocument planned or host-boundary | `wx-compat`、`mcp-schema`、Host file/media provider、`anp-adapter` | high-risk tests、`format:image/file` validation、media-review snapshot sensitive scan | Step 03-03 file/network permission decision；Step 03-05 audit redaction; Phase 4 file provider conformance | 真实文件 provider、upload/download broker 和 encrypted storage 未完成 |
| `wx.makePhoneCall`、`wx.scanCode` | L4 | host-boundary；无 provider fail closed | `wx-compat`、Host provider、`consent-audit` | high-risk tests | Step 03-03 deny-by-default policy；Step 03-05 consent/audit proof | 真实 Host UI/conformance 仍待 Phase 4 |
| `wx.verifyPaymentPassword`、facial recognition、相册写入、生物识别、微信运动/发票等微信生态能力 | L4 | `unsupported-by-design` 或 planned；deterministic unsupported stub | `wx-compat`、Host adapter | `cargo test -p wx-compat unsupported`、Atomic API unsupported tests | Step 03-03 must keep unsupported fail closed; any future support requires Plan change before implementation | 默认不进入产品范围；若未来支持需单独合规设计 |
| Dynamic component `wx.request` | L3/L4 depending on URL/scope | `host-boundary`；声明 dynamic 后也走 injected `RequestBroker`；无 production transport | `component-runtime`、`wx-compat`、`anp-adapter`、Host adapter | `cargo test -p component-runtime dynamic`、`cargo test -p component-runtime sandbox`、`cargo test -p component-runtime snapshot_size` | Step 03-02 full sandbox/resource release gate 已补齐；Step 03-03 scheme/host/port/path/method/scope allowlist; Phase 4 transport/audit persistence | 无 Host registry allowlist 和 persistent request audit 前不得 production-ready |
| Dynamic component timers / background callbacks | L2-L3 | `host-boundary`；dynamic 才暴露受限 timer；expire/detach cleanup 已测 | `component-runtime`、Host adapter | `cargo test -p component-runtime dynamic`、`cargo test -p component-runtime sandbox` | Step 03-02 timer/resource exhaustion gate 已补齐；Phase 4 background scheduler/pause policy | 真实后台调度和 resource metrics 未完成 |
| `openDetailPage({ url })` / `preloadDetailPage({ url })` / related page query | L3/L4 | `openDetailPage` host-boundary，`preloadDetailPage` planned；manifest metadata redacted | `component-runtime`、Host renderer、`mcp-schema` | component matrix + metadata tests | Step 03-03 URL allowlist/permission decision; Phase 4 Host action conformance | Host URL canonicalization 和 external link UI 未完成 |

## 3. 资产清单

| 资产 | 保护目标 | 当前控制 | 证据 | 残余风险 |
|---|---|---|---|---|
| DID private key | 永不进入 JS、日志、Render IR、audit export、CLI JSON | Credential provider 只在 `anp-adapter` 使用；CLI E2E 断言不输出 private key path/material | [`challenge.rs`](../../crates/anp-adapter/src/challenge.rs)、[`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) | 本地文件权限和 secret store 仍是 Host/部署责任；Phase 3 需规划 secret store。 |
| DID document / user DID / agent DID / merchant DID | 正确绑定 session、audience、token scope | Challenge payload 绑定 DID、skill、session、audience、nonce、expiry | [`challenge.rs`](../../crates/anp-adapter/src/challenge.rs)、[`demo_api.rs`](../../crates/demo-server/tests/demo_api.rs) | DID resolver cache、trust anchor 和 rotation 尚未生产化。 |
| capability token | 只在 Host/request boundary 使用，短期有效、可按 scope 验证 | JWT claims 绑定 issuer/audience/merchant/user/agent/skill/session/scopes/jti；Debug 输出 redacted | [`token.rs`](../../crates/anp-adapter/src/token.rs)、[`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs) | revoke/logout、jti replay store、持久化 token lifecycle 仍为 Phase 3/4。 |
| Skill package | 路径边界、完整性、来源、版本 | canonicalize；拒绝绝对路径、`..`、包外路径 | [`resolver.rs`](../../crates/skill-loader/src/resolver.rs)、[`coffee_skill_load.rs`](../../crates/skill-loader/tests/coffee_skill_load.rs) | digest/signature、publisher DID、registry cache quarantine 尚未实现。 |
| Atomic API JS | 不能逃逸 sandbox、不能任意 require、不能直接 fetch/process/eval/WebSocket/timer | 受限 CommonJS；禁用 `fetch`、`process`、`eval`、`Function`、`WebSocket` 和 timer globals；memory/stack/timeout、Promise job drain、console size、result size 有默认限制和 focused tests | [`bridge.rs`](../../crates/js-runtime-quickjs/src/bridge.rs)、[`api_vm.rs`](../../crates/js-runtime-quickjs/src/api_vm.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | CI 自动 gate runner 和 resource metrics 仍待 Phase 6；默认上限放宽必须重新 Review threat model。 |
| Component JS | 不能任意网络、timer、WebSocket、Function escape；过期后不能继续事件 | Component VM 独立 context；默认禁用网络/timer/Function/eval；dynamic 组件只开放受限 `wx.request` 和 timer getter；native request bridge 不暴露给组件全局；expire 后清理 timer 并拒绝事件；snapshot output size 有默认限制 | [`component_vm.rs`](../../crates/component-runtime/src/component_vm.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs) | 生产 Host background scheduler、transport、request audit persistence 和 resource metrics 仍是 Phase 4/6。 |
| scoped storage | DID + merchant + Skill 隔离；拒绝敏感 key、超限 key/value、quota 溢出和非 JSON-safe value | `StorageScope`、in-memory storage 与 Atomic API JS bridge 测试 | [`storage.rs`](../../crates/wx-compat/src/storage.rs)、[`scoped_storage.rs`](../../crates/wx-compat/tests/scoped_storage.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | 当前非持久化；生产 storage encryption、migration、backend quota 和 cleanup 未实现。 |
| outbound network | 默认无任意出站，所有业务网络走 RequestBroker/allowlist | empty allowlist deny；scheme/host/port/path prefix/method/scope allowlist；component 默认 deny request；dynamic 组件 request 经 `RequestBroker` trait 且 JS auth headers fail closed；Atomic API `wx.request` 经 `wx-compat::RequestBroker` trait 的 loopback DID broker 拒绝非 loopback URL | [`signed_request.rs`](../../crates/anp-adapter/src/signed_request.rs)、[`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | 生产 Host transport、registry 配置来源和 request audit persistence 仍为 Phase 4/03-05。 |
| Host providers | 不能被 Skill 绕过 consent 调用 | Atomic API 高风险 provider boundary 默认 fail closed；未通过 consent 不执行 provider；组件 action 只记录 boundary | [`high_risk.rs`](../../crates/wx-compat/src/high_risk.rs)、[`high_risk_provider.rs`](../../crates/wx-compat/tests/high_risk_provider.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs)、[`wx-api-compatibility-matrix.md`](../architecture/wx-api-compatibility-matrix.md)、[`component-compatibility-matrix.md`](../architecture/component-compatibility-matrix.md) | 真实 Host provider contract、permission UI、provider audit persistence 和 conformance tests 未实现。 |
| consent proof | L3/L4 action 必须有人类授权或明确 policy decision | RiskPolicy 推断 L3/L4；Orchestrator 在 executor 前检查 consent | [`consent.rs`](../../crates/consent-audit/src/consent.rs)、[`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs) | 当前 provider 多为 mock/in-memory；生产 Host consent UI、policy version 和 proof retention 未实现。 |
| audit records | 可追踪、默认脱敏、不能泄露敏感参数 | audit parameter summary 使用 `redact_value`；`dock-core::AuditEvent` 记录脱敏 `permissionDecision`；测试覆盖 token/address/private redaction 和 Host deny/prompt decision audit | [`audit.rs`](../../crates/consent-audit/src/audit.rs)、[`payment_requires_consent.rs`](../../crates/consent-audit/tests/payment_requires_consent.rs)、[`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs) | 持久化 audit sink、retention、export approval 仍为 Step 03-05/Phase 4。 |
| Render IR / CardSpec | 不含 token/secret/private `_meta`，未知 action 不直接执行 | Component action 是数据；Render IR 已有 `schemaVersion`、snapshot gate 和 stable fallback reason；矩阵要求 Host unknown action fail closed | [`render_ir.rs`](../../crates/component-runtime/src/render_ir.rs)、[`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs)、[`component-compatibility-matrix.md`](../architecture/component-compatibility-matrix.md) | 生产 Host renderer conformance、unknown action handling 和 persistent card/audit policy 仍为 Phase 4。 |
| CLI/demo output | 只输出 mock/demo 结果和 redacted auth | coffee E2E 断言不含 token、Authorization、Signature、private key path/material | [`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) | 新 CLI 命令必须延续 redaction test。 |

## 4. 攻击者模型

| 攻击者 | 能力 | 主要风险 | 当前控制 | 缺口 / 下一阶段 |
|---|---|---|---|---|
| 恶意 Skill | 提交恶意 API JS / Component JS、尝试 require 包外文件、无限循环、读取 token、发起网络、伪造 action | sandbox escape、数据泄露、绕过 consent | QuickJS sandbox、包内 require、path canonicalize、RequestBroker allowlist、component 默认无网络/timer；Atomic API result/console/pending job 和 Component snapshot size 有上限；dynamic request/timer 有权限 gate、header deny、timer limit、expire cleanup；action 回 Orchestrator | package signature、Host transport policy、CI gate 自动化和 resource metrics。 |
| 被篡改 Skill 包 | 替换 `index.js`、`apis/*.js`、`components/*`、`mcp.json` | 供应链植入、权限声明漂移、组件路径逃逸 | manifest validation、path boundary、当前本地目录加载 | digest/signature、publisher DID、trusted publisher allowlist、registry quarantine。 |
| 恶意商家 Agent | 返回恶意 challenge/login response、错误 scope、诱导隐私/支付、返回恶意 `_meta` | token scope 混淆、用户隐私泄露、交易欺诈 | DID proof、audience/scope/token verification、ConsentGate、model-visible filtering、audit redaction | merchant trust policy、response size/type limits、payment provider contract。 |
| 网络中间人 | 篡改 challenge、business response、token header；重放 proof | credential theft、replay、scope bypass | HTTP Signature challenge proof、challenge TTL/audience/nonce、token signature/scope validation、allowlist | 生产 HTTPS requirement、resolver trust anchor、jti replay store。 |
| 恶意或误配置 Host provider | 过度返回手机号/地址/文件路径，错误执行 payment/phone/location | L4 隐私泄露、L3 交易绕过 | Host provider 仅为 boundary；矩阵要求 consent/audit/fail closed | Provider contract、least-privilege field shape、provider conformance tests。 |
| 日志 / audit 读取者 | 读取 stdout、server logs、audit export、CLI JSON | token、private key、手机号、地址泄露 | `redact_value`、CLI E2E redaction、security runbook checks | 持久化 audit encryption、export approval、retention policy。 |
| 本地文件系统攻击者 | 读取测试 DID 私钥、替换 Skill 文件、创建 symlink/path escape | credential exposure、package tamper | path canonicalize、private key fixture 不进输出 | file permission gate、secret store、package digest/signature。 |
| Host renderer / adapter bug | 执行未知 action、渲染未脱敏 payload、打开未授权 URL | 越权 UI action、外链跳转、隐私泄露 | Render IR 是数据；unknown action 必须 fail closed 的计划红线 | Host adapter contract、snapshot/conformance tests、URL canonicalization。 |

## 5. 控制矩阵

| 威胁 / 失败模式 | 当前控制 | 测试 / 证据 | Release gate 状态 |
|---|---|---|---|
| JS `eval` / `Function` / constructor escape | Atomic API VM 和 Component VM 禁用相关全局与 prototype constructor；Atomic API VM 禁用 WebSocket/timer globals 并限制 Promise job drain、console 和 result size；Component VM 限制 snapshot output size，dynamic 例外仍由 capability profile 控制 | [`bridge.rs`](../../crates/js-runtime-quickjs/src/bridge.rs)、[`api_vm.rs`](../../crates/js-runtime-quickjs/src/api_vm.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs)、[`component_vm.rs`](../../crates/component-runtime/src/component_vm.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs) | Step 03-02 本地 required gate：`cargo test -p js-runtime-quickjs sandbox`、`cargo test -p js-runtime-quickjs limit`、`cargo test -p component-runtime sandbox`、`cargo test -p component-runtime dynamic`；CI 自动化仍待 Phase 6。 |
| CommonJS / component path escape | `skill-loader` 拒绝绝对路径、`..`、包外 canonical path | [`coffee_skill_load.rs`](../../crates/skill-loader/tests/coffee_skill_load.rs) | 当前 gate：workspace tests。 |
| 任意网络出站 | RequestBroker allowlist deny by default；allowlist 支持 scheme/host/port/path prefix/method/scope；component profile 默认 deny request；dynamic component request 只通过 injected broker 且默认 `UnsupportedRequestBroker` fail closed；Atomic API `wx.request` 非 loopback URL fail closed | [`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | Step 03-03 本地 gate 已覆盖 scheme/host/port/path/method/scope mismatch；Phase 4 替换 demo-only loopback transport 并接入 Host audit persistence。 |
| JS 覆盖 `Authorization` | API bridge 和 Component VM dynamic request 对 `Authorization`、`Signature`、`Signature-Input`、`Cookie` fail closed，不出站 | [`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs)、[`wx-api-compatibility-matrix.md`](../architecture/wx-api-compatibility-matrix.md) | 当前 gate：focused VM tests + workspace tests。 |
| token scope / audience mismatch | capability claims 和 verifier 检查 scope/audience/merchant/skill/session | [`token.rs`](../../crates/anp-adapter/src/token.rs)、[`demo_api.rs`](../../crates/demo-server/tests/demo_api.rs) | Step 03-04 必须补齐 refresh/revoke/logout、jti replay、resolver trust anchor 和 redaction gates。 |
| challenge replay / wrong signer | challenge proof 绑定 nonce/audience/expiry/user DID；demo-server 消耗 challenge | [`challenge.rs`](../../crates/anp-adapter/src/challenge.rs)、[`demo_api.rs`](../../crates/demo-server/tests/demo_api.rs) | Step 03-04 必须补齐 challenge nonce 一次性、TTL、audience/method/url binding、DID document binding 和 replay tests。 |
| L3/L4 consent bypass | Orchestrator 在 executor 前执行 consent；denied/required fail closed | [`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs)、[`payment_requires_consent.rs`](../../crates/consent-audit/tests/payment_requires_consent.rs) | Step 03-05 必须补齐 Host consent adapter、ConsentProof policy version/prompt digest/parameter digest 和 persistent audit gate。 |
| 高风险 `wx.*` API 绕过 Host boundary | Atomic API `getPhoneNumber`、`chooseAddress`、location、media/file、payment、scan、phone call 进入 `wx-compat` provider boundary；无 provider 为 `provider_unavailable`，未 consent 为 `consent_required`，本地文件路径被拒绝且不回显 | [`high_risk.rs`](../../crates/wx-compat/src/high_risk.rs)、[`high_risk_provider.rs`](../../crates/wx-compat/tests/high_risk_provider.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | 当前 gate：focused high-risk tests；真实 provider conformance planned。 |
| raw token / signature / private key output | Debug redaction、audit redaction、CLI E2E redaction | [`audit.rs`](../../crates/consent-audit/src/audit.rs)、[`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) | 当前 gate：workspace tests + redaction search review。 |
| `_meta` 进入模型可见输出 | `AtomicApiResult::model_visible()` 隔离 `_meta` | [`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs)、[`current-capability-baseline.md`](../architecture/current-capability-baseline.md) | 当前 gate：workspace tests。 |
| Render IR / Host unknown action 执行 | Render IR action 是数据；矩阵要求 unknown action fail closed；Render IR snapshot 已进入当前 gate | [`component-compatibility-matrix.md`](../architecture/component-compatibility-matrix.md)、[`render_ir_snapshots.rs`](../../crates/component-runtime/tests/render_ir_snapshots.rs) | 当前 gate：snapshot tests；Phase 4 仍需 Host renderer conformance。 |
| unsupported API 静默成功 | deterministic unsupported registry 和 unknown fallback 已实现，未支持 API 不应静默 no-op 成功 | [`wx-api-compatibility-matrix.md`](../architecture/wx-api-compatibility-matrix.md)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | 当前 gate：`cargo test -p wx-compat unsupported`、`cargo test -p js-runtime-quickjs unsupported`。 |
| Permission drift | `wx-compat::PermissionPolicyEngine` 统一 `Allow` / `Deny` / `Prompt` / `MockAllowed(dev_only)`，Host deny override 最高优先级，Host allow 不能替代 manifest/meta/dynamic 声明，mock 只能 dev/headless profile | [`permissions.rs`](../../crates/wx-compat/src/permissions.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs) | Step 03-03 本地 gate 已覆盖未声明 deny、Host deny、Host allow 不声明敏感权限、mock dev-only、dynamic scope 和 decision audit；生产 Host policy UI/adapter contract 仍待 Phase 4。 |
| dynamic timer 资源耗尽 | Component VM 只在声明 dynamic 后暴露 timer getter，限制 timer 数量，`clearTimeout` / `clearInterval` 生效，expire/detach 清理 pending timers | [`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs)、[`component-compatibility-matrix.md`](../architecture/component-compatibility-matrix.md) | 当前 gate：`cargo test -p component-runtime dynamic`；Host background pause/scheduler 仍为 Phase 4。 |
| audit export 泄露 | `redact_value` 当前覆盖敏感 key 和长字符串截断 | [`payment_requires_consent.rs`](../../crates/consent-audit/tests/payment_requires_consent.rs) | Step 03-05 必须补齐 persistent audit sink、retention、query/export redaction gate。 |
| package tamper | 当前只有 path/manifest validation | [`coffee_skill_load.rs`](../../crates/skill-loader/tests/coffee_skill_load.rs) | Step 03-06 必须补齐 digest/signature/publisher DID、trusted publisher allowlist 和 quarantine gate。 |

## 6. 安全红线

以下任一情况是 release blocker：

- 任何 Skill API 或 Component 可绕过 RequestBroker / allowlist 直接网络出站。
- raw capability token、Authorization、HTTP Signature、DID private key path/material、merchant secret、手机号、地址、文件内容进入 CLI JSON、日志、audit export、Render IR 或模型可见输出。
- L3/L4 API、支付、手机号、地址、位置、文件、电话或外部链接在无 consent proof / Host provider policy 下执行。
- package path、component path、CommonJS require 可逃逸 Skill root。
- sandbox escape regression 失败，包括 `eval`、`Function`、prototype constructor、`process`、`fetch`、`WebSocket`、component timer 默认开放。
- dynamic component request/timer 绕过 `scope.dynamic`、RequestBroker、auth-header deny、timer limit 或 expire cleanup。
- unsupported API 或 Host unknown action 静默成功。
- demo-only/mock provider 被文档或配置写成 production-ready。

## 7. 残余风险

| 风险 | 影响 | 概率 | 当前控制 | 残余风险 | Owner | Review date | Release blocker |
|---|---|---|---|---|---|---|---|
| `wx.request` 仍使用 demo-only localhost transport | 若误用于生产，只能访问 loopback demo 服务，缺少 Host registry 配置来源、persistent audit 和部署级 transport 策略 | 中 | QuickJS 已走 `wx-compat::RequestBroker` trait 的本地 DID broker；`anp-adapter` 已有 scheme/host/port/path/method/scope allowlist；API 矩阵和 release gates 仍标为 demo-only，禁止 production release | Phase 4 需接入 production Host RequestBroker transport、registry 配置来源和 audit persistence | `js-runtime-quickjs`、`wx-compat`、`anp-adapter`、Host adapter | Phase 4 | 是，production release 前 |
| Component dynamic request/timer 仍是 headless 最小 gate | 若误当作完整 Host dynamic runtime，可能缺少生产网络 transport、后台暂停/恢复和 request audit 持久化 | 中 | 默认 deny；声明 dynamic 后才注入受限 `wx.request`/timer；RequestBroker 默认 unsupported；auth headers fail closed；timer limit、clear、expire cleanup、snapshot size limit 和 redaction tests 已覆盖 | Phase 4 接入 Host transport、background lifecycle 和 persistent audit；Phase 6 补 CI gate runner 和 resource metrics | `component-runtime`、`wx-compat`、Host adapter | Phase 4/6 | 是，production release 前 |
| 真实 Host consent UI 未实现 | L3/L4 只能在 mock/headless 流程验证 | 中 | ConsentGate trait、mock provider、audit redaction tests；Atomic API high-risk boundary 默认 fail closed，不会用 mock 冒充 production | Phase 3/4 需 Host consent adapter 和 provider conformance tests | `dock-core`、`consent-audit`、`wx-compat`、Host adapter | Phase 3 | 是，production release 前 |
| audit 仍为 in-memory/mock 为主 | 审计不可持久化，不满足线上追溯 | 中 | redacted record model 与 tests | Phase 3/4 需 persistent audit sink 和 retention | `consent-audit` | Phase 3 | 是，production release 前 |
| Skill 包签名未实现 | 本地包被篡改时只能靠路径/manifest 校验 | 中 | path canonicalization、manifest validation | Phase 3 需 digest/signature/publisher DID | `skill-loader` | Phase 3 | 是，远端 registry 前 |
| 生产 Host renderer conformance 未完成 | Host adapter 可能错误执行未知 action、忽略 fallback 或渲染未脱敏 payload | 中 | Render IR 已版本化，snapshot gate 已覆盖当前 headless output；组件矩阵要求 unknown action fail closed | Phase 4 需 Host renderer/action conformance tests 和 stable Host adapter contract | `component-runtime`、Host adapter | Phase 4 | 是，production Host release 前 |
| DID resolver / token revoke 未生产化 | rotation、revocation 和 trust anchor 不完整 | 中 | challenge proof、JWT scope、TTL、tests | Step 03-04 需 resolver cache、trust anchor、revoke/logout、jti replay；Phase 4 承接 secret store 和持久化 token cache | `anp-adapter` | Phase 3 | 是，production release 前 |
| 生产 Host permission UI / policy 配置来源未完成 | 统一 decision 已有本地 engine 和 audit summary，但真实 Host policy UI、配置加载、provider conformance 仍未冻结 | 中 | Step 03-03 本地 gate 覆盖 `Allow` / `Deny` / `Prompt` / `MockAllowed(dev_only)`、Host deny override、未声明 deny、mock dev/headless、dynamic scope 和 decision audit | Phase 4 需 Host adapter contract、policy UI/config、provider conformance；Step 03-05 需 persistent audit | `wx-compat`、`dock-core`、`anp-adapter`、Host adapter | Phase 4 | 是，production release 前 |

## 8. Review 要求

安全敏感改动必须额外 review 下列项目：

- DID credential provider、challenge proof、signed request、token claims 和 cache scope。
- URL allowlist、RequestBroker、`Authorization` 处理和 network retry。
- QuickJS sandbox globals、CommonJS resolver、Component VM dynamic request/timer。
- ConsentGate enforcement order、RiskPolicy、ConsentProof 和 audit record。
- CLI/demo/server output redaction。
- Render IR action、Host provider contract、fallback reason 和 unknown action 行为。
- Skill package loading、path canonicalization、future digest/signature。

本文件是 Phase 3 详细安全实现的输入。改变公开契约、安全边界、验证策略或 release blocker 必须先更新 roadmap 的 Plan 变更记录。
