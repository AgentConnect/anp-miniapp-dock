# Step 04-07：Persistent Audit Sink retention/export

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：04-07
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
| Next action | 等待 04-06 完成后，启动 persistent audit sink |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：为 consent/audit 建立持久化 sink、retention policy、redacted export 和 audit sink unavailable failure policy。
- 用户 / 系统可见行为：高风险 API、ConsentGate、RequestBroker 和 component action 的 audit evidence 可按策略保留、导出和脱敏；audit sink 失败时不会静默放行高风险动作。
- 非目标：不实现通用 SIEM；不导出真实隐私原文或 raw token。
- 完成标准：persistent audit 有 append/retention/export/unavailable tests，且 export 默认 redacted。

## 3. 设计方法

- 设计边界：audit 是安全证据，不是业务 payload 存储；只保存必要 metadata、consent proof reference、risk level、outcome 和 redacted summary。
- 核心决策：production sink 可为 append-only file、SQLite 或 Host-provided sink；in-memory 只能用于 dev/test。
- 契约 / API / 数据流：ConsentGate / RequestBroker / Runtime action -> AuditRecord -> persistent sink -> retention/export -> runbook evidence。
- 兼容性：保持现有 consent-audit redaction 规则；新增持久化不能改变模型可见输出。
- 风险控制：audit sink unavailable 对 L3/L4 动作默认 fail closed 或进入明确 degraded policy 并记录 release blocker。

## 4. 实现方法

1. 阅读 Step 03-05 Consent Adapter、Step 04-04 config boundary 和 `consent-audit` 当前审计模型。
2. 定义 audit sink trait、record schema、retention policy、redacted export format 和 unavailable policy。
3. 实现或规划 persistent sink wiring，保留 in-memory dev sink。
4. 增加 tests：append/read、retention delete、redacted export、sink unavailable fail closed、sensitive field redaction。
5. 更新 Threat Model、Release Gates、runbook 和 Phase 4 文档。
6. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/consent-audit` | persistent sink、retention、redacted export、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/dock-core` | audit sink provider wiring | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/anp-adapter` | request/audit outcome integration | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/security/threat-model.md` | 同步 audit retention/export 风险 | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | persistent audit gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-4-runtime-host-integration.md` | 同步 audit persistence 状态 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/04-07-persistent-audit-retention-export.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 03-05、Step 04-04。
- 外部文档或决策：Threat Model、ConsentProof、Release Gates、Runtime config。
- 环境前提：Rust toolchain 1.88.0；production sink 可先以 trait + local candidate 落地。

## 7. 验收标准

- [ ] audit sink 有 persistent backend boundary、in-memory dev profile 和 production profile gate。
- [ ] audit record schema 不包含 raw token、Authorization、signature、private key material、手机号、地址、文件内容或精确位置。
- [ ] retention policy 和 redacted export 有 tests。
- [ ] audit sink unavailable 对 L3/L4 动作 fail closed 或明确记录 degraded policy/release blocker。
- [ ] Threat Model、Release Gates 和 Phase 4 文档与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Audit tests | `cd anp/anp-miniapp-dock && cargo test -p consent-audit audit` | retention/export/redaction tests 通过 |
| Runtime audit regression | `cd anp/anp-miniapp-dock && cargo test -p dock-core audit` | audit wiring tests 通过；若 filter 不匹配，记录实际命令 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/consent-audit crates/dock-core crates/anp-adapter docs/security docs/runbook docs/plan` | 无空白错误 |
| 敏感信息扫描 | 手工或 `rg` 检查 audit export/test output | 不含 raw token、Authorization、signature、private key material 或真实隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：audit 是否只保存必要 metadata；export 是否默认 redacted；sink unavailable 是否不会静默放行高风险动作；retention 是否可审计。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 persistent audit sink、retention/export、direct tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase4: add persistent audit sink`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 04-07 小 Plan | 按 Review 发现将原 04-04 的 audit persistence 拆成 focused Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：audit 若记录 payload 或 secret，会把安全证据变成长期泄露面。
- 回滚 / 回退：production sink 不可用时对高风险动作 fail closed 或 release blocked；只保留 in-memory dev sink 用于测试。
- 后续文档：Step 06-01 结构化事件和 Step 06-06 运维 runbook 必须引用 audit sink unavailable 和 redacted export 行为。
