# Step 04-01：Runtime API Facade 与版本化

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：04-01
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
| Next action | 等待 Phase 3 完成后，启动 Runtime API Facade 稳定化 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：定义并实现稳定的 public Runtime API facade，覆盖 validate/load/call/render/action/expire/audit/close session。
- 用户 / 系统可见行为：CLI、Host adapter 和未来 IPC 都调用同一 runtime contract，不再维护第二套流程。
- 非目标：不在本 Step 实现 HTTP/gRPC sidecar；不实现真实 Host UI。
- 完成标准：Runtime API 输入输出 JSON 可序列化、错误码稳定、API version 可协商，CLI 主流程可以逐步迁移到 facade。

## 3. 设计方法

- 设计边界：Runtime facade 是产品化宿主集成边界；内部 crate 可以重构，但对外 contract 必须版本化。
- 核心决策：先稳定 Rust library API，再让 CLI 和 IPC 复用；所有返回不得泄露 token、private key、Host private meta。
- 契约 / API / 数据流：Host/CLI -> RuntimeService -> skill-loader / dock-core / component-runtime / consent-audit -> RuntimeResult / RuntimeError。
- 兼容性：保持 coffee flow、`dock-cli validate`、`run-demo` 行为；新增 facade 不改变现有公开 JSON，除非记录 migration。
- 风险控制：错误码和 `_meta` 隔离必须复用现有 redaction 和 model-visible 边界。

## 4. 实现方法

1. 阅读 `dock-core` orchestrator、`dock-cli` commands、skill-loader、component-runtime 和 current CLI flow。
2. 定义 `RuntimeService` 或等价 facade：`validate_skill`、`load_skill`、`call_api`、`render_component`、`dispatch_component_action`、`expire_cards`、`get_audit_records`、`close_session`。
3. 定义 version、request/response DTO、stable error codes 和 redaction boundary。
4. 将至少一个 CLI 主流程迁移到 facade，避免 duplicate orchestration。
5. 增加 runtime API tests：load/call/render/action/expire/audit/close、error serialization、redaction。
6. 更新 Phase 4 文档、README/runbook 或 developer notes。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-core` | Runtime API facade、DTO、error/version tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/dock-cli` | 迁移 CLI 主流程调用 facade | 代码实现 |
| `anp/anp-miniapp-dock/crates/component-runtime` | render/action facade 接入 | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/skill-loader` | load/validate facade 接入 | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-4-runtime-host-integration.md` | 同步 Runtime API contract | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/local-demo.md` | 若 CLI 命令行为变化，同步说明 | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/04-01-runtime-api-facade-versioning.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 02-06、Step 03-06。
- 外部文档或决策：Phase 4 Runtime Host Integration、Release Gates、Threat Model。
- 环境前提：Rust toolchain 1.88.0；无需外部 Host。

## 7. 验收标准

- [ ] Runtime facade 覆盖 validate/load/call/render/action/expire/audit/close session 的最小 stable API。
- [ ] Runtime API 有 version 字段或版本协商策略。
- [ ] Error code stable、JSON 可序列化、敏感字段 redacted。
- [ ] CLI 至少一个关键路径调用 facade，且 coffee E2E 不回归。
- [ ] Phase 4 文档记录 API contract 和 migration 影响。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Runtime tests | `cd anp/anp-miniapp-dock && cargo test -p dock-core runtime` | facade/API DTO/error tests 通过；若 filter 不匹配，记录实际命令 |
| CLI 回归 | `cd anp/anp-miniapp-dock && cargo test -p dock-cli --test coffee_order_flow` | coffee CLI E2E 通过 |
| Workspace 回归 | `cd anp/anp-miniapp-dock && cargo test --workspace` | 通过；如耗时受限，记录 focused 替代和风险 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-core crates/dock-cli crates/component-runtime crates/skill-loader docs/runbook docs/plan` | 无空白错误 |
| 脱敏抽样 | 手工检查 Runtime API error/result JSON | 不含 token、Authorization、signature、private key path 或隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：facade 是否成为唯一 orchestration contract；version/error 是否稳定；CLI 是否避免第二套流程；redaction 是否集中。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 Runtime API facade、直接 tests、CLI 接入和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase4: add runtime api facade`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 04-01 小 Plan | 将 Runtime API Facade 与版本化拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：facade 如果只包一层 CLI 逻辑，会变成第三套流程而不是收敛。
- 回滚 / 回退：先让 CLI 逐步迁移，保持旧命令行为；任何 contract break 需要 migration note。
- 后续文档：Step 04-02 IPC/SDK 必须复用本 facade，不直接调内部 crate。
