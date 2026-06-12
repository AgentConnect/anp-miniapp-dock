# Step 01-03：`wx.modelContext` 原子接口桥接

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：01-03
状态：draft

## 1. 执行状态

| 字段 | 值 |
|---|---|
| Status | pending |
| Branch | 待执行时记录 |
| Started | 待记录 |
| Completed | 待记录 |
| Commit | 待记录 |
| Review evidence | 待记录 |
| Verification evidence | 待记录 |
| Next action | 等待 Step 01-01、01-02 完成后，实现 `wx.modelContext` 原子接口能力 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：在 Atomic API VM 中注入 `wx.modelContext.getSessionId()`、`wx.modelContext.expireAllCards(options)` 和 `wx.modelContext.NotificationType`，并加强 `createSkill(skillPath)` 路径校验和多 Skill 预留。
- 用户 / 系统可见行为：Skill 原子接口 JS 可读取 session id、发出 card expiration event，CLI/tests 可观察 card event/audit，非法 componentPath fail closed。
- 非目标：不实现完整多 Skill registry；不实现 UI 页面路由；不把 card state 直接暴露给 JS 修改。
- 完成标准：API VM 和 Component VM 的 NotificationType 不漂移；`expireAllCards` 变成 runtime-level event；相关 tests 和矩阵同步完成。

## 3. 设计方法

- 设计边界：`wx.modelContext` 是 Skill 与 runtime 的受控桥，不能让 JS 直接修改 Host 状态。
- 核心决策：`expireAllCards` 只生成 runtime card event；`componentPaths` 必须 canonicalize；只影响声明 `expirable: true` 的组件；操作进入 audit。
- 契约 / API / 数据流：Atomic API JS 调用 -> JS wrapper -> Rust bridge / broker -> `dock-core` card event routing -> audit / CLI observable output。
- 兼容性：NotificationType 常量与 Component Runtime 保持一致；`createSkill(skillPath)` 保留现有能力但加强边界。
- 风险控制：非法 componentPath、未声明 expirable、跨包路径、未知 match 策略全部 fail closed 或 deterministic warning。

## 4. 实现方法

1. 阅读 `anp/anp-miniapp-dock/crates/js-runtime-quickjs` 中 Atomic API VM 注入路径和 tests。
2. 阅读 `anp/anp-miniapp-dock/crates/wx-compat` 的 model context/card expiration helper。
3. 阅读 `anp/anp-miniapp-dock/crates/dock-core`、`anp/anp-miniapp-dock/crates/component-runtime` 中 card event、component metadata、Render IR action 回流。
4. 按 Step 01-01 contract 注入 `getSessionId`、`expireAllCards`、NotificationType，并将 outcome 统一为 `WxApiOutcome` 或当前等价结构。
5. 增加 tests：session id 返回、invalid path deny、latest/all match、expirable filter、audit/card event 可观察、NotificationType 一致性。
6. 更新 `wx-api-compatibility-matrix.md`、必要 runbook/README、Phase 1 文档状态。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/js-runtime-quickjs` | Atomic API VM 注入与 tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/wx-compat` | model context / card event helper | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/dock-core` | card event routing / audit 接入 | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/component-runtime` | NotificationType 一致性或 metadata | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/dock-cli` | 如需输出 card event 证据 | 视实现影响 |
| `anp/anp-miniapp-dock/docs/architecture/wx-api-compatibility-matrix.md` | 同步状态 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/01-03-model-context-bridge.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 01-01、Step 01-02。
- 外部文档或决策：Phase 1.2、bridge contract、API/组件矩阵。
- 环境前提：Rust toolchain 1.88.0；无需外部 merchant server。

## 7. 验收标准

- [ ] Atomic API JS 可调用 `wx.modelContext.getSessionId()` 并获得当前 session id。
- [ ] `wx.modelContext.expireAllCards(options)` 生成 runtime card event，不直接在 JS 内修改 card state。
- [ ] `componentPaths` canonicalize，非法路径 fail closed。
- [ ] `NotificationType` 与 Component VM 常量一致，有测试防漂移。
- [ ] 操作进入 audit 或可审计事件摘要，且不含 token/private data。
- [ ] `wx-api-compatibility-matrix.md` 与实现状态同步。
- [ ] Review 发现已修复或明确记录。
- [ ] 本步骤在进入下一步之前已创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Focused tests | `cd anp/anp-miniapp-dock && cargo test -p js-runtime-quickjs -p wx-compat -p dock-core model_context` | 相关测试通过；若 test filter 不匹配，记录实际命令 |
| Coffee E2E 回归 | `cd anp/anp-miniapp-dock && cargo test -p dock-cli --test coffee_order_flow` | 通过 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/js-runtime-quickjs crates/wx-compat crates/dock-core crates/component-runtime docs/architecture docs/plan` | 无空白错误 |
| 安全抽样 | 手工检查 card event/audit 输出 | 不含 token、Authorization、private key path |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：JS 是否能绕过 runtime card manager；path canonicalization 是否正确；NotificationType 是否单源或有防漂移测试；audit 是否脱敏；coffee flow 是否回归。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 `wx.modelContext` bridge、直接 tests 和文档同步。
- Commit 前状态：记录 `git status --short`。
- Commit 后证据：记录 commit hash 和 `git status --short --branch`。
- 建议消息：`phase1: add model context bridge`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 01-03 小 Plan | 将 Phase 1.2 modelContext 对齐拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：card expiration 事件如果与 Component Runtime 当前行为耦合过紧，会影响 Phase 2 扩展。
- 回滚 / 回退：保持 runtime event 结构向后兼容；如必须变更公开输出，先更新 contract 和矩阵。
- 后续文档：Phase 2 组件 action 和 Render IR fixture 将依赖此 card event 语义。
