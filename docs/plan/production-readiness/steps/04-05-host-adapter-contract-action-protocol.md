# Step 04-05：Host Adapter Contract 与 Action Protocol

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：04-05
状态：pending

## 1. 执行状态

| 字段 | 值 |
|---|---|
| Status | pending |
| Branch | `main` |
| Started | 待记录 |
| Completed | 待记录 |
| Commit | 待记录 |
| Review evidence | 待记录 |
| Verification evidence | 待记录 |
| Next action | 等待 04-04 完成后，启动 Host adapter contract |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：定义 Host adapter 必须实现或声明 unsupported 的 renderer、fallback、consent、provider、event dispatch 和 secure identity contract。
- 用户 / 系统可见行为：至少一个 headless/mock Host adapter 可通过 conformance tests 渲染 Render IR 或安全 fallback，并保持 action -> Orchestrator -> consent/audit 边界。
- 非目标：不实现完整 Mac/Flutter/Web UI；不复刻微信页面路由或 TabBar。
- 完成标准：Host 不认识的 node/action fail closed 或 fallback，不允许直接把组件 action 变成高风险系统调用。

## 3. 设计方法

- 设计边界：Host 负责展示和真实设备/provider UI，但不能绕过 Runtime 的 permission、ConsentGate、audit、redaction。
- 核心决策：Render IR renderer、CardSpec fallback renderer、consent prompt、phone/address/media/file/location/payment providers、openDetailPage、event dispatch、identity provider 都要有 conformance contract。
- 契约 / API / 数据流：Runtime RenderOutcome -> Host renderer -> user event -> Runtime dispatch action -> Orchestrator/API/provider -> audit。
- 兼容性：Headless adapter 用于 CI；真实 Host adapter 可以逐步实现，未支持 capability 必须声明 unsupported。
- 风险控制：Host unknown action 不执行；高风险 action 必须回 Runtime；external URL/path canonicalize。

## 4. 实现方法

1. 阅读 Render IR contract、component matrix、Step 01-08 high-risk provider boundary、Step 03-05 consent adapter 和 Step 04-01 Runtime API。
2. 定义 Host adapter trait / conformance spec，列出 required/optional/unsupported-by-design capability。
3. 实现 headless/mock Host adapter conformance harness，覆盖 render、fallback、consent required、provider unavailable、event dispatch。
4. 增加 tests：unknown node/action fallback、api/call 回 Orchestrator、high-risk provider cannot bypass consent、openDetailPage URL canonicalize、redaction。
5. 更新 Phase 4 文档、组件矩阵、developer Host adapter guide 计划和 release gates。
6. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-core` | Host adapter trait / action dispatch conformance | 代码实现 |
| `anp/anp-miniapp-dock/crates/component-runtime` | Render IR / event dispatch contract tests | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/card-spec` | fallback renderer contract | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/dock-cli` | headless Host adapter / preview conformance | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/architecture/component-compatibility-matrix.md` | 同步 Host renderer/action boundary | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | Host conformance gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-4-runtime-host-integration.md` | 同步 Host adapter contract | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/04-05-host-adapter-contract-action-protocol.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 01-08、Step 02-06、Step 03-05、Step 04-01。
- 外部文档或决策：Render IR contract、Host provider boundary、Runtime API。
- 环境前提：Rust toolchain 1.88.0；真实 UI Host 可后置。

## 7. 验收标准

- [ ] Host adapter contract 明确 required/optional/unsupported capability。
- [ ] Headless/mock adapter 通过 render/fallback/action/consent/provider conformance tests。
- [ ] Unknown node/action fail closed 或 fallback，不静默执行。
- [ ] `api/call`、payment、phone、address、location、file/media 等高风险 action 必须回 Runtime/Orchestrator。
- [ ] openDetailPage、external URL/path 有 canonicalize 和 deny tests。
- [ ] 组件矩阵、Release Gates 和 Phase 4 文档与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Host conformance tests | `cd anp/anp-miniapp-dock && cargo test -p dock-core host` | Host adapter/action tests 通过 |
| Component/action tests | `cd anp/anp-miniapp-dock && cargo test -p component-runtime action` | Render event/action tests 通过；若 filter 不匹配，记录实际命令 |
| CLI/headless tests | `cd anp/anp-miniapp-dock && cargo test -p dock-cli --test coffee_order_flow` | headless flow 回归通过 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-core crates/component-runtime crates/card-spec crates/dock-cli docs/architecture docs/runbook docs/plan` | 无空白错误 |
| 安全抽样 | 手工检查 Host action/result payload | 不含 token、Authorization、signature、private key path 或隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：Host 是否可能绕过 Runtime；unknown action 是否 fail closed；高风险 provider 是否 consent-first；fallback 是否保持 redaction。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 Host adapter contract/action protocol、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase4: define host adapter action protocol`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 04-05 小 Plan | 将 Host Adapter Contract 与 Action Protocol 拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：Host adapter 一旦直接执行 action，会绕过容器的安全边界。
- 回滚 / 回退：未通过 conformance 的 Host capability 必须声明 unsupported 并 fallback。
- 后续文档：Phase 5 Host adapter guide 和 Phase 6 runbook 必须引用本 contract。
