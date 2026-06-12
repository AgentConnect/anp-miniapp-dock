# Step 04-03：Skill Registry / Cache 与版本回滚

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：04-03
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
| Next action | 等待 04-02 完成后，启动 Skill registry/cache |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：把本地目录加载扩展为可对接 merchant manifest / ANP Agent registry 的 Skill 发现、下载、缓存、版本选择和回滚 contract。
- 用户 / 系统可见行为：相同 digest 可复用缓存，digest mismatch 拒绝加载，可回滚到上一个已验证 Skill version。
- 非目标：不实现完整远端 marketplace；不弱化 Step 03-06 的签名和 publisher DID gate。
- 完成标准：registry/cache 复用 package integrity contract，支持 latest/pinned/prerelease/rollback 策略和 cache eviction。

## 3. 设计方法

- 设计边界：registry/cache 是供应链 gate 后的受控分发层；未经验证的包进入 quarantine，不可执行。
- 核心决策：cache key 使用 publisher DID + skill id + version + digest；cache directory verify 后只读。
- 契约 / API / 数据流：merchant manifest/registry -> skill package URL/digest/signature -> download -> verify -> cache -> load_skill -> audit。
- 兼容性：本地 path skill 仍可 dev profile 加载；production profile 优先 registry/cache。
- 风险控制：下载必须 allowlist/HTTPS/DID policy；cache purge/rollback 不删除 audit 证据。

## 4. 实现方法

1. 阅读 Step 03-06 package integrity contract 和 Phase 4 registry/cache 计划。
2. 定义 Skill reference：local path、package URL、registry id、publisher DID、version、digest。
3. 实现 cache metadata、digest-keyed directory、read-only after verify、eviction 和 rollback record。
4. 对接 merchant manifest / ANP Agent registry 的最小 trait；无真实 registry 时用 local/mock conformance tests。
5. 增加 tests：same digest cache reuse、digest mismatch reject、unknown publisher quarantine、version pin、rollback、cache eviction、audit redaction。
6. 更新 Phase 4 文档、Threat Model、Release Gates 和 developer validate 计划。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/skill-loader` | registry ref、cache、version pin/rollback、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/anp-adapter` | merchant/agent discovery trait | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/dock-core` | load_skill 使用 registry/cache ref | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/dock-cli` | validate/load registry ref 支持 | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/security/threat-model.md` | 同步 registry/cache supply chain 风险 | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | cache/rollback gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-4-runtime-host-integration.md` | 同步 registry/cache contract | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/04-03-skill-registry-cache-versioning.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 03-06、Step 04-01。
- 外部文档或决策：Skill package integrity contract、Runtime API facade、Threat Model。
- 环境前提：Rust toolchain 1.88.0；真实远端 registry 可由 trait/mock 先行验证。

## 7. 验收标准

- [ ] Skill ref 支持 local、package URL 或 registry id，并保留 publisher DID、version、digest。
- [ ] Cache key 使用 publisher DID + skill id + version + digest；verified cache 只读。
- [ ] digest mismatch、signature mismatch、unknown publisher 进入 quarantine 或 fail closed。
- [ ] 支持 latest/pinned/prerelease/rollback 策略，并有 rollback tests。
- [ ] package source/version/digest 进入脱敏 audit summary。
- [ ] Phase 4 文档和 release gates 与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Loader/cache tests | `cd anp/anp-miniapp-dock && cargo test -p skill-loader cache` | cache/version/rollback tests 通过 |
| Registry tests | `cd anp/anp-miniapp-dock && cargo test -p skill-loader registry` | registry ref tests 通过；若 filter 不匹配，记录实际命令 |
| CLI 回归 | `cd anp/anp-miniapp-dock && cargo test -p dock-cli validate` | validate/load 报告不回归 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/skill-loader crates/anp-adapter crates/dock-core crates/dock-cli docs/security docs/runbook docs/plan` | 无空白错误 |
| 安全抽样 | 手工检查 cache metadata、audit、CLI JSON | 不含 token、private key material、真实 secret 或本地敏感绝对路径 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：cache 是否在 verify 后只读；rollback 是否安全；registry download 是否通过 allowlist/trust policy；audit 是否脱敏。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 Skill registry/cache/versioning、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase4: add skill registry cache`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 04-03 小 Plan | 将 Skill Registry / Cache 与版本回滚拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：cache 与 package integrity 分离会导致已篡改包被复用。
- 回滚 / 回退：cache 命中也必须重新验证 digest/signature metadata；无法验证时 purge/quarantine。
- 后续文档：Phase 5 import/validate 和 Phase 6 rollback runbook 必须复用本版本策略。
