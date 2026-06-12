# anp-miniapp-dock 产品化迭代总体计划

> 状态：计划文档
> 日期：2026-06-11
> 范围：仅补齐后续开发计划文档，不在本步骤开发代码。
> 依据：`docs/architecture/`、`docs/weichat-miniapp-mcp-protocol/weichat-miniapp-mcp.txt`、`docs/runbook/`、`docs/plan/did-wx-python-integration-plan.md`、当前 Cargo workspace 与测试现状。

## 1. 目标与边界

本计划的目标是把当前 coffee Demo 原型逐步演进为可线上使用的稳定 **Agentic MiniApp Container**。核心原则保持不变：

1. **智能体原生**：容器服务于 Agent 对话场景中的 Skill 调用，而不是复刻完整微信小程序运行时。
2. **MCP 接口兼容优先**：尽量对齐小程序 MCP 的 `SKILL.md`、`mcp.json`、原子接口、原子组件、`wx.modelContext` 与关键 `wx.*` 能力。
3. **ANP DID 替换底层身份与网络**：登录、鉴权、签名、请求、token、商家 Agent 访问使用 ANP DID / ANP Rust SDK 能力承载。
4. **核心能力补齐，不做完整 UI**：组件运行时要对齐原子组件契约与 Render IR 输出，但不进入完整页面路由、TabBar、半屏页面、小程序宿主 UI 复刻。
5. **安全默认开启**：沙箱隔离、权限声明、allowlist、capability token、human authorization、审计与脱敏都必须成为默认路径。

### 1.1 当前已具备的基线能力

当前代码已经不是空 Demo，已具备 P0/P0.5 基线：

- `mcp-schema`：`mcp.json.apis[]`、`components[]`、`inputSchema`、`outputSchema`、`AtomicApiResult`、模型可见字段与 `_meta` 隔离的基础模型和校验。
- `skill-loader`：加载 `SKILL.md`、`mcp.json`、`index.js`、`apis/*.js` 与组件包，阻断路径穿越和跨包 require。
- `js-runtime-quickjs`：QuickJS 原子接口 VM、受限 CommonJS、`createSkill`、`registerAPI`、`skill.use`、middleware、超时、日志、禁用 `fetch` / `process` / `eval` / `Function` 等全局逃逸入口；当前已注入 demo 级 `wx.login` / `wx.request` localhost DID 登录桥。
- `component-runtime`：`Component({})` 子集、`data` / `properties` / `methods`、`created` / `attached` / `detached`、`setData`、`Input` / `Result` / `Expire`、`sendFollowUpMessage`、`api/call`、`expirePreviousCards` / `expireAllCards`、tap / image load / image error、WXML/WXSS 子集与 Render IR JSON。
- `wx-compat`：capability profile、request broker trait、scoped storage、model context、card expiration、device/app info helper。
- `anp-adapter`：DID credential provider、HTTP signature helper、challenge proof、scoped capability token、token cache、allowlist request broker。
- `consent-audit`：风险分级、mock consent provider、consent proof、审计记录与敏感字段脱敏。
- `card-spec`：结构化 fallback card。
- `demo-server` / `dock-cli`：coffee merchant Agent demo、真实 DID challenge proof、scoped capability token、coffee order E2E、组件 action 驱动、卡片过期验证。
- `examples/coffee-fastapi-server` 与 `mac-app/`：用于演示 Python 远端服务和 Mac Chatbot host 的辅助链路。

### 1.2 仍需补齐的产品化缺口

为了达到线上稳定产品，后续不能只扩展 demo 业务，而要补齐以下系统能力：

- **接口对齐缺口**：`wx.modelContext` 与微信标准 `wx.*` API 还没有完整的能力分层、JS 注入、错误语义、callback / Promise 兼容、权限声明和测试矩阵。
- **组件对齐缺口**：当前组件运行时覆盖交易型卡片 P0 子集，尚未系统支持小程序 MCP 的组件支持列表、`relatedPage`、`expirable`、`openDetailPage` fallback、`Overflow`、动态组件、表单类组件、map/canvas 静态能力和更完整 WXML/WXSS。
- **安全缺口**：需要正式 threat model、生产级沙箱资源限制、包完整性/签名、权限策略引擎、token 轮换/撤销、真实审计落盘、敏感输出审计和供应链治理。
- **运行时产品缺口**：缺少稳定公共 API、持久化 session/token/storage/audit、Skill 获取与版本管理、Host 接入协议、生产部署形态、观测指标和发布门禁。
- **开发者生态缺口**：需要导入/校验/迁移工具、兼容性报告、golden fixtures、示例 Skill 集、文档和 release certification。

## 2. 三层计划总览

本计划按三个层级组织：

1. **总体计划**：从 Demo 原型到线上容器的阶段路线图。
2. **每个阶段的整体计划**：说明阶段目标、主要工作包、产出物和验收门槛。
3. **每个阶段中的具体细分小阶段及实施方案**：拆成可执行的开发迭代单元。

### 2.1 详细阶段文档

每个阶段的可执行开发计划已展开到独立目录：[`production-readiness/`](production-readiness/README.md)。后续开发应以该目录中的阶段文档作为 issue 拆分和验收依据。

| 阶段 | 详细计划 |
|---|---|
| Phase 0 | [基线冻结与产品化门槛](production-readiness/phase-0-baseline-and-gates.md) |
| Phase 1 | [接口对齐与 wx Capability Broker](production-readiness/phase-1-wx-capability-broker.md) |
| Phase 2 | [组件运行时对齐](production-readiness/phase-2-component-runtime-alignment.md) |
| Phase 3 | [安全增强与可信执行](production-readiness/phase-3-security-hardening.md) |
| Phase 4 | [生产运行时与 Host 接入](production-readiness/phase-4-runtime-host-integration.md) |
| Phase 5 | [开发者体验与生态兼容](production-readiness/phase-5-developer-experience.md) |
| Phase 6 | [观测、性能与发布运营](production-readiness/phase-6-observability-release.md) |

### 2.2 总体路线图

| 阶段 | 名称 | 核心目标 | 主要产出 | 完成标志 |
|---|---|---|---|---|
| Phase 0 | 基线冻结与产品化门槛 | 把当前 P0.5 能力变成可追踪基线，建立兼容矩阵和 release gates | 兼容缺口台账、测试基线、里程碑拆分 | 所有后续开发都有明确 scope、DoD 和验收命令 |
| Phase 1 | 接口对齐与 wx Capability Broker | 补齐原子接口环境中关键 `wx.modelContext` / `wx.*` 能力，对齐小程序 MCP 接口语义 | wx API 兼容层、权限映射、JS bridge、测试矩阵 | 核心交易型 Skill 可不改或少改运行 |
| Phase 2 | 组件运行时对齐 | 补齐小程序 MCP 原子组件核心能力，保持 Render IR 主线 | 组件兼容矩阵、更多 WXML/WXSS/内置组件、动态组件受控能力 | 多个真实交易型组件 fixture 通过快照和交互测试 |
| Phase 3 | 安全增强与可信执行 | 让容器默认满足线上安全边界 | threat model、沙箱加固、权限策略、token 生命周期、审计落盘、包签名 | 安全审计 checklist 全部通过，高风险动作无法绕过 |
| Phase 4 | 生产运行时与 Host 接入 | 从 CLI demo 变成可集成、可部署、可升级的容器 | Runtime API、进程/SDK 形态、持久化、Skill registry/cache、Host adapter contract | 至少一个真实 Host 能通过稳定协议接入 |
| Phase 5 | 开发者体验与生态兼容 | 让 Skill 开发者能迁移、调试、认证兼容性 | CLI/SDK、导入工具、示例 Skill、兼容报告、文档站 | 外部 Skill 可自助完成本地验证 |
| Phase 6 | 观测、性能与发布运营 | 达到线上可运维、可回滚、可持续发布 | metrics/logs/traces、性能基线、CI/CD gates、runbook | 可灰度发布并定位线上问题 |

> 推荐执行顺序：Phase 0 必须先做；Phase 1、Phase 2 可并行小步推进，但 Phase 3 的安全设计应在 Phase 1/2 开始前冻结关键原则；Phase 4 依赖 Phase 1/2/3 的稳定接口；Phase 5/6 与各阶段同步补齐。

### 2.3 Codex Goal 执行控制

本节把 roadmap 补强为可由 Codex Goal 长跑执行的主 Plan。执行者必须把本文作为唯一规划入口；阶段设计仍以第 3 至第 12 节和 [`production-readiness/`](production-readiness/README.md) 下的阶段文档为准，具体执行状态以本节台账和小 Step 文档为准。

主 Plan 路径：`anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md`

Step 文档目录：`anp/anp-miniapp-dock/docs/plan/production-readiness/steps/`

Harness 入口：`awiki-harness/context/00-context-map.md`、`awiki-harness/context/40-verification.md`、`awiki-harness/context/50-task-workflow.md`

执行状态：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`

#### 2.3.1 Resume From Here

当前恢复指针：从 Step 01-05 [`production-readiness/steps/01-05-unsupported-api-registry-fail-shape.md`](production-readiness/steps/01-05-unsupported-api-registry-fail-shape.md) 开始。

恢复规则：

1. 启动或恢复前，读取本文、当前第一个非 `done` 的 Step 文档、执行台账、本节执行协议、Blocked 处理、Plan 变更记录和当前 `git status --short --branch`。
2. 从执行台账中第一个状态不是 `done` 的 Step 继续，不依赖聊天历史判断进度。
3. 同一时间只允许一个 active Step，除非本文明确标记多个 Step 彼此独立且 parallel-safe。当前 Step 批次没有标记 parallel-safe。
4. 如果当前 Step 状态为 `blocked`，先读取该 Step 的 Blocked 记录；只有依赖允许且风险已记录时，才能转入下一个独立 Step。
5. 每个 Step 完成实现、验证、Review、必要修复和聚焦 commit 后，才能把状态标为 `done` 并进入下一个 Step。

#### 2.3.2 Step 拆分

Phase 0 和 Phase 1 首批 Step 00-01 至 01-04 已完成。Step 01-05 至 02-07 覆盖剩余 Phase 1 `planned-p1` API 能力、Phase 2 组件运行时 P1 起步能力，以及进入 Phase 3 前必须完成的批次最终 Review gate；Step 03-01 至 06-06 覆盖 Phase 3 至 Phase 6 的安全、运行时/Host、开发者体验和发布运营里程碑。后续新增阶段或范围变化仍必须按同一模板拆分小 Step，并通过 Plan 变更记录补充到本表。

| Step | 标题 | 依赖 | 主要产出 | 小 Plan 文档 | Commit gate | 状态 |
|---|---|---|---|---|---|---|
| 00-01 | 当前能力盘点与基线固化 | 无 | 当前能力清单、证据表、demo-only 标注 | [production-readiness/steps/00-01-baseline-inventory.md](production-readiness/steps/00-01-baseline-inventory.md) | 必须 | done |
| 00-02 | wx API 兼容矩阵 | 00-01 | `docs/architecture/wx-api-compatibility-matrix.md` | [production-readiness/steps/00-02-wx-api-compatibility-matrix.md](production-readiness/steps/00-02-wx-api-compatibility-matrix.md) | 必须 | done |
| 00-03 | 组件兼容矩阵 | 00-01 | `docs/architecture/component-compatibility-matrix.md` | [production-readiness/steps/00-03-component-compatibility-matrix.md](production-readiness/steps/00-03-component-compatibility-matrix.md) | 必须 | done |
| 00-04 | Threat model 与 release gates 初版 | 00-01, 00-02, 00-03 | `docs/security/threat-model.md`、`docs/runbook/release-gates.md` | [production-readiness/steps/00-04-threat-model-release-gates.md](production-readiness/steps/00-04-threat-model-release-gates.md) | 必须 | done |
| 01-01 | wx API Bridge Contract 冻结 | 00-02 | bridge 契约、错误语义、callback/Promise 决策 | [production-readiness/steps/01-01-wx-api-bridge-contract-freeze.md](production-readiness/steps/01-01-wx-api-bridge-contract-freeze.md) | 必须 | done |
| 01-02 | Skill package 与 manifest 对齐 | 00-02, 00-03, 01-01 | manifest 校验、兼容报告、测试 | [production-readiness/steps/01-02-skill-package-manifest-alignment.md](production-readiness/steps/01-02-skill-package-manifest-alignment.md) | 必须 | done |
| 01-03 | `wx.modelContext` 原子接口桥接 | 01-01, 01-02 | `getSessionId`、`expireAllCards`、NotificationType、card event | [production-readiness/steps/01-03-model-context-bridge.md](production-readiness/steps/01-03-model-context-bridge.md) | 必须 | done |
| 01-04 | DID 会话与 RequestBroker 收敛 | 01-01, 01-02 | `DidAuthSessionManager`、`wx.login`、`wx.checkSession`、`wx.request` 正式路径 | [production-readiness/steps/01-04-did-session-request-broker.md](production-readiness/steps/01-04-did-session-request-broker.md) | 必须 | done |
| 01-05 | Unsupported API Registry 与统一 fail shape | 01-01, 01-04 | unsupported registry、统一 fail shape、矩阵同步 | [production-readiness/steps/01-05-unsupported-api-registry-fail-shape.md](production-readiness/steps/01-05-unsupported-api-registry-fail-shape.md) | 必须 | pending |
| 01-06 | Storage JS Bridge | 01-01, 01-04, 01-05 | `wx.getStorage` / `setStorage` / `removeStorage` / `clearStorage` 及同步版本 | [production-readiness/steps/01-06-storage-js-bridge.md](production-readiness/steps/01-06-storage-js-bridge.md) | 必须 | pending |
| 01-07 | Device/App Info Atomic API | 01-01；独立于 01-06，可在 01-06 blocked 时按 Blocked 规则串行跳转 | `wx.getDeviceInfo`、`wx.getAppBaseInfo` Atomic API 最小实现 | [production-readiness/steps/01-07-device-app-info-atomic-api.md](production-readiness/steps/01-07-device-app-info-atomic-api.md) | 必须 | pending |
| 01-08 | 高风险 API Host Boundary 与 fail-closed | 01-01, 01-04, 01-05 | phone/address/location/media/payment/scan/phone call Host boundary、ConsentGate、fail closed | [production-readiness/steps/01-08-high-risk-api-host-boundary.md](production-readiness/steps/01-08-high-risk-api-host-boundary.md) | 必须 | pending |
| 02-01 | Render IR schemaVersion 与 fallback reason enum | 00-03, 01-03 | `dock.render-ir.v1`、fallback reason enum | [production-readiness/steps/02-01-render-ir-schema-fallback-reasons.md](production-readiness/steps/02-01-render-ir-schema-fallback-reasons.md) | 必须 | pending |
| 02-02 | Component manifest metadata runtime flow | 01-02, 01-03, 02-01 | `relatedPage`、`scope.dynamic`、`expirable`、`expiredText` runtime metadata | [production-readiness/steps/02-02-component-manifest-metadata-runtime-flow.md](production-readiness/steps/02-02-component-manifest-metadata-runtime-flow.md) | 必须 | pending |
| 02-03 | WXML/WXSS P1 语法增强 | 02-01 | `wx:elif` / `wx:else`、`catchtap`、disabled、简单表达式、P1 WXSS | [production-readiness/steps/02-03-wxml-wxss-p1-syntax.md](production-readiness/steps/02-03-wxml-wxss-p1-syntax.md) | 必须 | pending |
| 02-04 | 表单与静态媒体节点 | 01-08, 02-01, 02-03 | `input`、`textarea`、`radio`、`checkbox`、`picker`、`map-preview`、`canvas-static` | [production-readiness/steps/02-04-form-static-media-nodes.md](production-readiness/steps/02-04-form-static-media-nodes.md) | 必须 | pending |
| 02-05 | Dynamic component controls | 01-04, 02-02 | dynamic `wx.request`、timer、component sandbox escape/resource-limit tests、expire/detach cleanup | [production-readiness/steps/02-05-dynamic-component-controls.md](production-readiness/steps/02-05-dynamic-component-controls.md) | 必须 | done |
| 02-06 | Fixture 与 Render IR snapshots | 02-01, 02-02, 02-03, 02-04, 02-05 | address-form、media-review、dynamic-status、location-map-preview、golden snapshots | [production-readiness/steps/02-06-fixtures-render-ir-snapshots.md](production-readiness/steps/02-06-fixtures-render-ir-snapshots.md) | 必须 | pending |
| 02-07 | 01-05 至 02-06 批次最终 Review 与整体验证 | 01-05, 01-06, 01-07, 01-08, 02-01, 02-02, 02-03, 02-04, 02-05, 02-06 | 批次全局 Review 记录、整体验证证据、Phase 3 启动 gate | [production-readiness/steps/02-07-batch-final-review-verification.md](production-readiness/steps/02-07-batch-final-review-verification.md) | 必须 | pending |
| 03-01 | Threat Model 与安全分级收敛 | 00-04, 01-08, 02-07 | 安全控制矩阵、L0-L4 风险分级、release gate 收敛 | [production-readiness/steps/03-01-threat-model-security-classification.md](production-readiness/steps/03-01-threat-model-security-classification.md) | 必须 | pending |
| 03-02 | QuickJS 沙箱逃逸回归与资源限制 | 02-05, 03-01 | API VM / Component VM sandbox escape tests、resource limits | [production-readiness/steps/03-02-quickjs-sandbox-resource-limits.md](production-readiness/steps/03-02-quickjs-sandbox-resource-limits.md) | 必须 | pending |
| 03-03 | 权限策略引擎与 allowlist decision | 01-08, 02-05, 03-01 | `PermissionDecision`、Host override、network allowlist、decision audit | [production-readiness/steps/03-03-permission-policy-engine-allowlist.md](production-readiness/steps/03-03-permission-policy-engine-allowlist.md) | 必须 | pending |
| 03-04 | DID / Token 生命周期与 Resolver 信任锚 | 01-04, 03-01, 03-03 | refresh/revoke/logout、jti replay、resolver cache/trust anchor | [production-readiness/steps/03-04-did-token-lifecycle-resolver.md](production-readiness/steps/03-04-did-token-lifecycle-resolver.md) | 必须 | pending |
| 03-05 | Consent Adapter 与持久化 Audit Sink | 01-08, 03-01, 03-03 | Host consent adapter、ConsentProof、persistent audit、redacted export | [production-readiness/steps/03-05-consent-adapter-persistent-audit.md](production-readiness/steps/03-05-consent-adapter-persistent-audit.md) | 必须 | pending |
| 03-06 | Skill 包完整性与供应链 Gate | 01-02, 03-01, 03-03 | digest、signature、publisher DID、trusted allowlist、quarantine | [production-readiness/steps/03-06-skill-package-integrity-supply-chain.md](production-readiness/steps/03-06-skill-package-integrity-supply-chain.md) | 必须 | pending |
| 04-01 | Runtime API Facade 与版本化 | 02-06, 03-06 | public Runtime API、stable DTO/error、version、CLI 收敛 | [production-readiness/steps/04-01-runtime-api-facade-versioning.md](production-readiness/steps/04-01-runtime-api-facade-versioning.md) | 必须 | pending |
| 04-02 | IPC / SDK 形态与 Host 进程边界 | 04-01 | local IPC/headless JSON/Rust SDK envelope、version/error/redaction | [production-readiness/steps/04-02-ipc-sdk-host-process-boundary.md](production-readiness/steps/04-02-ipc-sdk-host-process-boundary.md) | 必须 | pending |
| 04-03 | Skill Registry / Cache 与版本回滚 | 03-06, 04-01 | registry ref、digest-keyed cache、version pin、rollback、eviction | [production-readiness/steps/04-03-skill-registry-cache-versioning.md](production-readiness/steps/04-03-skill-registry-cache-versioning.md) | 必须 | pending |
| 04-04 | Runtime Config 与 Secret Store 边界 | 04-01 | non-secret runtime config、secret provider boundary、redaction | [production-readiness/steps/04-04-runtime-config-secret-store.md](production-readiness/steps/04-04-runtime-config-secret-store.md) | 必须 | pending |
| 04-05 | Token Cache 持久化与恢复 | 03-04, 04-04 | secure token cache backend、TTL/revocation restore policy | [production-readiness/steps/04-05-token-cache-persistence.md](production-readiness/steps/04-05-token-cache-persistence.md) | 必须 | pending |
| 04-06 | Scoped Storage 持久化与 quota | 01-06, 04-04 | scoped storage backend、quota、scope cleanup | [production-readiness/steps/04-06-scoped-storage-persistence.md](production-readiness/steps/04-06-scoped-storage-persistence.md) | 必须 | pending |
| 04-07 | Persistent Audit Sink retention/export | 03-05, 04-04 | audit persistence、retention、redacted export | [production-readiness/steps/04-07-persistent-audit-retention-export.md](production-readiness/steps/04-07-persistent-audit-retention-export.md) | 必须 | pending |
| 04-08 | Skill Cache cleanup 与版本清理 | 04-03, 04-04 | digest cache cleanup、eviction、privacy/delete scope hooks | [production-readiness/steps/04-08-skill-cache-cleanup.md](production-readiness/steps/04-08-skill-cache-cleanup.md) | 必须 | pending |
| 04-09 | Host Adapter Contract 与 Action Protocol | 01-08, 02-06, 03-05, 04-01 | Host renderer/provider/action conformance、headless adapter | [production-readiness/steps/04-09-host-adapter-contract-action-protocol.md](production-readiness/steps/04-09-host-adapter-contract-action-protocol.md) | 必须 | pending |
| 04-10 | 并发、取消、重试与幂等策略 | 02-05, 03-03, 03-05, 04-01, 04-05, 04-06, 04-07, 04-09 | session manager、cancellation、retry policy、idempotency key | [production-readiness/steps/04-10-concurrency-cancellation-idempotency.md](production-readiness/steps/04-10-concurrency-cancellation-idempotency.md) | 必须 | pending |
| 05-01 | CLI validate 兼容报告增强 | 01-05, 02-06, 03-06, 04-01 | JSON compatibility report、releaseBlockers、修复建议 | [production-readiness/steps/05-01-cli-validate-compatibility-report.md](production-readiness/steps/05-01-cli-validate-compatibility-report.md) | 必须 | pending |
| 05-02 | CLI inspect Skill package | 05-01 | package 文件、API/registration 对照、组件/权限/risk/wx usage | [production-readiness/steps/05-02-cli-inspect-skill-package.md](production-readiness/steps/05-02-cli-inspect-skill-package.md) | 必须 | pending |
| 05-03 | CLI test-skill 与 Fixture Runner | 02-06, 04-01, 05-01 | fixture runner、snapshot compare、action/audit report | [production-readiness/steps/05-03-cli-test-skill-fixture-runner.md](production-readiness/steps/05-03-cli-test-skill-fixture-runner.md) | 必须 | pending |
| 05-04 | CLI import-wechat-mcp | 05-01, 05-02 | dry-run/safe copy、兼容报告、ANP `_meta` patch 建议 | [production-readiness/steps/05-04-cli-import-wechat-mcp.md](production-readiness/steps/05-04-cli-import-wechat-mcp.md) | 必须 | pending |
| 05-05 | CLI doctor 环境诊断 | 03-04, 04-04, 04-05, 04-06, 04-07, 05-01 | toolchain/DID/resolver/allowlist/storage/audit/provider diagnostics | [production-readiness/steps/05-05-cli-doctor-environment.md](production-readiness/steps/05-05-cli-doctor-environment.md) | 必须 | pending |
| 05-06 | 示例 Skill 与兼容测试集 | 02-06, 05-01, 05-03 | address/media/dynamic/location 示例、README、expected JSON、snapshots | [production-readiness/steps/05-06-example-skills-compatibility-fixtures.md](production-readiness/steps/05-06-example-skills-compatibility-fixtures.md) | 必须 | pending |
| 05-07 | 开发者文档与迁移指南 | 05-01, 05-02, 05-03, 05-04, 05-05, 05-06 | import/API/component/security/Host adapter developer docs | [production-readiness/steps/05-07-developer-docs-migration-guides.md](production-readiness/steps/05-07-developer-docs-migration-guides.md) | 必须 | pending |
| 06-01 | 结构化观测事件与脱敏日志 | 04-01, 04-04, 05-07 | structured events、traceId/sessionId、redaction | [production-readiness/steps/06-01-structured-observability-events.md](production-readiness/steps/06-01-structured-observability-events.md) | 必须 | pending |
| 06-02 | Metrics / Tracing 与请求链路关联 | 04-02, 06-01 | metrics registry、trace propagation、low-cardinality labels | [production-readiness/steps/06-02-metrics-tracing-correlation.md](production-readiness/steps/06-02-metrics-tracing-correlation.md) | 必须 | pending |
| 06-03 | 性能基线与 Stress Tests | 04-10, 06-02 | benchmarks、stress tests、baseline artifact | [production-readiness/steps/06-03-performance-baselines-stress.md](production-readiness/steps/06-03-performance-baselines-stress.md) | 必须 | pending |
| 06-04 | CI/CD Release Gates 自动化 | 02-06, 03-06, 05-03, 06-03 | gate runner、CI workflow、release report、docs link/redaction/snapshot gates | [production-readiness/steps/06-04-ci-cd-release-gates-automation.md](production-readiness/steps/06-04-ci-cd-release-gates-automation.md) | 必须 | pending |
| 06-05 | Canary 发布、版本化与回滚策略 | 04-03, 04-09, 06-02, 06-03, 06-04 | release notes、canary stages、rollback/cache purge | [production-readiness/steps/06-05-canary-release-rollback.md](production-readiness/steps/06-05-canary-release-rollback.md) | 必须 | pending |
| 06-06 | 运维 Runbook 与隐私删除流程 | 04-04, 04-06, 04-07, 04-08, 05-05, 06-01, 06-02, 06-04, 06-05 | operations/troubleshooting/privacy deletion runbooks | [production-readiness/steps/06-06-operations-runbook-privacy-deletion.md](production-readiness/steps/06-06-operations-runbook-privacy-deletion.md) | 必须 | pending |

独立跳转说明：Step 01-07 只依赖 Step 01-01，和 Step 01-06 storage 工作没有实现依赖。若 Step 01-06 被标记为 `blocked`，执行者可以在记录 blocker、确认工作区无未提交完成工作后，按 Blocked 规则串行转入 Step 01-07；这不是并行执行授权。

#### 2.3.3 执行台账

执行者必须在 Step 开始、进入 Review、被阻塞、完成 commit 和标记 done 时更新本台账。证据字段必须指向 Step 文档中的 Review/验证记录或实际命令输出摘要，不得只写“已完成”。

| Step | 状态 | 分支 | 开始时间 | 完成时间 | Commit | Review 证据 | 验证证据 | 下一步 |
|---|---|---|---|---|---|---|---|---|
| 00-01 | done | `main` | 2026-06-12 10:05:15 +0800 | 2026-06-12 10:16:44 +0800 | `de4c3e2` | Step 文档 Review 环节已记录：未发现需修复问题，demo-only/host-boundary 未被误标为 production-ready | `cargo metadata --format-version 1 --no-deps` 成功；`git diff --check -- docs/architecture README.md docs/plan/production-readiness-roadmap.md docs/plan/production-readiness/steps/00-01-baseline-inventory.md` 无输出；新增基线文档 Markdown 链接手工检查无破链；post-commit `git status --short --branch` = `## main...origin/main [ahead 2]` | 进入 00-02 |
| 00-02 | done | `main` | 2026-06-12 10:18:26 +0800 | 2026-06-12 10:31:32 +0800 | `22e7f25` | Step 文档 Review 环节已记录：未发现阻塞问题，`demo-only`/`host-boundary` 未被误标为 production-ready，高风险 API 均要求 ConsentGate/audit 或 fail closed | `git diff --check -- docs/architecture/wx-api-compatibility-matrix.md README.md docs/plan/production-readiness-roadmap.md docs/plan/production-readiness/steps/00-02-wx-api-compatibility-matrix.md` 无输出；协议覆盖抽样命中协议参考和矩阵；status 列结构化检查无非法枚举；矩阵 Markdown 链接检查无破链；L3/L4 与敏感字段抽样通过；post-commit `git status --short --branch` = `## main...origin/main [ahead 4]` | 进入 00-03 |
| 00-03 | done | `main` | 2026-06-12 10:33:02 +0800 | 2026-06-12 10:40:11 +0800 | `fea9d35` | Step 文档 Review 环节已记录：未发现阻塞问题，Runtime/Host 责任分离，dynamic、Host renderer 和完整小程序能力未被误标为 production-ready | `git diff --check -- docs/architecture/component-compatibility-matrix.md README.md docs/plan/production-readiness-roadmap.md docs/plan/production-readiness/steps/00-03-component-compatibility-matrix.md` 无输出；覆盖抽样命中矩阵、架构和 Phase 2 计划；status 列结构化检查无非法枚举；矩阵 Markdown 链接检查无破链；安全边界和 Host 边界抽样通过；post-commit `git status --short --branch` = `## main...origin/main [ahead 6]` | 进入 00-04 |
| 00-04 | done | `main` | 2026-06-12 10:41:43 +0800 | 2026-06-12 10:51:58 +0800 | `04448a1` | Step 文档 Review 环节已记录：未发现阻塞问题，planned gate 未被误写成当前已自动化，demo-only/mock 能力仍为 production release blocker | pre-flight: `git status --short --branch` = `## main...origin/main [ahead 7]`；`git diff --check -- docs/security docs/runbook/release-gates.md README.md docs/plan/production-readiness-roadmap.md docs/plan/production-readiness/steps/00-04-threat-model-release-gates.md` 无输出；安全红线抽样命中 threat model、release gates 和 redaction 规则；README、threat model、release gates Markdown 相对链接检查无破链；release gate 命令与 `AGENTS.md` 一致；post-commit `git status --short --branch` = `## main...origin/main [ahead 8]` | 进入 01-01 |
| 01-01 | done | `main` | 2026-06-12 10:53:13 +0800 | 2026-06-12 10:59:18 +0800 | `10db676` | Step 文档 Review 环节已记录：未发现阻塞问题，callback/Promise、`wx.request` HTTP status、unsupported shape、JS-provided auth header、sync API 和 redaction 均已有冻结行为 | pre-flight: `git status --short --branch` = `## main...origin/main [ahead 9]`；`git diff --check -- docs/plan/production-readiness/phase-1-wx-api-bridge-contract.md docs/architecture/wx-api-compatibility-matrix.md docs/plan/production-readiness-roadmap.md docs/plan/production-readiness/steps/01-01-wx-api-bridge-contract-freeze.md` 无输出；未决行为检查无命中；脱敏和契约抽样命中敏感字段、HTTP response、4xx/5xx、Promise reject、unsupported、错误码和 WxApiCall/WxApiOutcome；contract 与矩阵 Markdown 链接检查无破链；post-commit `git status --short --branch` = `## main...origin/main [ahead 10]` | 进入 01-02 |
| 01-02 | done | `main` | 2026-06-12 11:00:47 +0800 | 2026-06-12 11:22:56 +0800 | `ec46e1f` | Step 文档 Review 环节已记录：修复 registration mismatch 报告测试缺口和根级 `format:file` warning path 稳定性问题，未发现阻塞问题 | `cargo fmt --check` 通过；`cargo test -p mcp-schema` 13 passed；`cargo test -p skill-loader` 7 passed；`cargo test -p dock-cli validate` 2 passed；`cargo run -p dock-cli -- validate examples/coffee-skill` 输出 `compatibilityLevel: demo-only` 和完整 `compatibilityReport`；`git diff --check -- crates/mcp-schema crates/skill-loader crates/dock-cli docs/architecture docs/runbook docs/plan` 无输出；`cargo test -p dock-cli --test coffee_order_flow` 3 passed；`cargo clippy --workspace --all-targets -- -D warnings` 通过；post-commit `git status --short --branch` = `## main...origin/main [ahead 12]` | 进入 01-03 |
| 01-03 | done | `main` | 2026-06-12 11:24:05 +0800 | 2026-06-12 11:45:53 +0800 | `1504eff` | Step 文档 Review 环节已记录：补齐 `createSkill(skillPath)` 路径校验、脱敏 `expireAllCards` invalid options 错误、`match: all` 覆盖和矩阵同步；生产 card manager/持久化 audit 仍按 Phase 2/4 边界记录 | `cargo fmt --check` 通过；`cargo test -p js-runtime-quickjs -p wx-compat -p dock-core model_context` 通过，实际 js-runtime 6 passed、wx-compat 2 passed、dock-core 0 tests under filter；`cargo test -p js-runtime-quickjs create_skill` 1 passed；`cargo test -p component-runtime` 25 passed；`cargo test -p dock-core` 9 passed；`cargo test -p dock-cli --test coffee_order_flow` 3 passed；`git diff --check -- crates/js-runtime-quickjs crates/wx-compat crates/dock-core crates/component-runtime docs/architecture docs/plan` 无输出；`cargo clippy --workspace --all-targets -- -D warnings` 通过；post-commit `git status --short --branch` = `## main...origin/main [ahead 14]` | 进入 01-04 |
| 01-04 | done | `main` | 2026-06-12 11:47:39 +0800 | 2026-06-12 12:23:36 +0800 | `e9cbdbe` | Step 文档 Review 环节已记录：修复 callback exception 改变 Promise outcome/跳过 `complete` 的契约问题；确认 token 只在 Host/runtime 边界、JS auth header fail closed、response auth/token headers 已脱敏；production Host RequestBroker transport 和 persistent audit 按 Phase 4 残余风险记录 | pre-flight: `git status --short --branch` = `## main...origin/main [ahead 15]`；`cargo fmt --check` 通过；`cargo test -p anp-adapter session` 8 passed；`cargo test -p js-runtime-quickjs wx_login/check_session/wx_request/wx_callback_exception/model_context_expire_all_cards` 均通过；`cargo test -p js-runtime-quickjs` 28 passed；`cargo test -p anp-adapter` 41 passed；`cargo test -p wx-compat` 9 passed；`cargo test -p demo-server token` 4 passed under filter；`cargo test -p dock-cli --test coffee_order_flow` 3 passed；`cargo test --workspace` 通过；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`git diff --check -- crates/anp-adapter crates/js-runtime-quickjs crates/wx-compat crates/dock-core crates/demo-server crates/dock-cli examples/coffee-fastapi-server docs/architecture docs/runbook docs/security docs/plan` 无输出；post-commit `git status --short --branch` = `## main...origin/main [ahead 16]` | 首批 Step 已全部完成，进入最终全局 Review |
| 01-05 | done | `main` | 2026-06-12 20:02:26 +0800 | 2026-06-12 20:21:27 +0800 | `8e475dd` | Step 文档 Review 环节已记录：修复 focused `unsupported` filter 未覆盖 unknown root fallback 的测试命名；确认 registry/stub 不覆盖已支持 API，Proxy intrinsic 不暴露给 Skill，真实 provider 状态未误标 supported | `cargo fmt --check` 通过；`cargo test -p js-runtime-quickjs unsupported` 5 passed；`cargo test -p wx-compat unsupported` 4 passed；`cargo test -p js-runtime-quickjs wx_` 13 passed；`cargo test -p js-runtime-quickjs` 33 passed；`cargo test -p wx-compat` 11 passed；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`git diff --check -- crates/wx-compat crates/js-runtime-quickjs docs/architecture docs/runbook docs/plan` 无输出；敏感词抽样仅命中安全说明、测试夹具假值和 redaction 断言 | 进入 01-06 |
| 01-06 | done | `main` | 2026-06-12 20:22:41 +0800 | 2026-06-12 20:43:49 +0800 | `1599294` | Step 文档 Review 环节已记录：未发现阻塞问题；确认 scope 使用 `userDid + merchantDid + skillId` 且不含 `sessionId`，sync/async 语义与 Step 01-01 契约一致，storage 内容不自动进入 model-visible result，未引入生产持久化承诺 | `cargo fmt --check` 通过；`cargo test -p wx-compat storage` 8 passed；`cargo test -p js-runtime-quickjs storage` 6 passed；`cargo test -p js-runtime-quickjs wx_` 20 passed；`cargo test -p js-runtime-quickjs` 39 passed；`cargo test -p wx-compat` 16 passed；`cargo test -p dock-cli --test coffee_order_flow` 3 passed；`git diff --check -- crates/wx-compat crates/js-runtime-quickjs crates/dock-core docs/architecture docs/runbook docs/security docs/plan` 无输出；`cargo clippy --workspace --all-targets -- -D warnings` 通过；敏感词抽样仅命中文档规则、测试假值和 redaction 断言；post-commit `git status --short --branch` = `## main...origin/main [ahead 25]` | 进入 01-07 |
| 01-07 | done | `main` | 2026-06-12 20:45:17 +0800 | 2026-06-12 20:53:52 +0800 | `50cc245` | Step 文档 Review 环节已记录：未发现阻塞问题；确认 Atomic API 与 Component VM 使用 `wx-compat` shared defaults，字段最小化，不返回真实设备指纹或 Host credential 信息，sync API 不返回 Promise | `cargo fmt --check` 通过；`cargo test -p js-runtime-quickjs info` 2 passed；`cargo test -p wx-compat device` 1 passed；`cargo test -p component-runtime` 26 passed；`git diff --check -- crates/wx-compat crates/js-runtime-quickjs crates/component-runtime docs/architecture docs/plan docs/runbook` 无输出；`cargo clippy --workspace --all-targets -- -D warnings` 通过；敏感字段抽样仅命中文档红线、测试假值、禁用字段断言和既有 redaction 代码；post-commit `git status --short --branch` = `## main...origin/main [ahead 27]` | 进入 01-08 |
| 01-08 | done | `main` | 2026-06-12 20:55:30 +0800 | 2026-06-12 21:09:39 +0800 | `33591f0` | Step 文档 Review 环节已记录：未发现阻塞问题；确认默认 Atomic API runtime 只接入 unavailable provider、dev-only provider 未进入 production 默认路径、ConsentGate 在 provider 前阻断、本地路径和支付密码不会回显 | `cargo fmt --check` 通过；`cargo test -p wx-compat provider` 3 passed；`cargo test -p wx-compat high_risk` 2 passed；`cargo test -p js-runtime-quickjs high_risk` 3 passed；`cargo test -p wx-compat unsupported` 4 passed；`cargo test -p consent-audit -p dock-core consent` 8 passed；`cargo test --workspace` 通过；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`git diff --check -- crates/wx-compat crates/dock-core crates/consent-audit crates/js-runtime-quickjs docs/architecture docs/runbook docs/security docs/plan` 无输出；敏感词抽样仅命中文档红线、测试假值和 redaction 断言；post-commit `git status --short --branch` = `## main...origin/main [ahead 29]` | 进入 02-01 |
| 02-01 | done | `main` | 2026-06-12 21:11:25 +0800 | 2026-06-12 21:20:27 +0800 | `0cfea24` | Step 文档 Review 环节已记录：未发现阻塞问题；确认所有 Component Runtime Render IR 输出带 `schemaVersion`，fallback reason 对外为稳定枚举值，旧自由字符串只在内部 normalize，未向 CLI/Host payload 泄露路径或错误细节 | `cargo fmt --check` 通过；`cargo test -p component-runtime render` 5 passed；`cargo test -p component-runtime render_output_serializes_schema_version` 1 passed；`cargo test -p card-spec fallback` 1 passed；`cargo test -p dock-core fallback` 1 passed；`cargo test -p dock-cli preview_card` 2 passed；`cargo test -p card-spec -p dock-core -p component-runtime` 40 passed；`cargo test -p dock-cli --test coffee_order_flow` 3 passed；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`git diff --check -- crates/component-runtime crates/dock-core crates/card-spec crates/dock-cli docs/architecture docs/runbook docs/plan` 无输出；敏感词抽样仅命中测试假路径和文档安全说明；post-commit `git status --short --branch` = `## main...origin/main [ahead 31]` | 进入 02-02 |
| 02-02 | done | `main` | 2026-06-12 21:22:16 +0800 | 2026-06-12 21:37:07 +0800 | `79417d5` | Step 文档 Review 环节已记录：修复 validate report 泄露原始 `relatedPage.query.secretToken` 的问题和 metadata 初版从 JS snapshot 回传的错误设计；确认 metadata 来自 manifest，dynamic request/timer 仍默认关闭 | `cargo fmt --check` 通过；`cargo test -p mcp-schema -p skill-loader component` 5 passed；`cargo test -p component-runtime metadata` 1 passed；`cargo test -p dock-cli metadata` 2 passed；`cargo test -p dock-cli --test coffee_order_flow` 3 passed；`cargo test -p component-runtime -p dock-cli -p mcp-schema -p skill-loader` 73 passed；`cargo run -p dock-cli -- validate examples/coffee-skill` 输出 `demo-only` 且 components 含 `runtimeMetadata`；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`git diff --check -- crates/mcp-schema crates/skill-loader crates/component-runtime crates/wx-compat crates/dock-cli docs/architecture docs/plan docs/runbook` 无输出；敏感词抽样仅命中 redaction 规则、测试假值和文档红线；post-commit `git status --short --branch` = `## main...origin/main [ahead 33]` | 进入 02-03 |
| 02-03 | done | `main` | 2026-06-12 21:38:22 +0800 | 2026-06-12 21:54:00 +0800 | `c8bb813` | Step 文档 Review 环节已记录：修复复杂 selector 静默吞掉的问题；确认 expression evaluator 为 allowlist-only，不执行任意 JS；disabled button 不产生 tap/catchtap action；`catchtap` 只扩展 Render IR 事件语义并仍映射为受控 tap；未引入 02-04 表单节点或 02-05 dynamic request/timer | `cargo fmt --check` 通过；`cargo test -p component-runtime wx` 14 passed under filter；`cargo test -p component-runtime` 36 passed；`cargo test -p dock-cli --test coffee_order_flow` 3 passed；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`git diff --check -- crates/component-runtime docs/architecture docs/plan` 无输出；本步骤 diff 敏感词抽样未新增真实 secret、本机绝对路径或隐私数据；post-commit `git status --short --branch` = `## main...origin/main [ahead 35]` | 进入 02-04 |
| 02-04 | done | `main` | 2026-06-12 21:54:59 +0800 | 2026-06-12 22:03:40 +0800 | `cc7b3b8` | Step 文档 Review 环节已记录：修复 `maxlength` 等数值 props 初版以字符串输出的问题；确认新增表单节点只是 Render IR 数据，disabled 会抑制 `input` / `change` / tap event，`map-preview` 不透传精确经纬度/markers，`canvas-static` 不开放 script/touch 交互，未绕过 Orchestrator input validation、ConsentGate 或 Host provider | `cargo fmt --check` 通过；`cargo test -p component-runtime node` 7 passed under filter；`cargo test -p component-runtime` 40 passed；`cargo test -p dock-cli --test coffee_order_flow` 3 passed；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`git diff --check -- crates/component-runtime crates/dock-core docs/architecture docs/plan` 无输出；敏感词抽样仅命中本步骤精确经纬度/markers 拒绝测试、文档安全说明和既有台账文字；post-commit `git status --short --branch` = `## main...origin/main [ahead 37]` | 进入 02-05 |
| 02-05 | done | `main` | 2026-06-12 22:05:11 +0800 | 2026-06-12 22:32:24 +0800 | `7baca29` | 2026-06-12 22:30:25 +0800 commit 前 Review 已记录：修复 native request bridge 全局暴露、component `wx.request` callback 语义与 Atomic API bridge 不一致、`setInterval` 退化为一次性 flush、resource-limit 缺少 focused timeout 测试；剩余 Host transport/background scheduler/persistent audit 按 Phase 4 边界记录 | `cargo fmt --check` 通过；`cargo test -p component-runtime dynamic` 5 passed；`cargo test -p component-runtime sandbox` 2 passed；`cargo test -p component-runtime` 46 passed；`cargo test -p wx-compat` 22 passed；`cargo test -p anp-adapter request` 2 passed；`cargo test --workspace` 通过；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`git diff --check -- crates/component-runtime crates/wx-compat crates/js-runtime-quickjs crates/anp-adapter docs/architecture docs/runbook docs/security docs/plan` 无输出；敏感词抽样仅命中 redaction 规则、测试假值和安全文档；post-commit `git status --short --branch` = `## main...origin/main [ahead 39]` | 进入 02-06 |
| 02-06 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 02-05 完成 |
| 02-07 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 01-05 至 02-06 全部完成后，执行批次最终 Review |
| 03-01 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 02-07 完成 |
| 03-02 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 03-01 完成 |
| 03-03 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 03-02 完成 |
| 03-04 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 03-03 完成 |
| 03-05 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 03-04 完成 |
| 03-06 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 03-05 完成 |
| 04-01 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 03-06 完成 |
| 04-02 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-01 完成 |
| 04-03 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-02 完成 |
| 04-04 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-03 完成 |
| 04-05 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-04 完成 |
| 04-06 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-05 完成 |
| 04-07 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-06 完成 |
| 04-08 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-07 完成 |
| 04-09 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-08 完成 |
| 04-10 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-09 完成 |
| 05-01 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 04-10 完成 |
| 05-02 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 05-01 完成 |
| 05-03 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 05-02 完成 |
| 05-04 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 05-03 完成 |
| 05-05 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 05-04 完成 |
| 05-06 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 05-05 完成 |
| 05-07 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 05-06 完成 |
| 06-01 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 05-07 完成 |
| 06-02 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 06-01 完成 |
| 06-03 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 06-02 完成 |
| 06-04 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 06-03 完成 |
| 06-05 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 06-04 完成 |
| 06-06 | pending | `main` | 待记录 | 待记录 | 待记录 | 待记录 | 待记录 | 等待 06-05 完成 |

#### 2.3.4 Codex Goal 执行协议

1. 启动前先执行并记录 `git status --short --branch`，确认是否存在用户未提交改动；若有，必须保护这些改动，不得回滚。
2. 读取顺序：本文、当前 Step 文档、相关 Phase 文档、相关深入子文档、`AGENTS.md`、必要的源文件和测试。
3. 一个 Step 的标准状态流为：`pending` -> `in_progress` -> `review` -> `committed` -> `done`。若被阻塞，记录为 `blocked`，解决后回到 `in_progress` 或 `review`。
4. 每个 Step 必须按对应小 Plan 执行，不得把多个 Step 的完成工作混在一个 commit 中。
5. 每个 Step 都必须同步更新实现、测试、兼容矩阵、runbook 或开发者文档中与该 Step 直接相关的部分；如果某类文档不适用，必须在 Step 记录原因。
6. 每个 Step 都必须运行小 Plan 中列出的验证命令；不能运行时，记录原因、影响和替代证据。
7. 每个 Step 都必须在 commit 前进行 Review，记录发现、修复、剩余风险、新增或缺失测试、已更新或缺失文档。
8. 只有 Review 必要问题已修复或明确记录、验证证据已记录、commit 已创建并回填台账后，才能把 Step 标为 `done`。
9. 任何改变范围、顺序、验收标准、公开契约、数据模型、配置、验证策略或安全边界的决定，必须先更新本文的 Plan 变更记录和受影响 Step 文档。
10. 完成全部 Step 后，执行最终全局 Review 与整体验证，不得只依赖各 Step 的局部验证。

#### 2.3.5 Review 与提交门禁

每个 Step 的 Review 是 commit 前门禁，不是可选收尾。Review 至少覆盖：

- 正确性：实现是否满足 Step 验收标准和阶段目标；
- 回归风险：是否破坏 coffee demo、现有 VM、组件、DID、consent/audit 路径；
- 公开契约：`SKILL.md`、`mcp.json`、`structuredContent`、`_meta.ui.componentPath`、Render IR、CLI JSON 是否漂移；
- 安全与隐私：token、`Authorization`、HTTP signature、private key path、手机号、地址、文件内容是否脱敏；
- 测试覆盖：是否有 unit、integration、fixture、snapshot 或文档检查证据；
- 文档同步：兼容矩阵、runbook、开发者文档、阶段文档是否与实现一致。

提交要求：

- 每个完成的 Step 创建一个 focused commit，commit message 建议格式为 `phase<N>: <step outcome>` 或 `docs: <step outcome>`。
- commit 前记录 `git status --short`、纳入文件和不纳入文件。
- commit 后记录 commit hash 与 `git status --short --branch`。
- 上一步完成工作未提交前，不得开始依赖它的下一步，除非 Plan 变更记录明确说明依赖原因和风险控制。
- 最终全局 Review 如果修改文件，需要单独 Review、验证并创建最终集成 commit。

#### 2.3.6 Blocked 处理

| Blocker | Step | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

处理规则：

1. Blocker 必须写清楚触发命令、文件、错误输出或缺失决策，不得只写“实现困难”。
2. 若 blocker 只影响当前 Step，且后续某个 Step 在依赖表中不依赖它，执行者可以在记录风险后转入该独立 Step；当前 Step 批次默认不并行，除非更新本文。
3. 若 blocker 影响公开契约、安全边界、数据模型或阶段顺序，必须先更新 Plan 变更记录，再决定是否继续。
4. 只有没有安全假设、替代方案或独立下一步时，才向用户提问。

#### 2.3.7 Plan 变更控制

| 日期 | 变更 | 原因 | 影响步骤 | 是否需要 Review |
|---|---|---|---|---|
| 2026-06-12 | 新增 Codex Goal 执行控制、执行台账和 Phase 0/Phase 1 首批 Step 文档 | 将 roadmap 补强为可恢复执行的 AWiki 长跑计划 | 00-01 至 01-04 | 是 |
| 2026-06-12 | 新增 Phase 1 剩余 `planned-p1` 与 Phase 2 P1 下一批 Step 文档 | 将首批完成后的残余 API 缺口和组件运行时起步能力拆成可由 Codex Goal 继续执行的小 Plan | 01-05 至 02-06 | 是 |
| 2026-06-12 | 新增 Phase 3 至 Phase 6 全部后续里程碑 Step 文档 | 将安全增强、生产运行时/Host、开发者体验、观测发布运营阶段拆成可由 Codex Goal 顺序执行的小 Plan | 03-01 至 06-06 | 是 |
| 2026-06-12 | 修复计划 Review 发现：新增 02-07 final Review gate、前置 dynamic sandbox gate、拆分 Phase 4 持久化、标注 01-07 blocked 跳转 | 让 Codex Goal 恢复执行时不会跳过批次最终 Review，避免动态组件先于安全 gate 扩权，保持每个 Step 一个 focused commit | 01-07、02-05、02-06、02-07、03-01、04-04 至 04-10、05-05、06-01、06-03、06-05、06-06 | 是 |

变更规则：

- 先改 Plan，再改实现；不得先扩大实现范围再补文档。
- 变更必须说明是否影响依赖、验收标准、验证命令、Review 范围和 commit 策略。
- 受影响 Step 已经开始时，必须同步更新该 Step 的执行状态和变更记录。

#### 2.3.8 最终全局 Review 与整体验证

触发条件：当前批次 Step 或后续扩展 Step 全部 `done`，且每个 Step 都已有 Review 证据、验证证据和 commit hash。历史执行记录只覆盖当时已完成的 Step；新增 Step 完成后必须追加新的最终全局 Review 记录。

最终 Review 范围：

- 全部变更文件、公开契约、测试、fixtures、兼容矩阵、runbook、开发者文档；
- `wx.*` / `wx.modelContext` 行为与小程序 MCP 兼容策略；
- ANP DID、capability token、allowlist、consent、audit、redaction 和 sandbox 安全边界；
- 执行台账是否与 git history、验证命令和 Step 文档一致；
- 是否存在未提交变更、未解决 Review 发现、跳过验证或文档漂移。

整体验证基线：

```bash
cd anp/anp-miniapp-dock
cargo metadata --format-version 1 --no-deps
cargo fmt --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p dock-cli --test coffee_order_flow
```

文档和计划验证：

```bash
cd anp/anp-miniapp-dock
git diff --check -- docs/plan docs/architecture docs/runbook docs/security README.md AGENTS.md
```

若命令因环境、依赖或时间无法运行，必须记录原因、影响、替代检查和剩余风险。最终 Review 若修改文件，必须单独记录 Review、验证和最终集成 commit。

最终 Review 执行记录：

> 以下记录只覆盖 Step 00-01 至 01-04。新增 Step 01-05 至 02-06 全部完成后，必须在本节追加新的最终全局 Review 与整体验证记录。

| 项目 | 记录 |
|---|---|
| 执行时间 | 2026-06-12 12:31:47 +0800 |
| 范围 | Roadmap 执行台账、Step 00-01 至 01-04 文档、Phase 1 子文档、兼容矩阵、release gates、local demo runbook、threat model、相关源码/测试和 git history。 |
| Step/ledger 审计 | 执行台账中 00-01 至 01-04 均为 `done`；未发现 pending / in_progress / review / blocked / committed Step；台账记录的主产物 commit `de4c3e2`、`22e7f25`、`fea9d35`、`04448a1`、`10db676`、`ec46e1f`、`1504eff`、`e9cbdbe` 均能在 git history 解析。 |
| Review 发现与修复 | 修复文档漂移：`current-capability-baseline.md` 明确为 Step 00-01 时点基线，避免与 01-03/01-04 后实时能力混淆；`phase-1-wx-capability-broker.md` 和 `phase-1-wx-api-bridge-contract.md` 标出已由 01-03/01-04 证明的验收项，并保留 storage、L3/L4 provider、全量 unsupported stub、production Host RequestBroker 等后续工作。 |
| 安全/敏感信息 Review | 运行敏感词扫描命中预期的测试夹具、redaction 代码、安全文档和 demo-only 文档；未发现真实 secret、真实 DID private key、真实 bearer token 或生产凭据。测试继续覆盖 CLI/demo redaction、JS auth header fail closed、response auth/token header redaction、challenge proof redaction 和 audit redaction。 |
| 残余风险 | 首批 Step 已完成；production Host RequestBroker transport、registry allowlist、persistent request/audit store、logout/revocation list、全量 unsupported API stub、storage JS bridge、L3/L4 Host provider conformance 仍按 Phase 2/3/4/5 计划推进，不作为本次首批 Step 完成条件。 |
| 整体验证 | `cargo metadata --format-version 1 --no-deps` 通过；`cargo fmt --check` 通过；`cargo clippy --workspace --all-targets -- -D warnings` 通过；`cargo test --workspace` 通过；`cargo test -p dock-cli --test coffee_order_flow` 3 passed；`cargo run -p dock-cli -- validate examples/coffee-skill` 通过并保持 `compatibilityLevel: demo-only`；`git diff --check -- docs/plan docs/architecture docs/runbook docs/security README.md AGENTS.md` 通过。 |
| 最终工作区状态 | 记录 Review 前 `git status --short --branch` = `## main...origin/main [ahead 17]`；最终 Review 文档提交为 `636f1e9 docs: record final production readiness review`；最终台账关闭提交后 `git status --short --branch` = `## main...origin/main [ahead 19]`，且工作区无未提交变更。 |

#### 2.3.9 Codex Goal 提示词

下面提示词用于启动后续实现型 Codex Goal。执行时仍以本文和 Step 文档为准，当前应从 Step 01-05 开始。

```text
请以 anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md 为唯一规划入口，按文档执行生产化计划。当前从第一个非 done Step（现在是 01-05）开始。

开始前先读取：
- anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md
- 当前第一个未 done 的 Step 文档
- 主 Plan 的执行台账、Codex Goal 执行协议、Review 与提交门禁、Blocked 处理、Plan 变更记录
- 当前 git status --short --branch

请从第一个状态不是 done 的 Step 开始，一次只执行一个 Step。每步都要按对应小 Plan 实现、验证、Review、修复或记录 Review 发现，然后创建一个 focused commit，并回填主 Plan 执行台账和 Step 执行状态。

需要改变范围、顺序、验收标准、公开契约、数据模型、安全边界或验证策略时，先更新 Plan 变更记录和受影响 Step 文档。不得绕过 ANP DID、capability token、allowlist、ConsentGate、audit、redaction 和 sandbox 边界。

所有步骤完成后，执行最终全局 Review 和整体验证，记录实际命令、通过/失败/跳过数量、失败或跳过原因、剩余风险和最终工作区状态。
```

本次 Phase 1/2 执行型 Codex Goal 可使用更窄的提示词。它只执行当前未完成的 Phase 1 Step 和 Phase 2 全部 Step，完成 Step 02-07 后停止，不进入 Phase 3：

```text
请以 anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md 为唯一规划入口，只执行当前未完成的 Phase 1 Step 和 Phase 2 全部 Step：从 01-05 开始，做到 02-07 完成后停止，不进入 03-01。

开始前先读取主 Plan、当前 Step 文档、执行台账、Codex Goal 执行协议、Review 与提交门禁、Blocked 处理、Plan 变更记录，以及当前 git status --short --branch。

请按执行台账从第一个非 done Step 继续，一次只执行一个 Step。每个 Step 都必须按对应小 Plan 实现、验证、Review、修复或记录 Review 发现，创建一个 focused commit，并回填主 Plan 执行台账和 Step 执行状态。

Step 01-07 独立于 01-06；只有当 01-06 被记录为 blocked 且无未提交完成工作时，才可按 Blocked 规则串行跳转到 01-07。Step 02-05 开放 dynamic request/timer 前必须先通过 component sandbox escape/resource-limit gate。Step 02-06 完成后必须执行 02-07 批次最终 Review 与整体验证。

不要启动 Phase 3。02-07 完成、提交并回填台账后结束本 Goal，报告 commit hash、验证证据、剩余风险和最终 git status。
```

## 3. Phase 0：基线冻结与产品化门槛

### 3.1 阶段整体计划

Phase 0 的目标不是新增功能，而是把当前 Demo 能力、缺口和上线门槛固化下来，避免后续开发变成零散 patch。

主要工作包：

- 冻结当前 P0.5 能力清单与验证命令；
- 建立 `wx.*` API 和组件兼容矩阵；
- 定义线上产品的 release gate、测试 gate、安全 gate；
- 将后续 Phase 拆成可创建 issue / milestone 的 backlog。

阶段产出物：

- `docs/architecture/wx-api-compatibility-matrix.md`：接口/API 支持等级、映射策略、错误语义、优先级。
- `docs/architecture/component-compatibility-matrix.md`：组件、事件、WXML、WXSS、动态组件支持等级。
- `docs/security/threat-model.md`：容器威胁模型与安全基线。
- `docs/runbook/release-gates.md`：每次 release 前必须通过的命令、fixture、审计项。

### 3.2 细分小阶段与实施方案

#### 0.1 当前能力盘点与基线固化

实施方案：

1. 将当前 workspace crate、CLI 命令、demo-server endpoint、coffee Skill 能力整理成表格。
2. 为每项能力标注证据：对应 crate、测试文件、runbook 命令或 demo 输出。
3. 将 `cargo metadata --format-version 1 --no-deps`、`cargo test --workspace`、`cargo fmt --check`、`cargo clippy --workspace --all-targets -- -D warnings` 设为基础 gate。
4. 在文档中明确当前 demo-only 能力，例如 localhost `wx.request` bridge、mock payment、mock consent provider、非生产 FastAPI 示例。

验收：

- 文档能回答“当前已经做到什么、由哪个 crate 负责、哪些测试证明”。
- 任何新功能都能挂到已有能力图或明确新增模块。

#### 0.2 建立接口兼容矩阵

实施方案：

1. 从 `docs/weichat-miniapp-mcp-protocol/weichat-miniapp-mcp.txt` 抽取 API 列表，按原子接口环境、原子组件环境、半屏页面环境分类。
2. 为每个 API 标注：`supported`、`host-boundary`、`planned-p1`、`planned-p2`、`unsupported-by-design`、`demo-only`。
3. 对每个 planned API 定义映射策略：
   - 原样 JS 语义；
   - Host capability；
   - ANP DID 替代；
   - mock / fallback；
   - deterministic unsupported error。
4. 定义 callback、Promise、`errMsg`、错误码、敏感字段返回规则。

验收：

- 每个小程序 MCP 列表中的 API 都有明确状态，不允许出现“未知”。
- 不支持的 API 有原因，不因兼容压力进入完整微信 Runtime 复刻。

#### 0.3 建立组件兼容矩阵

实施方案：

1. 将组件支持列表拆成内置组件、Component JS、WXML、WXSS、事件、`wx.modelContext`、动态组件。
2. 将当前 P0 支持能力和目标 P1/P2 能力分离。
3. 明确哪些能力只输出 Render IR，不要求当前提供完整 UI 展现。
4. 为每个能力绑定 fixture 或未来 golden snapshot。

验收：

- 交易型卡片必须能力和可延期能力区分清晰。
- Host renderer 不再阻塞 component runtime 的契约演进。

#### 0.4 定义 release gates 与 milestone backlog

实施方案：

1. 为每个 Phase 建立 Definition of Done。
2. 将每个小阶段拆成 issue 粒度：输入、输出、影响 crate、测试、文档。
3. 规定每次开发必须同步更新兼容矩阵和 runbook。
4. 建立失败回滚规则：任何新 API 默认 fail closed，不能静默 mock 成成功。

验收：

- 后续代码开发可以直接按 issue/milestone 执行。
- 文档、测试、runbook 与实现同步成为 release 条件。

## 4. Phase 1：接口对齐与 wx Capability Broker

### 4.1 阶段整体计划

Phase 1 的目标是补齐原子接口环境的核心能力，使容器底层对 Skill 暴露的接口尽量对齐小程序 MCP 和微信标准 API 的调用方式。重点不是一次性支持所有微信 API，而是建立统一的 `wx Capability Broker`，让已支持、后续支持和明确不支持的 API 都有一致语义。

主要工作包：

- 完整化 Skill package / manifest 校验；
- 扩展原子接口 JS bridge 的 `wx.modelContext`；
- 把 `wx.login` / `wx.checkSession` / `wx.request` / storage / payment / privacy APIs 纳入统一 capability broker；
- 对齐 callback / Promise / `errMsg` 语义；
- 引入接口级权限声明、allowlist 和风险分级。

阶段完成标志：

- coffee Skill 不依赖 demo-only 特殊路径也能通过统一 broker 运行；
- 一个新的交易型 Skill 可以使用登录、请求、存储、下单、支付确认、地址/手机号授权等核心能力；
- 不支持 API 返回稳定、可测试、可脱敏的错误，不会访问宿主资源。

### 4.2 细分小阶段与实施方案

#### 1.1 Skill package 与 manifest 对齐

实施方案：

1. 扩展 validator 支持小程序 MCP 关键约束：
   - `SKILL.md` 单文件和长度限制；
   - `mcp.json` 长度统计规则；
   - `apis[].name` 与注册名一致；
   - `inputSchema` 必须为对象；
   - `outputSchema` mismatch 作为 warning；
   - `format: "image" | "file"` 输入字段识别；
   - `_meta.ui.componentPath` 和 `components[].path` 关系；
   - `components[].relatedPage`、`permissions.scope.dynamic`、`expirable`、`expiredText`。
2. 增加可选 `app.json` / `AGENTS.md` 读取规划：
   - 不要求 P1 完整支持小程序分包；
   - 可识别 `agent.skills[]` 作为多 Skill registry 输入；
   - 保留单 Skill 目录作为当前默认路径。
3. 校验策略分层：
   - spec error：不能加载；
   - compatibility warning：可加载但报告降级；
   - production warning：demo 可用但上线不允许。

涉及模块：`mcp-schema`、`skill-loader`、`dock-cli validate`。

验收：

- `dock-cli validate` 输出机器可读兼容性报告。
- 每个 manifest warning 都有修复建议或明确降级行为。

#### 1.2 `wx.modelContext` 原子接口 API 对齐

实施方案：

1. 在 Atomic API VM 注入：
   - `wx.modelContext.getSessionId()`；
   - `wx.modelContext.expireAllCards({ componentPaths, match })`；
   - `wx.modelContext.NotificationType` 常量；
   - 未来多 Skill 场景下的 `createSkill(skillPath)` 路径校验。
2. 将卡片过期从组件 VM 行为扩展为 runtime-level card event，不直接耦合 CLI demo。
3. 规定 `expireAllCards` 的策略：
   - 只影响声明 `expirable: true` 的组件；
   - `componentPaths` 必须 canonicalize；
   - `match: latest` 与 all 行为明确；
   - 操作进入 audit。

涉及模块：`js-runtime-quickjs`、`wx-compat`、`dock-core`、`component-runtime`。

验收：

- 原子接口 JS 可直接调用上述 API。
- CLI 和测试能观察到 card expiration event。

#### 1.3 `wx.login` / `wx.checkSession` 生产化

实施方案：

1. 把当前 `HostDidAuthConfig` 提炼为正式 `DidAuthSessionManager`：
   - key：`merchantDid + userDid + agentDid + skillId + sessionId + serverBaseUrl`；
   - token 只由 host 持有，默认不暴露给 Skill；
   - 支持 token refresh、过期清理、强制登出、会话隔离。
2. 对齐微信语义：
   - `wx.login()` 返回 code-like receipt 与 `errMsg`；
   - `wx.checkSession()` 校验 session/token 状态；
   - callback 和 Promise 同时支持；
   - DID proof、token、Authorization 不进入模型可见输出。
3. 服务端契约固定：
   - challenge 字段包含 `challengeId`、`nonce`、`merchantDid`、`issuedAtMs`、`expiresAtMs`、`audience`；
   - login 请求携带 signed challenge；
   - 返回 scoped capability token 和 expiry；
   - replay、audience mismatch、scope mismatch 必须失败。

涉及模块：`anp-adapter`、`js-runtime-quickjs`、`demo-server`、`examples/coffee-fastapi-server`。

验收：

- 同一 session 重复 `wx.login` 使用缓存或刷新策略；
- token 不出现在 JS result、CLI output、日志和 audit 中；
- Rust demo-server 与 FastAPI 示例使用同一契约。

#### 1.4 `wx.request` 与网络能力对齐

实施方案：

1. 将当前 demo-only localhost bridge 替换/下沉到统一 `RequestBroker`：
   - 支持 method、headers、data、timeout、responseType、statusCode、header、data、errMsg；
   - 禁止 JS 传入 `Authorization` 覆盖 host token；
   - 默认 allowlist fail closed；
   - 自动附加 DID signature 或 cached bearer。
2. 区分请求类型：
   - business API：走 capability token；
   - login/challenge：走 DID signature；
   - public GET：可按策略匿名或签名；
   - upload/download：走文件 broker，不直接开放任意文件路径。
3. 对齐错误与重试：
   - 401 清 token 后可一次 challenge retry；
   - 网络错误返回 `request:fail ...`；
   - 非 2xx 仍走 success callback 还是 fail callback需按微信语义固定；
   - 所有错误输出脱敏。

涉及模块：`wx-compat`、`anp-adapter`、`js-runtime-quickjs`、`dock-core`。

验收：

- 非 allowlist 域名无网络出站；
- Skill 不能读取或覆盖 Authorization；
- 请求行为通过 fake transport 和 integration tests 覆盖。

#### 1.5 storage、文件、媒体与设备核心 API

实施方案：

1. 注入 storage API：
   - `wx.getStorage` / `setStorage` / `removeStorage` / `clearStorage`；
   - 同步版本 `getStorageSync` / `setStorageSync`；
   - batch API 可 P1.5；
   - scope 固定为 DID + merchant + Skill。
2. 文件/媒体 API 优先实现 host-boundary 形态：
   - `format:image/file` 入参只接收 host 提供的 opaque handle；
   - `wx.chooseMedia`、`wx.chooseMessageFile` 返回 host file handle，不暴露任意本地路径；
   - `wx.previewMedia` 在 Host 能力不足时返回 fallback。
3. 设备和系统 API：
   - `wx.getDeviceInfo` / `wx.getAppBaseInfo` 保持最小真实信息；
   - `wx.getNetworkType`、`wx.onNetworkStatusChange` 可先以 snapshot/no-op listener 支持；
   - 复杂传感器、WiFi、蓝牙、TCP/UDP 默认 unsupported-by-design。

验收：

- storage 隔离测试覆盖不同 DID、merchant、Skill。
- 文件/media API 不泄漏真实路径和隐私内容。

#### 1.6 隐私、地址、手机号、支付与高风险 API

实施方案：

1. 将高风险 API 统一接入 `ConsentGate`：
   - `wx.getPhoneNumber`；
   - `wx.chooseAddress`；
   - `wx.requestPayment` / `wx.requestVirtualPayment` / `wx.requestJointPayment` 的 ANP Payment Intent 映射；
   - `wx.openLocation` / `wx.chooseLocation`；
   - `wx.makePhoneCall` / `wx.scanCode`。
2. 默认实现分层：
   - production host：调用真实 host UI/系统能力；
   - headless/CLI：显式 mock provider，输出 mock 标识；
   - 未配置 provider：fail closed。
3. payment 不复刻微信支付收银台：
   - 以 Payment Intent + user consent + merchant API 为主线；
   - demo 可 mock pay，但必须保留风险等级、proof 和 audit。

验收：

- 未配置 consent/provider 时无法执行 L3/L4 API。
- 审计记录只包含脱敏摘要、proof id 和 digest。

#### 1.7 明确 unsupported API 策略

实施方案：

1. 对 `wx.cloud.*`、微信社交、广告、公众号/视频号/客服、WiFi、蓝牙、TCP、UDP、mDNS、传感器、人脸核身、完整地图交互等 API 建立 deterministic unsupported stub。
2. stub 返回：
   - `errMsg: "<api>:fail unsupported"`；
   - `reason`；
   - `suggestion`；
   - 不访问任何宿主资源。
3. 兼容矩阵中标注 unsupported-by-design 或 P2+。

验收：

- 任何未实现 API 都不会变成 `undefined is not a function` 这类不稳定错误。
- 业务开发者能从错误中知道如何 fallback。

## 5. Phase 2：组件运行时对齐

### 5.1 阶段整体计划

Phase 2 的目标是让组件运行时从 coffee P0 卡片子集扩展到小程序 MCP 原子组件的稳定核心子集。仍然不做完整 UI 展现，重点是 **Component VM + WXML/WXSS 子集 + Render IR contract** 足够稳定，Host 可以用 Flutter、SwiftUI、Web 或 native card adapter 渲染。

主要工作包：

- 组件 manifest 元数据对齐；
- Component JS 语义增强；
- WXML/WXSS 与内置组件支持扩展；
- 动态组件能力受控开放；
- Render IR 版本化和 snapshot tests；
- 多 fixture 兼容性套件。

阶段完成标志：

- `drink-list`、`order-confirm`、`payment-result` 之外，至少再加入 3 类真实交易/表单 Skill fixture；
- Render IR 有稳定 schema version；
- 动态组件 request/timer 只在声明权限后可用；
- Host renderer 不支持时可稳定 fallback 到 CardSpec。

### 5.2 细分小阶段与实施方案

#### 2.1 组件声明与生命周期元数据

实施方案：

1. 完整读取 `components[]` 字段：
   - `path`；
   - `relatedPage`；
   - `permissions.scope.dynamic`；
   - `expirable`；
   - `expiredText`；
   - `_meta` 扩展。
2. 读取 `index.json` 基础配置，至少保留 unknown fields。
3. 组件路径 canonicalize，保证 `api._meta.ui.componentPath` 与 `components[].path` 一致。
4. 组件实例增加元数据：component id、render id、created at、expiry state、related page state。

验收：

- 组件 manifest 信息能进入 RenderOutcome 或 card event。
- `expirable: false` 的组件不会被误过期。

#### 2.2 Component JS 语义增强

实施方案：

1. 增强 `properties` 类型处理：String、Number、Boolean、Object、Array、optional/default。
2. 增加 `this.triggerEvent()` P1 支持，将事件转换为 Render IR / Host event，不直接执行宿主动作。
3. 增加 `observers` / simple watchers P2 规划，P1 可先 warning。
4. 补齐 `NotificationType.Overflow` 和 view dimension 事件。
5. 统一 lifecycle trace：created、attached、result/input notification、event、setData、expire、detached。

验收：

- Component state update 和 Render IR refresh 有 snapshot。
- 不支持的 Component 选项有 warning，不静默忽略高风险行为。

#### 2.3 WXML 子集增强

实施方案：

1. 保持 P0 表达式简单，P1 增加常见表达式：
   - `wx:elif` / `wx:else`；
   - boolean not / equality；
   - string/number literal；
   - 简单三元表达式可 P2。
2. 增加事件和 dataset 语义：
   - `catchtap`；
   - `data-*` camelCase / original key 双表示；
   - disabled button 不触发 tap。
3. 增加内置组件：
   - 表单：`input`、`textarea`、`radio`、`checkbox`、`picker`；
   - 展示：`map` preview、`canvas` static；
   - 仍不支持 `video`、`web-view`、`navigator`、广告和社交 open-type。

验收：

- 每个新增 WXML 语法都有 parser test、compiler test 和 fixture snapshot。
- 表单组件只产生 host action / component state，不绕过 consent。

#### 2.4 WXSS 子集增强

实施方案：

1. P1 增加选择器：id、标签、简单后代选择器。
2. P1 增加属性：min/max width/height、box-shadow、gap、justify-content、align-items、overflow-x。
3. 维持禁止或降级：animation、transition、复杂 transform、filter、mask、自定义字体。
4. rpx 与 host logical pixels 规则文档化，避免不同 host 渲染不一致。

验收：

- unsupported WXSS 输出 warning，不影响安全渲染。
- Render IR style 字段保持跨 Host 中立。

#### 2.5 动态组件能力

实施方案：

1. 只有声明 `components[].permissions.scope.dynamic` 的组件可使用：
   - 受限 `wx.request`；
   - `setTimeout` / `setInterval` / clear；
   - 可选 polling helper。
2. 动态组件必须满足：
   - request allowlist；
   - timer 最大数量与频率限制；
   - expire/detach 后清理；
   - host background 后暂停；
   - 审计动态请求摘要。
3. 默认组件仍禁用网络、timer、WebSocket。

验收：

- 未声明 dynamic 的组件调用 request/timer 必须失败。
- expire 后 timer 不再触发。

#### 2.6 Render IR 版本化与 Host adapter contract

实施方案：

1. 为 Render IR 增加 schema version、node kind registry、action registry。
2. 规定 Host adapter 必须处理：
   - unknown node kind fallback；
   - unknown style warning；
   - action confirmation boundary；
   - accessibility fields。
3. 增加 golden snapshot tests：相同 input 生成稳定 Render IR。
4. 增加 CardSpec fallback contract：组件加载失败、WXML 解析失败、host 不支持 node、API error。

验收：

- Render IR 变更必须更新 schema version 或 migration notes。
- Host adapter 可以独立于 Component VM 开发和测试。

## 6. Phase 3：安全增强与可信执行

### 6.1 阶段整体计划

Phase 3 的目标是把安全能力从“Demo 中证明可行”升级为“线上默认安全”。后续新增任何 API 或组件能力都必须先通过本阶段定义的安全边界。

主要工作包：

- threat model 与安全基线；
- QuickJS 沙箱和资源限制加固；
- 权限策略引擎；
- DID / token 生命周期生产化；
- consent / audit 真实化；
- Skill 包供应链安全。

阶段完成标志：

- 高风险动作无法绕过 consent；
- 非 allowlist 网络、跨包 require、远程代码、私钥/令牌泄露都有自动化测试；
- audit 可落盘、可检索、可脱敏导出；
- Skill 包加载前可校验来源和完整性。

### 6.2 细分小阶段与实施方案

#### 3.1 Threat Model 与安全分级

实施方案：

1. 建立攻击面清单：Skill package、JS runtime、component runtime、request broker、storage、DID key、token cache、Host adapter、demo/server、logs。
2. 定义攻击者模型：恶意 Skill、被篡改 Skill 包、恶意商家 Agent、恶意 Host plugin、网络中间人、日志读取者。
3. 为每条威胁定义控制措施、测试和残余风险。
4. 将风险等级 L0-L4 与 API、manifest、consent、audit 绑定。

验收：

- 每个高风险 capability 都能在 threat model 中找到对应控制措施。

#### 3.2 QuickJS 沙箱加固

实施方案：

1. 审计 `eval` / `Function` / prototype constructor / async function constructor / generator constructor escape。
2. 增加资源限制：
   - memory hard limit；
   - stack limit；
   - CPU/interrupt timeout；
   - Promise job drain 上限；
   - console/log size 上限；
   - result size 上限。
3. 将 API VM 和 Component VM 的 sandbox policy 文档化并测试。
4. 禁止远程代码、任意文件系统、socket、WebSocket，除非 capability broker 明确开放。

验收：

- sandbox escape tests 作为 CI gate。
- 超限行为返回稳定错误并记录 audit。

#### 3.3 权限策略与 allowlist

实施方案：

1. 从 `mcp.json`、`_meta.anp`、`x_anp`、`components[].permissions` 推导权限。
2. 支持宿主级 policy override：allow、deny、mock、prompt。
3. 网络 allowlist 支持：scheme、host、port、path prefix、method、scope。
4. storage、file、media、location、phone、address、payment 全部通过 capability broker。
5. 未声明权限但调用敏感能力时 fail closed。

验收：

- 任何 capability 都有 permission decision 记录。
- CLI/headless mock 必须显式开启，不能默默通过。

#### 3.4 DID、token 与会话安全

实施方案：

1. token claims 固定：issuer、audience、merchantDid、userDid、agentDid、skillId、sessionId、scopes、iat/nbf/exp、jti、version。
2. 支持 token refresh、revoke、logout、cache eviction。
3. challenge 防 replay：nonce 一次性、TTL、audience、method/url、DID document binding。
4. DID document resolver 生产化：cache、TTL、trust anchors、network failure policy。
5. 私钥只在 host/credential provider 边界使用，永不进入 JS 或日志。

验收：

- replay、wrong audience、wrong scope、expired token、wrong DID document 全部有测试。
- token 轮换不破坏 running Skill session。

#### 3.5 Consent 与审计生产化

实施方案：

1. 把 mock consent provider 抽象为 host consent adapter：CLI、Mac、Flutter、server-side headless policy 可分别实现。
2. ConsentProof 增加不可抵赖所需字段：policy version、UI prompt digest、decision actor、timestamp、parameter digest。
3. Audit sink 支持持久化：SQLite / file append / remote audit service，至少一个生产候选实现。
4. 审计查询必须默认脱敏，原始敏感字段不落盘或加密保存。
5. 建立 redaction regression tests：Authorization、Signature、token、secret、private key、phone、address、file content。

验收：

- 支付/下单/地址/手机号 API 没有 consent proof 不会执行。
- `GET /audit` 或导出接口永不泄露 token/signature。

#### 3.6 Skill 包完整性与供应链

实施方案：

1. 支持 Skill package digest、签名、版本、publisher DID。
2. 下载/缓存 Skill 时校验 digest 和 path boundary。
3. 禁止 symlink escape、absolute path、remote require。
4. 记录 package source、version、digest 到 audit。
5. 为第三方 Skill 建立 quarantine / review / allowlist 流程。

验收：

- 篡改包、路径穿越、签名不匹配、未知 publisher 默认无法加载。

## 7. Phase 4：生产运行时与 Host 接入

### 7.1 阶段整体计划

Phase 4 的目标是把 CLI/demo-server 形态升级为可被真实宿主集成和线上部署的容器。这里仍不要求做完整 UI，只要求稳定的 runtime API、进程边界、持久化和 Host adapter contract。

主要工作包：

- Runtime 公共 API / SDK；
- Skill 发现、下载、缓存、版本管理；
- session/token/storage/audit 持久化；
- 本地进程或嵌入式 SDK 形态；
- Host renderer / action protocol；
- 并发、取消、重试和幂等。

阶段完成标志：

- 一个真实 Host 可以通过稳定协议调用容器加载 Skill、执行 API、渲染 Render IR、处理 action；
- 容器重启后 session/storage/audit 能按策略恢复；
- 多用户、多商家、多 Skill session 隔离通过测试。

### 7.2 细分小阶段与实施方案

#### 4.1 Runtime API 稳定化

实施方案：

1. 定义 public Rust API：load skill、validate、call api、render component、dispatch action、expire cards、query audit。
2. 定义可选 IPC API：HTTP/gRPC/JSON-RPC，以便非 Rust Host 接入。
3. 保持 API 输入输出模型稳定并版本化。
4. 将 CLI 改为调用同一 Runtime API，避免 CLI 逻辑成为第二套 runtime。

验收：

- CLI、Mac host、未来 Flutter host 共用一套 runtime contract。

#### 4.2 Skill 发现、获取与版本管理

实施方案：

1. 对接 ANP Agent registry / merchant manifest：发现 merchant DID、Skill manifest URL、package digest、auth endpoints。
2. 支持本地 cache：按 publisher DID + skill id + version + digest 存储。
3. 支持版本选择策略：latest、pinned、allow prerelease、rollback。
4. package.zip 从 no-op 变成真实服务路径，但加载仍先解包到安全隔离目录。

验收：

- 相同 digest 可复用缓存；digest mismatch 会拒绝加载。
- 可回滚到上一个已验证 Skill version。

#### 4.3 持久化与配置

实施方案：

1. session/token cache：生产可用 secure store 或加密 SQLite。
2. scoped storage：持久化 backend，按 DID/merchant/Skill 隔离，支持 quota。
3. audit：append-only 或 SQLite backend，支持 retention policy。
4. 配置项：identity、trusted DID、allowlist、token issuer、storage path、log level、mock providers。
5. secrets：env/secret store 注入，不写入 config 文件或日志。

验收：

- 重启后可恢复非过期 token 和 storage。
- 删除用户/Skill 数据能按 scope 清理。

#### 4.4 Host renderer 与 action protocol

实施方案：

1. 定义 Host 需要实现的最小协议：
   - render Render IR；
   - render CardSpec fallback；
   - request consent；
   - handle phone/address/media/file/payment/location providers；
   - dispatch user events back to container。
2. action 必须回到 `dock-core`，不允许组件直接调用高风险 host 操作。
3. 定义 headless 模式：只输出 JSON，不做 UI；用于 CI 和 server-side agent。
4. Mac/Flutter/Web adapter 可作为参考实现，但不是容器核心。

验收：

- Render IR snapshot 可被至少一个 adapter 渲染或安全 fallback。
- 用户点击后 action flow 保持 audit/consent 边界。

#### 4.5 并发、取消、重试与幂等

实施方案：

1. 每个 session 支持多个并发 API 调用，但同一高风险交易可按 policy 串行化。
2. API call 支持 cancellation token 和 timeout。
3. request broker 支持 retry policy，但支付/下单等非幂等 API 默认不自动重试。
4. order/payment API 建议引入 idempotency key。
5. 组件过期和 session 结束会取消动态请求/timer。

验收：

- 并发场景不会串 session/token/storage。
- 取消后不会继续执行高风险 action。

## 8. Phase 5：开发者体验与生态兼容

### 8.1 阶段整体计划

Phase 5 的目标是让外部 Skill 开发者能理解容器能力、导入 Skill、自助调试并获得兼容性报告。

主要工作包：

- CLI / SDK 工具链；
- 兼容性报告；
- 示例 Skill 和迁移指南；
- 本地调试器；
- 文档与模板。

阶段完成标志：

- 开发者可用一个命令验证 Skill 是否可在容器内上线；
- 兼容性报告能指出哪些 API/组件会降级或失败；
- 至少有 coffee 之外的多个示例覆盖不同能力。

### 8.2 细分小阶段与实施方案

#### 5.1 CLI 命令扩展

实施方案：

1. `dock-cli validate` 输出 compatibility level、warnings、unsupported API、component fallback risk。
2. `dock-cli inspect` 展示 Skill package、权限、风险 API、组件树、依赖。
3. `dock-cli test-skill` 执行 fixture inputs、snapshot Render IR、审计输出。
4. `dock-cli import-wechat-mcp` 将小程序 MCP Skill 目录转换/复制为容器可验证结构，但不强制修改原字段。
5. `dock-cli doctor` 检查 DID identity、host providers、allowlist、storage、sandbox。

验收：

- CLI 输出全 JSON，可用于 CI。
- 所有敏感值默认 redacted。

#### 5.2 示例 Skill 与兼容测试集

实施方案：

1. 保留 coffee Skill 作为交易流程基线。
2. 新增至少三类 fixture：
   - 表单/地址/手机号授权；
   - 图片/文件输入处理；
   - 动态组件/状态刷新；
   - 可选地图/位置预览。
3. 每个 fixture 包含 `SKILL.md`、`mcp.json`、API JS、组件、expected result、Render IR snapshot。

验收：

- 每类核心能力都有可复制的示例。
- 新增兼容能力必须先新增或更新 fixture。

#### 5.3 文档与迁移指南

实施方案：

1. 编写“从小程序 MCP Skill 迁移到 ANP MiniApp Dock”的指南。
2. 编写 API 对齐表和组件对齐表，标明 ANP 替代实现。
3. 编写安全开发指南：不要存 token、不要在 content 暴露隐私、如何声明权限、如何处理 fallback。
4. 编写 Host adapter 开发指南。

验收：

- 开发者无需阅读 Rust 源码也能完成 Skill 验证。
- 文档和 CLI 报告术语一致。

## 9. Phase 6：观测、性能与发布运营

### 9.1 阶段整体计划

Phase 6 的目标是让容器具备线上运行所需的可观测性、性能边界、发布门禁和故障处理流程。

主要工作包：

- 结构化日志、metrics、traces；
- 性能和资源基准；
- CI/CD gates；
- release/canary/rollback；
- 线上 runbook。

阶段完成标志：

- 线上问题能通过 session id、skill id、merchant DID、api name 定位；
- 任意 release 都可回滚；
- 性能退化和敏感信息泄露能在 CI 阶段发现。

### 9.2 细分小阶段与实施方案

#### 6.1 可观测性

实施方案：

1. 统一结构化事件：skill_load、api_call_start/end、request_start/end、consent_prompt/decision、render_start/end、component_event、audit_record。
2. metrics：API latency、VM time、render time、request status、fallback rate、consent required/approved/denied、sandbox timeout、memory limit hit。
3. traces：同一用户请求贯穿 model decision、API call、request、render、action。
4. 所有 logs 默认 redacted。

验收：

- 可以在不看敏感 payload 的情况下定位失败阶段。

#### 6.2 性能与容量

实施方案：

1. 建立基准：Skill load time、API call latency、component render latency、memory per VM、token cache lookup。
2. 加入 stress tests：并发 session、多 Skill、多组件渲染、动态组件 timer。
3. 定义资源限制默认值和可配置范围。
4. 对生产 host 建议 warm cache，但保持每次调用安全上下文隔离。

验收：

- 性能基准写入 release notes。
- 超过资源限制时 fail closed，不影响其他 session。

#### 6.3 CI/CD 与 release gates

实施方案：

1. CI 必跑：fmt、clippy、unit、integration、sandbox escape、compat fixture、redaction、snapshot。
2. release 前必跑：完整 demo、多个 fixture、security checklist、docs link check。
3. 版本策略：runtime API、Render IR、capability token、Skill package contract 分别版本化。
4. 建立 canary：先在 headless/CLI 和内部 Host 跑，再开放外部 Skill。

验收：

- 任一 gate 失败不得发布。
- breaking change 必须有 migration note。

#### 6.4 运维 runbook

实施方案：

1. 编写部署、配置、identity、secret、storage、audit、升级、回滚文档。
2. 编写常见故障处理：DID 验签失败、token scope mismatch、allowlist deny、component render failed、consent required、sandbox timeout。
3. 定义数据清理与用户隐私删除流程。

验收：

- 运维人员能独立判断是 Skill 问题、商家 Agent 问题、Host provider 问题还是容器问题。

## 10. 能力优先级建议

后续实际开发应按以下优先级推进。

### 10.1 必须优先补齐

1. `wx.login` / `wx.checkSession` / `wx.request` / storage 的正式 JS bridge 与统一 capability broker。
2. `wx.modelContext.getSessionId`、`expireAllCards`、card event 的 runtime 化。
3. `components[].relatedPage`、`expirable`、`expiredText`、`permissions.scope.dynamic` 的完整读取与行为。
4. 高风险 API 的 consent/audit 强制路径。
5. API/组件兼容矩阵与 CLI 兼容报告。
6. sandbox escape、allowlist、redaction、token replay/scope 的 CI gate。

### 10.2 第二优先级

1. `format:image/file`、`chooseMedia`、`previewMedia`、`uploadFile` / `downloadFile` / `openDocument` 的 host-boundary 形态。
2. `chooseAddress`、`getPhoneNumber`、`requestPayment` 的 production provider 接口。
3. 表单组件、`openDetailPage` fallback、`Overflow`、动态组件 request/timer。
4. Skill package digest/signature/cache。
5. persistent storage/token/audit。

### 10.3 可后置或明确不做

1. 完整微信页面路由、TabBar、多页面生命周期。
2. 完整半屏小程序页面能力。
3. 微信云开发、微信支付收银台、微信社交生态 API。
4. 蓝牙、WiFi、TCP、UDP、mDNS、复杂传感器、完整地图交互。
5. 完整 WXML/WXSS 和完整自定义组件系统。

## 11. 阶段验收总表

| 能力方向 | Demo 原型可接受 | 线上产品必须达到 |
|---|---|---|
| Skill 加载 | 单 coffee Skill，本地目录 | 多 Skill、版本、digest、签名、缓存、兼容报告 |
| 原子接口 | P0 createSkill/registerAPI/use | 核心 `wx.modelContext` 与 `wx.*` JS bridge 对齐，unsupported API 稳定失败 |
| 网络 | localhost demo bridge | allowlist + DID signature + scoped bearer + retry/refresh + redaction |
| 身份 | 示例 DID 文件 | host credential provider、DID resolver、token lifecycle、secret store |
| 组件 | coffee 三卡片 | 组件矩阵 P1、Render IR version、动态组件受控、snapshot fixtures |
| 安全 | 单元测试覆盖关键点 | threat model、sandbox gates、package signing、audit persistence、CI fail closed |
| Host | CLI/Mac demo | 稳定 Runtime API / IPC、Host adapter contract、headless mode |
| 运维 | 本地 runbook | metrics/logs/traces、release gates、rollback、privacy deletion |

## 12. 立即下一步建议

如果下一轮开始进入代码开发，建议按以下顺序开工：

1. 先完成 Phase 0 的三份基础文档：`wx-api-compatibility-matrix.md`、`component-compatibility-matrix.md`、`threat-model.md`。
2. 从 Phase 1.1 与 Phase 1.2 开始：增强 validator 与 `wx.modelContext` JS bridge，因为它们影响面清晰、风险较低、会为后续 API 对齐提供骨架。
3. 同步把当前 demo-only `wx.login` / `wx.request` 代码收敛为正式 `DidAuthSessionManager` 和 `RequestBroker` 注入路径。
4. 每补一个 API 或组件能力，必须同时补：兼容矩阵、unit test、fixture 或 snapshot、runbook/CLI 输出。
5. 在 Phase 1/2 期间就启动 Phase 3 threat model，避免后续因为安全边界不清晰返工。
