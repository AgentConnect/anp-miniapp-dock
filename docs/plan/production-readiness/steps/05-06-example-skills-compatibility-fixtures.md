# Step 05-06：示例 Skill 与兼容测试集

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：05-06
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
| Next action | 等待 05-05 完成后，启动示例 Skill 体系 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：在 coffee 之外补齐 address、media、dynamic-status、location 等示例 Skill / fixture，并与 validate/test-skill/snapshots 绑定。
- 用户 / 系统可见行为：开发者可以复制示例学习表单、地址/手机号 consent、media/file handle、dynamic component、location map preview。
- 非目标：不使用真实手机号、地址、文件、位置或支付；示例必须 mock-only。
- 完成标准：至少 3 个 coffee 之外示例可跑，每个有 README、run command、expected JSON、Render IR snapshot 和风险说明。

## 3. 设计方法

- 设计边界：示例是开发者教育和兼容回归，不是生产商家数据。
- 核心决策：复用 Step 02-06 fixtures；若 fixtures 已存在，本 Step 强化 README、CLI run commands、expected output 和开发者可读性。
- 契约 / API / 数据流：example Skill -> validate -> test-skill -> snapshot/audit -> README evidence。
- 兼容性：coffee 继续作为交易基线；新增示例覆盖 Phase 1/2/3/4 能力。
- 风险控制：所有示例 DID、手机号、地址、文件、位置、token 都是 mock/redacted。

## 4. 实现方法

1. 阅读 Step 02-06 fixture 输出、Phase 5 示例计划和 developer docs 计划。
2. 新增或整理 `examples/address-skill`、`examples/media-skill`、`examples/dynamic-status-skill`、`examples/location-skill`。
3. 每个示例补 `SKILL.md`、`mcp.json`、API JS、components、README、expected JSON、Render IR snapshot。
4. 将示例接入 `dock-cli validate` 和 `test-skill` regression。
5. 增加 tests：每个示例 validate/test-skill 通过或输出 expected planned gap。
6. 更新 README、Phase 5 文档、Release Gates 和兼容矩阵证据。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/examples/address-skill` | 新增或整理 address/phone/form 示例 | 计划新增 |
| `anp/anp-miniapp-dock/examples/media-skill` | 新增或整理 image/file/media 示例 | 计划新增 |
| `anp/anp-miniapp-dock/examples/dynamic-status-skill` | 新增 dynamic component 示例 | 计划新增 |
| `anp/anp-miniapp-dock/examples/location-skill` | 新增 location/map-preview 示例 | 计划新增 |
| `anp/anp-miniapp-dock/testdata/render-ir` | 示例 snapshots | 视 Step 02-06 结构修改 |
| `anp/anp-miniapp-dock/crates/dock-cli/tests` | 示例 validate/test-skill regression | 必须 |
| `anp/anp-miniapp-dock/docs/architecture` | 同步示例证据 | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | 示例 gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-5-developer-experience.md` | 同步示例列表 | 必须 |
| `anp/anp-miniapp-dock/README.md` | 示例入口 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/05-06-example-skills-compatibility-fixtures.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 02-06、Step 05-01、Step 05-03。
- 外部文档或决策：fixture/snapshot rules、API/组件矩阵、Threat Model。
- 环境前提：Rust toolchain 1.88.0；示例必须 mock-only。

## 7. 验收标准

- [ ] coffee 之外至少 3 个示例 Skill 可 validate/test-skill。
- [ ] 每个示例有 README、run command、expected JSON、Render IR snapshot、风险说明。
- [ ] 示例覆盖表单/地址/手机号、media/file、dynamic component、location/map preview 中至少 3 类。
- [ ] 示例不包含真实手机号、地址、文件内容、位置、token、private key material。
- [ ] Release Gates 和 README 将示例纳入开发者验证路径。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Example tests | `cd anp/anp-miniapp-dock && cargo test -p dock-cli example` | 示例 validate/test-skill tests 通过；若 filter 不匹配，记录实际命令 |
| Manual validate | `cd anp/anp-miniapp-dock && cargo run -p dock-cli -- validate examples/address-skill` | 示例输出 expected report；其它示例同理或记录 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- examples testdata crates/dock-cli docs/architecture docs/runbook docs/plan README.md` | 无空白错误 |
| 敏感信息扫描 | `cd anp/anp-miniapp-dock && rg -n "token|Authorization|signature|private key|phone|address|latitude|longitude" examples testdata docs/plan docs/architecture docs/runbook README.md` | 只命中 mock/dev-only 示例、redaction 规则或安全说明 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：示例、tests、文档同步完成后、commit 前。
- Review 重点：示例是否真正可跑；是否覆盖核心能力；mock/dev-only 是否清晰；是否含敏感数据。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含示例 Skill、fixtures/snapshots、direct tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase5: add compatibility example skills`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 05-06 小 Plan | 将示例 Skill 与兼容测试集拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：示例如果不可跑，会误导开发者和 release gates。
- 回滚 / 回退：示例必须由 CLI tests 覆盖；不能跑的示例标为 planned，不进入 release gate。
- 后续文档：Step 05-07 迁移指南应引用这些示例。
