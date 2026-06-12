# Step 06-05：Canary 发布、版本化与回滚策略

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：06-05
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
| Next action | 等待 06-04 完成后，启动 canary/release/rollback |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：定义 runtime API、Render IR、capability token、Skill package contract、Host adapter contract 的版本化和 canary/rollback 流程。
- 用户 / 系统可见行为：release 可以先 headless/internal canary，再 allowlisted merchant rollout；gate breach 可回滚并清理 cache。
- 非目标：不接入真实生产发布平台；不实现所有 Host 的 rollout UI。
- 完成标准：release notes 模板、canary checklist、rollback conditions、cache purge 和 migration note 规则可执行。

## 3. 设计方法

- 设计边界：发布流程以 gate report、metrics、fallback/error/consent/token 指标为准，不靠人工乐观判断。
- 核心决策：版本化对象独立管理；breaking change 必须 migration note；rollback 条件包括 token leakage、consent bypass、sandbox escape、fallback spike、auth failure spike、Host crash、audit failure。
- 契约 / API / 数据流：gate report -> release candidate -> headless canary -> internal Host -> allowlisted merchant -> expanded rollout -> monitor -> rollback/purge。
- 兼容性：Skill package version pin/rollback 复用 Step 04-03；Runtime/Render IR version 复用 Step 04-01/02-01。
- 风险控制：rollback 不删除审计；cache purge 保留 evidence；release notes 不含 secret。

## 4. 实现方法

1. 阅读 Phase 6 release strategy、Runtime API version、Render IR schema、token version、Skill package contract、Host adapter contract。
2. 定义 release notes 模板：version、compat changes、security changes、risk、migration、rollback。
3. 定义 canary stages：headless fixture、internal Host、allowlisted merchant Skill、publisher DID/skill version 扩展。
4. 定义 rollback conditions 和 actions：disable skill version、revert runtime, purge cache, revoke token, stop rollout。
5. 增加 scripts/checklists 或 docs：release checklist、rollback checklist、cache purge procedure。
6. 更新 Release Gates、Phase 6 文档和 runbook。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | release/canary准入和 rollback 条件 | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/release-process.md` | 新增 release/canary/rollback runbook | 计划新增 |
| `anp/anp-miniapp-dock/docs/runbook/rollback.md` | 如单独拆分 rollback 流程 | 视文档组织新增 |
| `anp/anp-miniapp-dock/scripts` | release notes/checklist helper | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-6-observability-release.md` | 同步发布策略 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/06-05-canary-release-rollback.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 04-03、Step 04-05、Step 06-02、Step 06-03、Step 06-04。
- 外部文档或决策：Runtime/Render IR/token/package/Host adapter versions、Release Gates。
- 环境前提：真实 production release 平台可后置；runbook/checklist 先可执行。

## 7. 验收标准

- [ ] Release notes 模板包含版本、兼容变化、安全变化、风险、migration、rollback。
- [ ] Canary stages 和准入条件明确，依赖 gate report 和 metrics。
- [ ] Rollback conditions 覆盖 token leakage、consent bypass、sandbox escape、fallback spike、auth failure spike、Host crash、audit failure。
- [ ] Rollback actions 覆盖 runtime revert、Skill version disable/rollback、cache purge、token revoke、rollout stop。
- [ ] Runbook 明确 audit evidence 保留和 secret redaction。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- docs/runbook docs/plan scripts` | 无空白错误 |
| 链接检查 | 手工检查 release/rollback runbook links | 链接目标存在 |
| Checklist dry-run | 按 release checklist 用当前 commit 做一次文档 dry-run | 每项有 pass/fail/skip 记录方式 |
| 安全抽样 | `cd anp/anp-miniapp-dock && rg -n "token|Authorization|private key|secret|rollback|audit|canary" docs/runbook docs/plan/production-readiness/phase-6-observability-release.md` | 命中安全规则和流程，不含真实 secret |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：runbook/checklist 文档同步完成后、commit 前。
- Review 重点：rollback 是否可执行；gate breach 是否 hard stop；版本化对象是否齐全；是否保留 audit evidence。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：验证、Review、文档同步完成后。
- Commit 范围：只包含 canary/release/rollback docs/scripts 和计划回填。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`docs: add canary rollback release process`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 06-05 小 Plan | 将 Canary 发布、版本化与回滚策略拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：没有真实平台接入时，流程可能停留在文档。
- 回滚 / 回退：先让 checklist 可本地 dry-run；真实平台接入作为后续 Host/deploy 任务。
- 后续文档：Step 06-06 运维 runbook 应引用 release/rollback 流程。
