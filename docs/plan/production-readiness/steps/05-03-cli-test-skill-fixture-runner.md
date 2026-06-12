# Step 05-03：CLI test-skill 与 Fixture Runner

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：05-03
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
| Next action | 等待 05-02 完成后，启动 test-skill fixture runner |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：新增或增强 `dock-cli test-skill`，执行 fixture cases，覆盖 API call、component render、action dispatch、snapshot compare 和 audit summary。
- 用户 / 系统可见行为：开发者可以在本地/CI 跑 Skill 的兼容测试，看到 pass/fail、diff、fallback 和 audit 摘要。
- 非目标：不替代 cargo tests；不连接真实生产 Host provider。
- 完成标准：fixture runner 复用 Runtime API、Render IR snapshots 和 validate report，输出稳定 JSON。

## 3. 设计方法

- 设计边界：test-skill 使用 mock/dev-only provider 时必须显式标记；不得把测试通过等同生产 provider 可用。
- 核心决策：fixture case 格式包含 API input、expected AtomicApiResult、expected Render IR snapshot、expected actions、expected audit summary。
- 契约 / API / 数据流：Fixture cases -> Runtime API -> normalize result/snapshot/audit -> compare -> test report。
- 兼容性：继续支持 coffee E2E；新增 runner 可逐步承接 Phase 2 fixtures。
- 风险控制：snapshot normalization 移除随机 id、时间戳、token、signature、本地路径和隐私原文。

## 4. 实现方法

1. 阅读 Step 02-06 fixtures/snapshots 和 Runtime API facade。
2. 定义 fixture case schema 和 test report schema。
3. 实现 API call、render component、dispatch action、snapshot compare、audit summary compare。
4. 增加 diff 输出和 redaction/normalization。
5. 增加 tests：coffee fixture、snapshot match/mismatch、audit redaction、mock/dev-only marker、failure report JSON。
6. 更新 Phase 5 文档、Release Gates 和 README。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-cli` | `test-skill` command、fixture runner、report tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/dock-core` | Runtime API fixture execution support | 视当前结构修改 |
| `anp/anp-miniapp-dock/testdata/render-ir` | golden snapshots | 视当前结构修改 |
| `anp/anp-miniapp-dock/examples/fixtures` | fixture cases | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | fixture runner gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-5-developer-experience.md` | 同步 test-skill contract | 必须 |
| `anp/anp-miniapp-dock/README.md` | 视 CLI 使用说明更新 | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/05-03-cli-test-skill-fixture-runner.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 02-06、Step 04-01、Step 05-01。
- 外部文档或决策：Render IR snapshot rules、Runtime API facade、Release Gates。
- 环境前提：Rust toolchain 1.88.0；fixtures mock-only。

## 7. 验收标准

- [ ] `dock-cli test-skill` 可执行 fixture cases，并输出 JSON report。
- [ ] Runner 覆盖 API call、component render、action dispatch、snapshot compare、audit summary。
- [ ] Snapshot normalization 移除随机 id、时间戳和敏感字段。
- [ ] Mock/dev-only provider 在 report 中显式标识，不误标 production-ready。
- [ ] failure diff 可定位 API/result/render/action/audit 哪一层失败。
- [ ] Release Gates 和 Phase 5 文档与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| CLI fixture tests | `cd anp/anp-miniapp-dock && cargo test -p dock-cli fixture` | fixture runner tests 通过 |
| Coffee E2E | `cd anp/anp-miniapp-dock && cargo test -p dock-cli --test coffee_order_flow` | 通过 |
| Manual test-skill | `cd anp/anp-miniapp-dock && cargo run -p dock-cli -- test-skill examples/coffee-skill` | 输出 JSON report；若命令参数不同，记录实际命令 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-cli crates/dock-core examples testdata docs/runbook docs/plan README.md` | 无空白错误 |
| 脱敏抽样 | 手工检查 fixture report 和 snapshots | 不含 token、Authorization、signature、private key path 或隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：fixture schema 是否稳定；mock/dev-only 是否清晰；diff 是否有用且不泄露敏感数据；runner 是否复用 Runtime API。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 test-skill fixture runner、直接 tests/fixtures 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase5: add skill fixture runner`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 05-03 小 Plan | 将 CLI test-skill 与 Fixture Runner 拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：fixture runner 如果绕过 Runtime API，会与生产路径漂移。
- 回滚 / 回退：runner 必须复用 Runtime facade；无法覆盖的 Host provider 标为 mock/dev-only。
- 后续文档：Release Gates 和开发者指南应把 test-skill 作为上线前标准步骤。
