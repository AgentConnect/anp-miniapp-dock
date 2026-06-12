# Step 04-04：持久化、配置与 Secret Store 边界

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：04-04
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
| Next action | 等待 04-03 完成后，启动持久化与配置边界 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：为 session/token cache、scoped storage、audit、skill cache 和 runtime config 建立生产候选持久化与 secret store 边界。
- 用户 / 系统可见行为：容器重启后按策略恢复非过期 token/storage/audit，删除用户/Skill 数据可按 scope 清理。
- 非目标：不绑定单一云厂商 secret store；不把 private key 或 token 写入普通 config。
- 完成标准：持久化 backend 有 encryption/secure-store 边界、quota、migration、retention、delete scope 和 failure policy。

## 3. 设计方法

- 设计边界：持久化只保存必要 runtime state；secrets 通过 env/secret store/Host credential provider 注入。
- 核心决策：token cache 使用 secure store 或加密 SQLite；scoped storage 使用 DID/merchant/Skill scope；audit append-only 或 SQLite + retention；config 非 secret。
- 契约 / API / 数据流：Runtime config -> persistence providers -> token/storage/audit/cache operations -> scope cleanup/export。
- 兼容性：保留 in-memory backend 作为 tests/dev；production profile 必须显式配置持久化或标记 release blocker。
- 风险控制：storage/audit path 不泄露 secret；持久化失败 fail closed 或 degradation 可审计。

## 4. 实现方法

1. 阅读 Step 01-06 storage、Step 03-04 token lifecycle、Step 03-05 audit sink、Step 04-03 skill cache。
2. 定义 runtime config model：identity、trusted DID、allowlist、token issuer、storage path、audit path、log level、mock providers。
3. 为 token/storage/audit/skill cache 定义 backend trait、production candidate 和 in-memory dev profile。
4. 实现 scope cleanup：user DID、merchant DID、Skill id、session。
5. 增加 tests：restart restore、expired token not restored、quota、scope cleanup、audit retention、config redaction、secret path redaction。
6. 更新 runbook、Threat Model、Release Gates 和 Phase 4 文档。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-core` | runtime config、persistence provider wiring、scope cleanup | 代码实现 |
| `anp/anp-miniapp-dock/crates/anp-adapter` | token cache persistence / secure boundary | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/wx-compat` | scoped storage backend/quota | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/consent-audit` | audit sink retention/export integration | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/skill-loader` | cache persistence cleanup | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/runbook/local-demo.md` | 同步 config/storage/audit path | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | production persistence gate | 必须 |
| `anp/anp-miniapp-dock/docs/security/threat-model.md` | 同步 secret/persistence 风险 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-4-runtime-host-integration.md` | 同步持久化策略 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/04-04-persistence-config-secret-store.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 01-06、Step 03-04、Step 03-05、Step 04-03。
- 外部文档或决策：Threat Model、Release Gates、Runtime API facade。
- 环境前提：Rust toolchain 1.88.0；生产 secret store 可用 trait + Host boundary 先行。

## 7. 验收标准

- [ ] token/storage/audit/skill cache 都有 backend boundary 和 production/dev profile。
- [ ] 非过期 token 和 storage 可按策略重启恢复；expired/revoked token 不恢复。
- [ ] scoped storage 有 quota 和 scope cleanup tests。
- [ ] audit retention/export 与 persistent sink 策略一致。
- [ ] config 文件不包含 secret；secret path/material 在 logs/CLI/audit 中 redacted。
- [ ] Release Gates 标出 in-memory backend 不能 production-ready。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Persistence tests | `cd anp/anp-miniapp-dock && cargo test --workspace persistence` | persistence 相关 tests 通过；若 filter 不匹配，记录实际命令 |
| Storage tests | `cd anp/anp-miniapp-dock && cargo test -p wx-compat storage` | quota/scope tests 通过 |
| Audit tests | `cd anp/anp-miniapp-dock && cargo test -p consent-audit audit` | retention/export tests 通过 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-core crates/anp-adapter crates/wx-compat crates/consent-audit crates/skill-loader docs/runbook docs/security docs/plan` | 无空白错误 |
| 敏感信息扫描 | 手工或 `rg` 检查 config/log/test output | 不含 private key material、raw token、Authorization、signature 或真实 secret |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：secret 是否不进 config；restart restore 是否安全；scope cleanup 是否精确；in-memory backend 是否被 release gate 阻断。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 persistence/config/secret boundary、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase4: add persistence config boundaries`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 04-04 小 Plan | 将持久化、配置与 Secret Store 边界拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：持久化引入后，错误的 scope cleanup 或 secret config 会造成长期数据泄露。
- 回滚 / 回退：production persistence 不可用时 fail closed 或明确 release blocker；dev in-memory 只能用于测试。
- 后续文档：Phase 6 运维 runbook 必须覆盖 storage quota、audit sink unavailable、privacy deletion。
