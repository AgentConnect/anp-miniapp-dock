# Step 04-06：并发、取消、重试与幂等策略

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：04-06
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
| Next action | 等待 04-05 完成后，启动并发/取消/幂等策略 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：为多 session、多 API 并发、高风险交易串行化、cancellation token、timeout、retry policy 和 idempotency key 建立 Runtime 策略。
- 用户 / 系统可见行为：取消后不会继续执行高风险 action；并发不会串 session/token/storage；非幂等支付/下单默认不自动重试。
- 非目标：不实现分布式事务；不保证跨 Host 的全局锁。
- 完成标准：Runtime API 和 RequestBroker 有并发隔离、取消、超时、重试和幂等 tests。

## 3. 设计方法

- 设计边界：并发策略保护用户交易与 session 隔离；不能为了吞吐绕过 consent、audit 或 idempotency。
- 核心决策：同一 session 可并发普通 API；同一高风险 transaction 按 policy 串行；支付/下单等非幂等请求默认不自动 retry。
- 契约 / API / 数据流：Runtime call -> session manager -> cancellation token/timeout -> permission/consent -> broker/executor -> idempotency/audit。
- 兼容性：现有 coffee flow 不回归；request 401 auth retry 仍只用于安全认证握手。
- 风险控制：取消/timeout 后不再调用 callback/action/provider；dynamic timer/request 在 expire/session close 清理。

## 4. 实现方法

1. 阅读 Runtime API facade、RequestBroker、DID session manager、component dynamic cleanup 和 dock-core orchestrator。
2. 定义 session manager、per-session lock、高风险 transaction policy、cancellation token 和 timeout shape。
3. 定义 retry policy：auth handshake 可重试，非幂等 business API 默认不自动重试；idempotency key 由 order/payment API 或 Host provider 接收。
4. 实现 session close、component expire 和 cancellation 对 pending request/timer/action 的清理。
5. 增加 tests：multi-session isolation、parallel safe API、high-risk serial、cancel before provider、timeout、non-idempotent no retry、idempotency key propagation。
6. 更新 Phase 4 文档、Threat Model、Release Gates 和 runbook。
7. 回填本 Step 和主 Plan 执行台账；Phase 4 完成后触发阶段 Review。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-core` | session manager、cancellation、high-risk serial policy、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/anp-adapter` | RequestBroker retry/idempotency policy | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/component-runtime` | dynamic timer/request cleanup on expire/session close | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/consent-audit` | idempotency/audit summary | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/security/threat-model.md` | 同步 concurrency/retry 风险 | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | concurrency/idempotency gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-4-runtime-host-integration.md` | 同步并发策略 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/04-06-concurrency-cancellation-idempotency.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 02-05、Step 03-03、Step 03-05、Step 04-01、Step 04-05。
- 外部文档或决策：Runtime API facade、RequestBroker contract、Threat Model。
- 环境前提：Rust toolchain 1.88.0；无需真实 distributed lock。

## 7. 验收标准

- [ ] 多 session 并发不会串 token/storage/audit。
- [ ] 高风险交易按 policy 串行或明确拒绝并发。
- [ ] cancellation token 和 timeout 能阻止后续 provider/action/callback。
- [ ] 非幂等支付/下单默认不自动重试；auth handshake retry 仍安全。
- [ ] idempotency key 可传递到 order/payment/provider boundary，并进入脱敏 audit summary。
- [ ] session close / component expire 清理 dynamic request/timer。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Core concurrency tests | `cd anp/anp-miniapp-dock && cargo test -p dock-core concurrency` | session/serial/cancel tests 通过；若 filter 不匹配，记录实际命令 |
| Request retry tests | `cd anp/anp-miniapp-dock && cargo test -p anp-adapter retry` | retry/idempotency tests 通过 |
| Component cleanup tests | `cd anp/anp-miniapp-dock && cargo test -p component-runtime cleanup` | expire/detach cleanup tests 通过 |
| Workspace 回归 | `cd anp/anp-miniapp-dock && cargo test --workspace` | 通过；如耗时受限，记录 focused 替代和风险 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-core crates/anp-adapter crates/component-runtime crates/consent-audit docs/security docs/runbook docs/plan` | 无空白错误 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：取消后是否仍可能执行高风险操作；retry 是否会重复交易；session isolation 是否覆盖 token/storage/audit。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 concurrency/cancellation/idempotency、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase4: add runtime concurrency controls`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 04-06 小 Plan | 将并发、取消、重试与幂等策略拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：retry/并发策略错误会造成重复下单、重复支付或跨 session 数据污染。
- 回滚 / 回退：高风险能力不确定时默认串行和 no retry；记录 residual risk。
- 后续文档：Phase 6 runbook 必须覆盖 cancellation、timeout、idempotency failure 和 dynamic cleanup。
