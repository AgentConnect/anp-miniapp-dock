# Threat Model 与安全基线

> 状态：Phase 0 安全模型初版
> 日期：2026-06-12
> 范围：覆盖 `anp-miniapp-dock` 从 Skill 包加载、QuickJS VM、wx Compatibility Layer、ANP DID / capability token、ConsentGate、audit、Render IR 到 Host provider 的主要威胁、控制措施、测试证据和残余风险。
> 上游计划：[`../plan/production-readiness-roadmap.md`](../plan/production-readiness-roadmap.md) Step 00-04。

## 1. 安全目标

`anp-miniapp-dock` 的安全目标是让 MiniApp MCP Skill 能在 Agent 对话中执行，同时不让 Skill 获得宿主私钥、capability token、任意网络、任意文件、真实支付、隐私数据或未授权 Host 能力。

核心原则：

- Skill JS 默认不可信。
- 高风险能力默认 fail closed。
- ANP DID、capability token、HTTP Signature 和 Host provider 均在容器/Host 边界内，不暴露给 Skill。
- `api/call`、支付、隐私、文件、位置、电话和外部链接必须经过 Orchestrator、Host provider、ConsentGate 和 audit。
- CLI JSON、日志、audit export、Render IR 和模型可见输出不得包含 token、Authorization、HTTP Signature、private key path、手机号、地址、文件内容或未脱敏位置。

## 2. 资产清单

| 资产 | 保护目标 | 当前控制 | 证据 | 残余风险 |
|---|---|---|---|---|
| DID private key | 永不进入 JS、日志、Render IR、audit export、CLI JSON | Credential provider 只在 `anp-adapter` 使用；CLI E2E 断言不输出 private key path/material | [`challenge.rs`](../../crates/anp-adapter/src/challenge.rs)、[`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) | 本地文件权限和 secret store 仍是 Host/部署责任；Phase 3 需规划 secret store。 |
| DID document / user DID / agent DID / merchant DID | 正确绑定 session、audience、token scope | Challenge payload 绑定 DID、skill、session、audience、nonce、expiry | [`challenge.rs`](../../crates/anp-adapter/src/challenge.rs)、[`demo_api.rs`](../../crates/demo-server/tests/demo_api.rs) | DID resolver cache、trust anchor 和 rotation 尚未生产化。 |
| capability token | 只在 Host/request boundary 使用，短期有效、可按 scope 验证 | JWT claims 绑定 issuer/audience/merchant/user/agent/skill/session/scopes/jti；Debug 输出 redacted | [`token.rs`](../../crates/anp-adapter/src/token.rs)、[`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs) | revoke/logout、jti replay store、持久化 token lifecycle 仍为 Phase 3/4。 |
| Skill package | 路径边界、完整性、来源、版本 | canonicalize；拒绝绝对路径、`..`、包外路径 | [`resolver.rs`](../../crates/skill-loader/src/resolver.rs)、[`coffee_skill_load.rs`](../../crates/skill-loader/tests/coffee_skill_load.rs) | digest/signature、publisher DID、registry cache quarantine 尚未实现。 |
| Atomic API JS | 不能逃逸 sandbox、不能任意 require、不能直接 fetch/process/eval | 受限 CommonJS；禁用 `fetch`、`process`、`eval`、`Function`；超时 | [`bridge.rs`](../../crates/js-runtime-quickjs/src/bridge.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | 生产级 CPU/memory/result size/console size gate 需 Phase 3 强化。 |
| Component JS | 不能任意网络、timer、WebSocket、Function escape；过期后不能继续事件 | Component VM 独立 context；禁用网络/timer/Function/eval；expire 后事件失败 | [`component_vm.rs`](../../crates/component-runtime/src/component_vm.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs) | dynamic request/timer 开放前需资源限制和 cleanup gate。 |
| scoped storage | DID + merchant + Skill 隔离；拒绝敏感 key、超限 key/value、quota 溢出和非 JSON-safe value | `StorageScope`、in-memory storage 与 Atomic API JS bridge 测试 | [`storage.rs`](../../crates/wx-compat/src/storage.rs)、[`scoped_storage.rs`](../../crates/wx-compat/tests/scoped_storage.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | 当前非持久化；生产 storage encryption、migration、backend quota 和 cleanup 未实现。 |
| outbound network | 默认无任意出站，所有业务网络走 RequestBroker/allowlist | empty allowlist deny；authority allowlist；component 默认 deny request；Atomic API `wx.request` 经 `wx-compat::RequestBroker` trait 的 loopback DID broker 拒绝非 loopback URL | [`signed_request.rs`](../../crates/anp-adapter/src/signed_request.rs)、[`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | 生产 Host transport、registry allowlist 和 request audit persistence 仍为 Phase 4。 |
| Host providers | 不能被 Skill 绕过 consent 调用 | 高风险 API 和组件 action 只记录 boundary；未配置 provider fail closed 或 unsupported | [`wx-api-compatibility-matrix.md`](../architecture/wx-api-compatibility-matrix.md)、[`component-compatibility-matrix.md`](../architecture/component-compatibility-matrix.md) | 真实 Host provider contract、permission UI 和 provider audit 未实现。 |
| consent proof | L3/L4 action 必须有人类授权或明确 policy decision | RiskPolicy 推断 L3/L4；Orchestrator 在 executor 前检查 consent | [`consent.rs`](../../crates/consent-audit/src/consent.rs)、[`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs) | 当前 provider 多为 mock/in-memory；生产 Host consent UI、policy version 和 proof retention 未实现。 |
| audit records | 可追踪、默认脱敏、不能泄露敏感参数 | audit parameter summary 使用 `redact_value`；测试覆盖 token/address/private redaction | [`audit.rs`](../../crates/consent-audit/src/audit.rs)、[`payment_requires_consent.rs`](../../crates/consent-audit/tests/payment_requires_consent.rs) | 持久化 audit sink、retention、export approval 仍为 Phase 3/4。 |
| Render IR / CardSpec | 不含 token/secret/private `_meta`，未知 action 不直接执行 | Component action 是数据；fallback 保留；矩阵要求 Host unknown action fail closed | [`render_ir.rs`](../../crates/component-runtime/src/render_ir.rs)、[`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs)、[`component-compatibility-matrix.md`](../architecture/component-compatibility-matrix.md) | schemaVersion、snapshot、fallback reason enum 仍为 Phase 2。 |
| CLI/demo output | 只输出 mock/demo 结果和 redacted auth | coffee E2E 断言不含 token、Authorization、Signature、private key path/material | [`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) | 新 CLI 命令必须延续 redaction test。 |

## 3. 攻击者模型

| 攻击者 | 能力 | 主要风险 | 当前控制 | 缺口 / 下一阶段 |
|---|---|---|---|---|
| 恶意 Skill | 提交恶意 API JS / Component JS、尝试 require 包外文件、无限循环、读取 token、发起网络、伪造 action | sandbox escape、数据泄露、绕过 consent | QuickJS sandbox、包内 require、path canonicalize、RequestBroker allowlist、component 默认无网络/timer、action 回 Orchestrator | package signature、resource limit、unsupported stub 全覆盖、dynamic resource cleanup。 |
| 被篡改 Skill 包 | 替换 `index.js`、`apis/*.js`、`components/*`、`mcp.json` | 供应链植入、权限声明漂移、组件路径逃逸 | manifest validation、path boundary、当前本地目录加载 | digest/signature、publisher DID、trusted publisher allowlist、registry quarantine。 |
| 恶意商家 Agent | 返回恶意 challenge/login response、错误 scope、诱导隐私/支付、返回恶意 `_meta` | token scope 混淆、用户隐私泄露、交易欺诈 | DID proof、audience/scope/token verification、ConsentGate、model-visible filtering、audit redaction | merchant trust policy、response size/type limits、payment provider contract。 |
| 网络中间人 | 篡改 challenge、business response、token header；重放 proof | credential theft、replay、scope bypass | HTTP Signature challenge proof、challenge TTL/audience/nonce、token signature/scope validation、allowlist | 生产 HTTPS requirement、resolver trust anchor、jti replay store。 |
| 恶意或误配置 Host provider | 过度返回手机号/地址/文件路径，错误执行 payment/phone/location | L4 隐私泄露、L3 交易绕过 | Host provider 仅为 boundary；矩阵要求 consent/audit/fail closed | Provider contract、least-privilege field shape、provider conformance tests。 |
| 日志 / audit 读取者 | 读取 stdout、server logs、audit export、CLI JSON | token、private key、手机号、地址泄露 | `redact_value`、CLI E2E redaction、security runbook checks | 持久化 audit encryption、export approval、retention policy。 |
| 本地文件系统攻击者 | 读取测试 DID 私钥、替换 Skill 文件、创建 symlink/path escape | credential exposure、package tamper | path canonicalize、private key fixture 不进输出 | file permission gate、secret store、package digest/signature。 |
| Host renderer / adapter bug | 执行未知 action、渲染未脱敏 payload、打开未授权 URL | 越权 UI action、外链跳转、隐私泄露 | Render IR 是数据；unknown action 必须 fail closed 的计划红线 | Host adapter contract、snapshot/conformance tests、URL canonicalization。 |

## 4. 控制矩阵

| 威胁 / 失败模式 | 当前控制 | 测试 / 证据 | Release gate 状态 |
|---|---|---|---|
| JS `eval` / `Function` / constructor escape | Atomic API VM 和 Component VM 禁用相关全局与 prototype constructor | [`bridge.rs`](../../crates/js-runtime-quickjs/src/bridge.rs)、[`component_lifecycle.rs`](../../crates/component-runtime/tests/component_lifecycle.rs) | 当前 gate：`cargo test --workspace`；Phase 3 需新增专项 sandbox regression gate。 |
| CommonJS / component path escape | `skill-loader` 拒绝绝对路径、`..`、包外 canonical path | [`coffee_skill_load.rs`](../../crates/skill-loader/tests/coffee_skill_load.rs) | 当前 gate：workspace tests。 |
| 任意网络出站 | RequestBroker allowlist deny by default；component profile 默认 deny request；Atomic API `wx.request` 非 loopback URL fail closed | [`capability_token_scope.rs`](../../crates/anp-adapter/tests/capability_token_scope.rs)、[`component_permissions.rs`](../../crates/wx-compat/tests/component_permissions.rs)、[`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs) | 当前 gate：workspace tests；Phase 4 需替换 demo-only loopback transport。 |
| JS 覆盖 `Authorization` | API bridge 对 `Authorization`、`Signature`、`Signature-Input`、`Cookie` fail closed，不出站 | [`middleware_chain.rs`](../../crates/js-runtime-quickjs/tests/middleware_chain.rs)、[`wx-api-compatibility-matrix.md`](../architecture/wx-api-compatibility-matrix.md) | 当前 gate：focused VM tests + workspace tests。 |
| token scope / audience mismatch | capability claims 和 verifier 检查 scope/audience/merchant/skill/session | [`token.rs`](../../crates/anp-adapter/src/token.rs)、[`demo_api.rs`](../../crates/demo-server/tests/demo_api.rs) | 当前 gate：workspace tests；revoke/replay 是 planned。 |
| challenge replay / wrong signer | challenge proof 绑定 nonce/audience/expiry/user DID；demo-server 消耗 challenge | [`challenge.rs`](../../crates/anp-adapter/src/challenge.rs)、[`demo_api.rs`](../../crates/demo-server/tests/demo_api.rs) | 当前 gate：workspace tests；jti replay store 是 planned。 |
| L3/L4 consent bypass | Orchestrator 在 executor 前执行 consent；denied/required fail closed | [`api_call_flow.rs`](../../crates/dock-core/tests/api_call_flow.rs)、[`payment_requires_consent.rs`](../../crates/consent-audit/tests/payment_requires_consent.rs) | 当前 gate：workspace tests。 |
| raw token / signature / private key output | Debug redaction、audit redaction、CLI E2E redaction | [`audit.rs`](../../crates/consent-audit/src/audit.rs)、[`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs) | 当前 gate：workspace tests + redaction search review。 |
| `_meta` 进入模型可见输出 | `AtomicApiResult::model_visible()` 隔离 `_meta` | [`coffee_order_flow.rs`](../../crates/dock-cli/tests/coffee_order_flow.rs)、[`current-capability-baseline.md`](../architecture/current-capability-baseline.md) | 当前 gate：workspace tests。 |
| Render IR / Host unknown action 执行 | Render IR action 是数据；矩阵要求 unknown action fail closed | [`component-compatibility-matrix.md`](../architecture/component-compatibility-matrix.md) | Planned gate：Render IR snapshot / Host conformance after Phase 2/4。 |
| unsupported API 静默成功 | API 矩阵要求 deterministic unsupported stub | [`wx-api-compatibility-matrix.md`](../architecture/wx-api-compatibility-matrix.md) | Planned gate：Step 01-01/Phase 1 后启用。 |
| audit export 泄露 | `redact_value` 当前覆盖敏感 key 和长字符串截断 | [`payment_requires_consent.rs`](../../crates/consent-audit/tests/payment_requires_consent.rs) | 当前 gate：workspace tests；persistent/export gate planned。 |
| package tamper | 当前只有 path/manifest validation | [`coffee_skill_load.rs`](../../crates/skill-loader/tests/coffee_skill_load.rs) | Planned gate：digest/signature/publisher DID after Phase 3。 |

## 5. 安全红线

以下任一情况是 release blocker：

- 任何 Skill API 或 Component 可绕过 RequestBroker / allowlist 直接网络出站。
- raw capability token、Authorization、HTTP Signature、DID private key path/material、merchant secret、手机号、地址、文件内容进入 CLI JSON、日志、audit export、Render IR 或模型可见输出。
- L3/L4 API、支付、手机号、地址、位置、文件、电话或外部链接在无 consent proof / Host provider policy 下执行。
- package path、component path、CommonJS require 可逃逸 Skill root。
- sandbox escape regression 失败，包括 `eval`、`Function`、prototype constructor、`process`、`fetch`、`WebSocket`、component timer 默认开放。
- unsupported API 或 Host unknown action 静默成功。
- demo-only/mock provider 被文档或配置写成 production-ready。

## 6. 残余风险

| 风险 | 影响 | 概率 | 当前控制 | 残余风险 | Owner | Review date | Release blocker |
|---|---|---|---|---|---|---|---|
| `wx.request` 仍使用 demo-only localhost transport | 若误用于生产，只能访问 loopback demo 服务，缺少 Host registry allowlist、persistent audit 和部署级 transport 策略 | 中 | QuickJS 已走 `wx-compat::RequestBroker` trait 的本地 DID broker；API 矩阵和 release gates 仍标为 demo-only，禁止 production release | Phase 4 需接入 production Host RequestBroker transport、registry allowlist 和 audit persistence | `js-runtime-quickjs`、`wx-compat`、`anp-adapter`、Host adapter | Phase 4 | 是，production release 前 |
| 真实 Host consent UI 未实现 | L3/L4 只能在 mock/headless 流程验证 | 中 | ConsentGate trait、mock provider、audit redaction tests | Phase 3/4 需 Host consent adapter | `dock-core`、`consent-audit`、Host adapter | Phase 3 | 是，production release 前 |
| audit 仍为 in-memory/mock 为主 | 审计不可持久化，不满足线上追溯 | 中 | redacted record model 与 tests | Phase 3/4 需 persistent audit sink 和 retention | `consent-audit` | Phase 3 | 是，production release 前 |
| Skill 包签名未实现 | 本地包被篡改时只能靠路径/manifest 校验 | 中 | path canonicalization、manifest validation | Phase 3 需 digest/signature/publisher DID | `skill-loader` | Phase 3 | 是，远端 registry 前 |
| Render IR 未版本化 | Host adapter 兼容性和 snapshot drift 难以管理 | 中 | Component matrix 与 Phase 2 子文档记录 schemaVersion 目标 | Phase 2 需 schemaVersion 和 golden snapshots | `component-runtime`、Host adapter | Phase 2 | 否，P0 docs release 可接受 |
| DID resolver / token revoke 未生产化 | rotation、revocation 和 trust anchor 不完整 | 中 | challenge proof、JWT scope、TTL、tests | Phase 3/4 需 resolver cache、revoke/logout、jti replay | `anp-adapter` | Phase 3 | 是，production release 前 |

## 7. Review 要求

安全敏感改动必须额外 review 下列项目：

- DID credential provider、challenge proof、signed request、token claims 和 cache scope。
- URL allowlist、RequestBroker、`Authorization` 处理和 network retry。
- QuickJS sandbox globals、CommonJS resolver、Component VM dynamic request/timer。
- ConsentGate enforcement order、RiskPolicy、ConsentProof 和 audit record。
- CLI/demo/server output redaction。
- Render IR action、Host provider contract、fallback reason 和 unknown action 行为。
- Skill package loading、path canonicalization、future digest/signature。

本文件是 Phase 3 详细安全实现的输入。改变公开契约、安全边界、验证策略或 release blocker 必须先更新 roadmap 的 Plan 变更记录。
