# Step 02-05：Dynamic component controls

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：02-05
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
| Next action | 等待 Step 02-04 完成后，启动 dynamic component controls |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：在组件声明 `permissions.scope.dynamic` 时，受限开放 component-side `wx.request`、timer 和 lifecycle cleanup；未声明时继续默认 deny。
- 用户 / 系统可见行为：动态组件可做受控 status polling；过期、detach 或 Host background 时 request/timer 被清理或暂停。
- 非目标：不开放 `fetch`、WebSocket、任意网络、无限 timer、后台常驻任务或真实 Host background scheduler。
- 完成标准：dynamic 权限、allowlist、token boundary、资源限制、audit summary 和 expire/detach cleanup 都有 tests。

## 3. 设计方法

- 设计边界：动态能力只面向声明了 `scope.dynamic` 的组件；默认 component sandbox 仍无网络、timer、WebSocket、Function/eval。
- 核心决策：component `wx.request` 复用 Phase 1 RequestBroker 和 JS auth header fail-closed 规则；timer 有数量、频率、生命周期限制。
- 契约 / API / 数据流：manifest dynamic metadata -> component capability profile -> Component VM inject limited APIs -> RequestBroker / timer scheduler -> audit summary / RenderOutcome refresh。
- 兼容性：非 dynamic 组件现有 tests 必须继续证明 request/timer 默认 deny。
- 风险控制：expire/detach 后清理 timers 和 pending actions；dynamic request 不得返回 token、Authorization、signature 或 Host private metadata。

## 4. 实现方法

1. 阅读 Step 01-04 `wx.request` 路径、Step 02-02 dynamic metadata flow 和组件矩阵 dynamic 部分。
2. 在 `component-runtime` 中按 capability profile 条件注入受限 `wx.request` 和 timer APIs。
3. 将 component-side request 接到 `wx-compat::RequestBroker` 或当前等价 broker，保持 allowlist、DID/session、redaction 和 JS auth header fail closed。
4. 实现 timer 数量/频率限制、clear、expire/detach cleanup 和 Host background pause 的 runtime hook 或记录为 host-boundary。
5. 增加 tests：默认 deny、dynamic allow、non-allowlist deny、Authorization header deny、timer limit、clear、expire/detach cleanup、audit summary redacted。
6. 更新组件兼容矩阵、Threat Model、Release Gates 和 Phase 2 文档。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/component-runtime` | dynamic wx.request、timer、cleanup、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/wx-compat` | component capability profile / request permission | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/js-runtime-quickjs` | 如需共用 wx request wrapper | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/anp-adapter` | RequestBroker 回归 tests | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/architecture/component-compatibility-matrix.md` | 同步 dynamic 状态 | 必须 |
| `anp/anp-miniapp-dock/docs/security/threat-model.md` | 同步 dynamic sandbox 风险 | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | 同步 dynamic gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-2-component-runtime-alignment.md` | 同步 dynamic 完成状态 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/02-05-dynamic-component-controls.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 01-04、Step 02-02。
- 外部文档或决策：RequestBroker contract、component dynamic metadata、Threat Model、Release Gates。
- 环境前提：Rust toolchain 1.88.0；无需生产 Host background scheduler。

## 7. 验收标准

- [ ] 未声明 `permissions.scope.dynamic` 的组件默认无法使用 `wx.request`、`setTimeout`、`setInterval`。
- [ ] 声明 dynamic 的组件只能使用受限 `wx.request`，且继续经过 allowlist、DID/session、JS auth header fail closed、redaction 和 audit。
- [ ] timer 有数量和频率限制，`clearTimeout` / `clearInterval` 生效。
- [ ] expire/detach 后 pending timers、dynamic request callbacks 和后续 actions 被清理或拒绝。
- [ ] dynamic audit summary 脱敏，Host background pause 若未实现则记录为 host-boundary 而非 production-ready。
- [ ] 组件兼容矩阵、Threat Model、Release Gates 与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Dynamic tests | `cd anp/anp-miniapp-dock && cargo test -p component-runtime dynamic` | dynamic request/timer/cleanup tests 通过；若 filter 不匹配，记录实际命令 |
| Compat / request tests | `cd anp/anp-miniapp-dock && cargo test -p wx-compat component_permissions && cargo test -p anp-adapter request` | permission 和 RequestBroker 回归通过 |
| Workspace 回归 | `cd anp/anp-miniapp-dock && cargo test --workspace` | 通过；如耗时受限，记录 focused 替代和风险 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/component-runtime crates/wx-compat crates/js-runtime-quickjs crates/anp-adapter docs/architecture docs/runbook docs/security docs/plan` | 无空白错误 |
| 安全抽样 | 手工检查 dynamic request/timer/audit output | 不含 token、Authorization、signature、private key path 或隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：默认 deny 是否保留；dynamic request 是否复用 RequestBroker；timer 是否有限制和 cleanup；expire 后是否无法继续触发高风险 action；audit 是否 redacted。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 dynamic component controls、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase2: add dynamic component controls`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 02-05 小 Plan | 将 Phase 2 dynamic component controls 拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：dynamic request/timer 是 component sandbox 的最大扩展点，容易绕过 RequestBroker 或造成资源耗尽。
- 回滚 / 回退：任何边界不清时保持默认 deny；dynamic feature 可通过 profile gate 关闭。
- 后续文档：Phase 4 Host background lifecycle 和 production RequestBroker transport 需要接入本 Step 的 boundary。
