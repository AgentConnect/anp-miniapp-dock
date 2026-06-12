# Step 06-06：运维 Runbook 与隐私删除流程

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：06-06
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
| Next action | 等待 06-05 完成后，启动最终运维 runbook |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：补齐线上运维 runbook，覆盖 DID 验签失败、token scope mismatch、allowlist deny、render failed、sandbox timeout、storage quota、audit sink unavailable、Host provider unavailable、merchant Agent unavailable、signature mismatch、rollback/cache purge 和 privacy deletion。
- 用户 / 系统可见行为：运维人员可以判断问题属于 Skill、商家 Agent、Host provider、身份/token、storage/audit 还是容器本身。
- 非目标：不替代具体部署平台文档；不提供真实用户数据样例。
- 完成标准：每类故障有症状、检查命令、观测信号、处理步骤、升级路径、回滚/数据删除策略。

## 3. 设计方法

- 设计边界：runbook 面向生产操作，必须默认保护隐私和 audit evidence。
- 核心决策：按故障域组织：identity/token、network/allowlist、component/render、sandbox/resource、storage/audit、Host provider、merchant Agent、package integrity、release/rollback、privacy deletion。
- 契约 / API / 数据流：alert/metric/event -> diagnosis -> safe command -> remediation -> audit/evidence -> closure。
- 兼容性：引用 doctor、release gates、metrics/tracing、Runtime API 和 Host adapter contract。
- 风险控制：所有命令示例使用 mock paths 和 redacted values；privacy deletion 必须按 scope 精确删除并保留必要审计。

## 4. 实现方法

1. 阅读 Phase 6 runbook 计划、Release Gates、doctor、metrics/tracing、runtime config、scoped storage cleanup、audit retention/export、Skill cache cleanup 和 rollback runbook。
2. 新增或更新 runbook：operations、troubleshooting、privacy deletion、cache purge。
3. 为每类故障写明：症状、相关 event/metric、检查命令、常见原因、处理步骤、升级路径、回滚条件。
4. 定义 privacy deletion：user DID、merchant DID、Skill id、session、storage、audit retention、cache。
5. 增加 dry-run checklist，确保 runbook 中命令可执行或明确环境前提。
6. 更新 Phase 6 文档、README/runbook index 和主 Plan。
7. 回填本 Step 和主 Plan 执行台账；全部 Step 完成后触发最终全局 Review。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/docs/runbook/operations.md` | 新增/更新线上运维 runbook | 计划新增 |
| `anp/anp-miniapp-dock/docs/runbook/troubleshooting.md` | 新增/更新故障处理索引 | 计划新增 |
| `anp/anp-miniapp-dock/docs/runbook/privacy-deletion.md` | 新增隐私删除流程 | 计划新增 |
| `anp/anp-miniapp-dock/docs/runbook/release-process.md` | 引用 release/rollback 流程 | 视 Step 06-05 输出更新 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | 链接 operations/privacy deletion | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-6-observability-release.md` | 同步 runbook 完成状态 | 必须 |
| `anp/anp-miniapp-dock/README.md` | runbook 入口 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账和最终 Review | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/06-06-operations-runbook-privacy-deletion.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 04-04、Step 04-06、Step 04-07、Step 04-08、Step 05-05、Step 06-01、Step 06-02、Step 06-04、Step 06-05。
- 外部文档或决策：Release Gates、doctor、metrics/tracing、rollback strategy、Threat Model。
- 环境前提：文档为主；真实 deploy 平台命令可标注为 Host-specific。

## 7. 验收标准

- [ ] Runbook 覆盖 DID 验签失败、token scope mismatch、allowlist deny、component render failed、sandbox timeout、storage quota exceeded、audit sink unavailable、Host provider unavailable、merchant Agent unavailable、Skill package signature mismatch。
- [ ] 每类故障包含症状、event/metric、检查命令、处理步骤、升级路径和回滚/关闭条件。
- [ ] Privacy deletion 流程按 user/merchant/Skill/session scope 定义 storage、audit、cache 清理策略。
- [ ] 命令示例使用 mock/redacted value，不包含真实 secret 或隐私数据。
- [ ] README/runbook index 和 Phase 6 文档与 runbook 状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入最终全局 Review 之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- docs/runbook docs/plan README.md` | 无空白错误 |
| 链接检查 | 手工检查 runbook/README/Phase 6 links | 链接目标存在 |
| 命令 dry-run | 手工检查 runbook 中命令是否为真实命令或标注 Host-specific/planned | 无误导性命令 |
| 安全抽样 | `cd anp/anp-miniapp-dock && rg -n "token|Authorization|private key|secret|phone|address|file|location" docs/runbook docs/plan/production-readiness/phase-6-observability-release.md README.md` | 命中 redaction/安全说明，不含真实 secret 或隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：runbook 文档同步完成后、commit 前。
- Review 重点：故障处理是否可执行；隐私删除 scope 是否精确；是否保留必要 audit evidence；是否泄露 secret。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：验证、Review、文档同步完成后。
- Commit 范围：只包含 operations/privacy deletion runbook、README/runbook index 和计划回填。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`docs: add operations privacy runbook`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 06-06 小 Plan | 将运维 Runbook 与隐私删除流程拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |
| 2026-06-12 | 更新持久化/清理依赖 | 按 Review 发现，原 04-04 已拆分，隐私删除 runbook 需依赖 storage、audit、cache cleanup 切片 | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：runbook 若没有实际命令和观测信号，线上故障时无法执行。
- 回滚 / 回退：真实平台特有命令标注 Host-specific；核心诊断保留 Runtime/CLI/metrics 通用路径。
- 后续文档：本 Step 完成后应执行全计划最终全局 Review，确认 Phase 0-6 的里程碑、台账、Review 和验收标准完整。
