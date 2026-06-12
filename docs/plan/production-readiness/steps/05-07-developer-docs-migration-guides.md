# Step 05-07：开发者文档与迁移指南

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：05-07
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
| Next action | 等待 05-06 完成后，启动开发者文档收敛 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：补齐开发者文档，包括从微信 MCP Skill 迁移、API/组件兼容、权限安全、Host adapter guide 和 CLI 工作流。
- 用户 / 系统可见行为：开发者无需阅读 Rust 源码，也能导入、验证、调试、修复兼容问题并理解 ANP DID 替代微信身份的方式。
- 非目标：不建立完整文档站；不承诺所有微信 API 兼容。
- 完成标准：文档与 CLI report 和兼容矩阵使用同一状态枚举，示例命令可复制，安全红线明确。

## 3. 设计方法

- 设计边界：开发者文档必须讲清 Agentic MiniApp Container 边界，不把产品写成完整微信小程序 runtime。
- 核心决策：以任务流组织文档：import -> inspect -> validate -> test-skill -> doctor -> deploy/Host integration。
- 契约 / API / 数据流：开发者 Skill -> CLI tools -> compatibility report -> migration fixes -> fixtures/tests -> Host adapter。
- 兼容性：API/组件兼容文档直接引用矩阵，避免复制后漂移。
- 风险控制：安全指南明确不要存 token、不要在 content 暴露隐私、如何声明权限、如何处理 fallback。

## 4. 实现方法

1. 创建 `docs/developer/` 文档入口和目录。
2. 编写 `import-wechat-mcp-skill.md`，说明迁移流程、ANP DID 替代微信身份、unsupported API 处理。
3. 编写 `wx-api-compatibility.md` 和 `component-compatibility.md`，引用矩阵并解释状态枚举。
4. 编写 `security-guidelines.md`，覆盖 token、隐私、permissions、ConsentGate、audit、fallback。
5. 编写 `host-adapter-guide.md`，引用 Runtime API、IPC/SDK、Host adapter contract 和 conformance。
6. 更新 README、local demo runbook 和 Phase 5 文档。
7. 回填本 Step 和主 Plan 执行台账；Phase 5 完成后触发阶段 Review。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/docs/developer/import-wechat-mcp-skill.md` | 新增迁移指南 | 计划新增 |
| `anp/anp-miniapp-dock/docs/developer/wx-api-compatibility.md` | 新增 API 兼容说明 | 计划新增 |
| `anp/anp-miniapp-dock/docs/developer/component-compatibility.md` | 新增组件兼容说明 | 计划新增 |
| `anp/anp-miniapp-dock/docs/developer/security-guidelines.md` | 新增安全开发指南 | 计划新增 |
| `anp/anp-miniapp-dock/docs/developer/host-adapter-guide.md` | 新增 Host adapter 开发指南 | 计划新增 |
| `anp/anp-miniapp-dock/docs/runbook/local-demo.md` | 增加多 fixture / CLI 调试方式 | 视实现结果更新 |
| `anp/anp-miniapp-dock/README.md` | 新增 developer docs 入口 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-5-developer-experience.md` | 同步文档完成状态 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/05-07-developer-docs-migration-guides.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 05-01、Step 05-02、Step 05-03、Step 05-04、Step 05-05、Step 05-06。
- 外部文档或决策：API/组件矩阵、Runtime API、Host adapter contract、Release Gates。
- 环境前提：文档为主；命令示例必须与实际 CLI 对齐。

## 7. 验收标准

- [ ] `docs/developer/` 至少包含 import、API compatibility、component compatibility、security guidelines、Host adapter guide。
- [ ] 文档使用与 CLI report/矩阵一致的状态枚举。
- [ ] CLI 示例命令可复制，或明确标注 planned/环境前提。
- [ ] 安全指南明确 token、Authorization、private key、手机号、地址、文件、位置不得进入 model-visible output。
- [ ] Host adapter guide 明确 action 必须回 Runtime，不得绕过 consent/audit。
- [ ] README 和 runbook 有开发者入口。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- docs/developer docs/runbook docs/plan README.md` | 无空白错误 |
| 链接检查 | 手工检查新增文档相对链接 | 链接目标存在 |
| 命令抽样 | 手工核对文档中的 CLI 命令与 `AGENTS.md` / CLI help 一致 | 命令可复制或标注 planned |
| 术语抽样 | `cd anp/anp-miniapp-dock && rg -n "supported|host-boundary|planned-p1|planned-p2|demo-only|unsupported-by-design" docs/developer docs/architecture` | 状态枚举一致 |
| 安全抽样 | `cd anp/anp-miniapp-dock && rg -n "token|Authorization|private key|phone|address|file|location|ConsentGate|audit" docs/developer` | 命中安全指南和 redaction 规则，未出现真实 secret |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：文档同步完成后、commit 前。
- Review 重点：开发者是否能按文档完成工作流；是否误导完整微信兼容；状态枚举是否漂移；安全红线是否清楚。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：验证、Review、文档同步完成后。
- Commit 范围：只包含 developer docs、README/runbook 入口和相关计划回填。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`docs: add developer migration guides`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 05-07 小 Plan | 将开发者文档与迁移指南拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：文档若复制矩阵内容，后续容易漂移。
- 回滚 / 回退：文档优先链接矩阵和 CLI report schema，只摘要解释状态。
- 后续文档：Phase 6 release notes 和 runbook 应引用开发者文档作为外部沟通基础。
