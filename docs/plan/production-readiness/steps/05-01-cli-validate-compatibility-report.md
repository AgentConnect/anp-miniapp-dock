# Step 05-01：CLI validate 兼容报告增强

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：05-01
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
| Next action | 等待 Phase 4 Runtime/Host 基线完成后，启动 CLI validate 报告增强 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：增强 `dock-cli validate`，输出稳定 JSON compatibility report，覆盖 API、组件、权限、风险、fallback、release blockers 和修复建议。
- 用户 / 系统可见行为：开发者和 CI 可以用一个命令判断 Skill 是否可上线、会降级还是被阻断。
- 非目标：不在本 Step 实现 inspect/test/import/doctor；不改变 Runtime 的安全边界。
- 完成标准：validate JSON schema 与 API/组件矩阵状态枚举一致，demo-only/mock/supply-chain/persistence blocker 清晰。

## 3. 设计方法

- 设计边界：validate 是报告工具，不执行高风险 provider，不让 unsupported API 静默通过。
- 核心决策：输出全 JSON，包含 `status`、`skillId`、`compatibilityLevel`、`apis`、`components`、`permissions`、`risks`、`fallbacks`、`releaseBlockers`。
- 契约 / API / 数据流：Skill package -> schema/loader/runtime compatibility scan -> report JSON -> CI/开发者。
- 兼容性：保持现有 validate 命令可用；新增字段向后兼容或记录 schema version。
- 风险控制：报告中所有路径、凭据、DID、token、private key 和隐私值必须 redacted。

## 4. 实现方法

1. 阅读现有 `dock-cli validate`、mcp-schema compatibility report、API/组件矩阵。
2. 定义 validate report schema/version 和状态枚举，与文档矩阵保持一致。
3. 增加 API/组件 unsupported、host-boundary、planned、demo-only、security/release blocker 的修复建议。
4. 集成 package integrity、permission policy、Host provider、persistence/backend、Render IR snapshot gate 的报告字段。
5. 增加 tests：coffee report、unsupported API、dynamic warning、demo-only release blocker、redaction、JSON schema stability。
6. 更新 README、developer docs 计划、Release Gates 和 Phase 5 文档。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-cli` | validate report schema、warnings、release blockers、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/mcp-schema` | compatibility report DTO/status enum | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/skill-loader` | package integrity/loader evidence 接入 | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | validate report gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-5-developer-experience.md` | 同步 CLI validate contract | 必须 |
| `anp/anp-miniapp-dock/README.md` | 视 CLI 使用说明更新 | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/05-01-cli-validate-compatibility-report.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 01-05、Step 02-06、Step 03-06、Step 04-01。
- 外部文档或决策：API/组件矩阵、Release Gates、Runtime API facade。
- 环境前提：Rust toolchain 1.88.0；无需真实 Host provider。

## 7. 验收标准

- [ ] `dock-cli validate` 输出稳定 JSON schema 和 schema/version 字段。
- [ ] Report 包含 APIs、components、permissions、risks、fallbacks、releaseBlockers 和修复建议。
- [ ] 状态枚举与 API/组件矩阵一致，不混用未知状态。
- [ ] demo-only/mock/in-memory/unsigned package 等 production blocker 清晰标识。
- [ ] JSON 输出 redacted，不含 token、Authorization、signature、private key path 或隐私原文。
- [ ] Release Gates 和 Phase 5 文档与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| CLI validate tests | `cd anp/anp-miniapp-dock && cargo test -p dock-cli validate` | report schema/tests 通过 |
| Schema tests | `cd anp/anp-miniapp-dock && cargo test -p mcp-schema compatibility` | status/report DTO tests 通过；若 filter 不匹配，记录实际命令 |
| Manual validate | `cd anp/anp-miniapp-dock && cargo run -p dock-cli -- validate examples/coffee-skill` | 输出 JSON，仍正确标识 demo-only |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-cli crates/mcp-schema crates/skill-loader docs/runbook docs/plan README.md` | 无空白错误 |
| 脱敏抽样 | 手工检查 validate JSON | 不含 token、Authorization、signature、private key path 或隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：report 是否可被 CI 消费；状态枚举是否准确；release blockers 是否不漏安全/供应链/Host boundary；输出是否 redacted。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 validate report、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase5: enhance validate compatibility report`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 05-01 小 Plan | 将 CLI validate 兼容报告增强拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：report 与矩阵枚举漂移会误导开发者。
- 回滚 / 回退：引入 schema version；breaking change 必须 migration note。
- 后续文档：Step 05-07 开发者文档必须引用本 report schema。
