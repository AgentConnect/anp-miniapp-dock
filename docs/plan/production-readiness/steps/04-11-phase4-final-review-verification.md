# Step 04-11：Phase 4 最终 Review 与整体验证

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：04-11
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
| Next action | 等待 Step 04-10 完成后，执行 Phase 4 最终 Review；完成后停止本 Goal，不进入 Step 05-01 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：把 Step 04-01 至 Step 04-10 的生产运行时与 Host 接入最终 Review 和整体验证变成可追踪、可恢复、可标记 `done` 的执行 gate。
- 用户 / 系统可见行为：Codex Goal 不会在 Phase 4 完成后直接进入 Phase 5；必须先完成阶段级 Review、验证、证据记录和必要修复。
- 非目标：不新增 Phase 5 CLI / developer experience 能力；不扩大 `05-01` 范围。
- 完成标准：主 Plan 最终全局 Review 记录已追加 Phase 4 证据，执行台账中 04-01 至 04-11 均有 Review/验证/commit 证据，工作区无未提交完成工作。

## 3. 设计方法

- 设计边界：本 Step 是 Phase 4 integration review gate，只有在 04-01 至 04-10 全部 `done` 后启动。
- 核心决策：按主 Plan 的最终全局 Review 与整体验证章节执行；如果 Review 修复需要改文件，本 Step 创建独立 final review commit。
- 契约 / API / 数据流：Step evidence -> ledger audit -> git history audit -> runtime/Host contract audit -> verification suite -> Review findings/fixes -> final review record。
- 兼容性：确认 Phase 4 runtime facade、IPC/SDK、registry/cache、config/secret、token/storage/audit persistence、Host adapter、action protocol、concurrency/cancellation/retry/idempotency 没有破坏 coffee demo、wx bridge contract、Render IR、DID/request、ConsentGate、audit、redaction、sandbox 和 supply-chain gate。
- 风险控制：若发现安全、隐私、公开契约、release gate、持久化恢复或文档漂移阻塞问题，不得进入 05-01；必须修复或记录 blocker。

## 4. 实现方法

1. 确认 Step 04-01 至 04-10 在主 Plan 执行台账和各 Step 文档中均为 `done`，且有 commit hash、Review 证据和验证证据。
2. 核对这些 commit 能在 git history 中解析，并且没有未提交完成工作。
3. 执行 Phase 4 全局 Review：Runtime public API/SDK、IPC/headless 边界、Skill registry/cache、runtime config/secret store、token/storage/audit persistence、Skill cache cleanup、Host adapter contract/action protocol、并发/取消/重试/幂等、redaction、release blockers 和文档漂移。
4. 运行整体验证基线；若命令不能运行，记录原因、影响、替代检查和剩余风险。
5. 修复 Review 发现的必要问题；若修改文件，按本 Step Review/验证/commit gate 创建 focused commit。
6. 在主 Plan `2.3.8 最终全局 Review 与整体验证` 追加 Phase 4 执行记录。
7. 回填本 Step 和主 Plan 执行台账；本 Goal 在本 Step `done` 后停止，不进入 Step 05-01。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 追加 Phase 4 最终 Review 记录，回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/04-11-phase4-final-review-verification.md` | 回填状态、证据、Review、commit | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-4-runtime-host-integration.md` | 审计 Phase 4 contract 状态 | 视发现更新 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | 审计 release gates 是否覆盖 Phase 4 新能力 | 视发现更新 |
| `anp/anp-miniapp-dock/docs/security/threat-model.md` | 审计 Phase 4 持久化/Host 边界风险 | 视发现更新 |
| `anp/anp-miniapp-dock/docs/architecture` | 审计 runtime/Host/API/component contract 是否一致 | 视发现更新 |

## 6. 依赖

- 前置步骤：Step 04-01、Step 04-02、Step 04-03、Step 04-04、Step 04-05、Step 04-06、Step 04-07、Step 04-08、Step 04-09、Step 04-10。
- 外部文档或决策：主 Plan `2.3.8 最终全局 Review 与整体验证`、Phase 4 章节、Release Gates、Threat Model、API/组件兼容矩阵。
- 环境前提：Rust toolchain 1.88.0；若完整 workspace 验证耗时或环境缺失，必须记录 focused 替代和残余风险。

## 7. 验收标准

- [ ] 主 Plan 执行台账中 Step 04-01 至 04-10 全部为 `done`，且 commit hash、Review 证据、验证证据完整。
- [ ] git history 能解析 Step 04-01 至 04-10 的 commit hash。
- [ ] 执行或明确记录主 Plan 整体验证基线：metadata、fmt、clippy、workspace tests、coffee E2E、docs diff check。
- [ ] Review 覆盖 Runtime public API/SDK、IPC/headless、Skill registry/cache、config/secret、token/storage/audit persistence、Host adapter/action、concurrency/cancellation/retry/idempotency、redaction、安全边界和文档漂移。
- [ ] 必要 Review 发现已修复；无法修复的问题已记录为 blocker 或剩余风险。
- [ ] 主 Plan `2.3.8` 已追加 Phase 4 最终 Review 记录。
- [ ] 本步骤在进入 Step 05-01 之前已经创建 focused commit，并回填主 Plan 执行台账；本 Goal 到此结束。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 工作区状态 | `cd anp/anp-miniapp-dock && git status --short --branch` | 无未提交完成工作；若有用户改动，记录并保护 |
| Metadata | `cd anp/anp-miniapp-dock && cargo metadata --format-version 1 --no-deps` | 通过 |
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Clippy | `cd anp/anp-miniapp-dock && cargo clippy --workspace --all-targets -- -D warnings` | 通过 |
| Workspace tests | `cd anp/anp-miniapp-dock && cargo test --workspace` | 通过 |
| Coffee E2E | `cd anp/anp-miniapp-dock && cargo test -p dock-cli --test coffee_order_flow` | 通过 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- docs/plan docs/architecture docs/runbook docs/security README.md AGENTS.md` | 无空白错误 |
| 敏感信息抽样 | 手工或 `rg` 检查 Phase 4 相关输出、fixtures 和 docs | 不含 raw token、Authorization、signature、private key material、手机号、地址、文件内容、本机绝对路径或真实隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：整体验证完成后、commit 前。
- Review 重点：Step 证据是否完整；Phase 4 runtime/Host contract 是否稳定；持久化恢复是否符合 scope 和 secret 边界；是否仍有未记录的 production release blocker。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：最终 Review、整体验证、必要修复和主 Plan 记录完成后。
- Commit 范围：只包含 Phase 4 final review 记录、必要文档修复和直接关联证据更新。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`docs: record phase4 final review`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-13 | 创建 Step 04-11 小 Plan | 将 Phase 4 最终 Review 与整体验证变成可追踪 gate，避免 Codex Goal 直接进入 Phase 5 | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：如果只在 04-10 的下一步文字中提到 Phase 4 Review，长跑 Goal 恢复时可能跳过 Phase 5 前置集成审计。
- 回滚 / 回退：若本 Step 发现阻塞问题，保持 04-11 为 `blocked`，不得启动 05-01。
- 后续文档：Step 05-01 必须以本 Step 的最终 Review 记录作为 Phase 5 启动前证据。
