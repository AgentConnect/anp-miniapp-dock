# Step 05-05：CLI doctor 环境诊断

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：05-05
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
| Next action | 等待 05-04 完成后，启动 CLI doctor |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：新增或增强 `dock-cli doctor`，检查 Rust toolchain、DID identity、private key permissions、trusted resolver、allowlist、storage/audit path、Host providers、sandbox gates 和 remote server health。
- 用户 / 系统可见行为：开发者可以快速定位本地环境、凭据、Host provider 或远端服务配置问题。
- 非目标：不读取或输出私钥内容；不自动修复生产 secret。
- 完成标准：doctor 输出 JSON + human summary，区分 pass/warn/fail/skip，并提供修复建议。

## 3. 设计方法

- 设计边界：doctor 是诊断工具，不应执行高风险业务 API 或泄露 secret。
- 核心决策：所有检查都要有 severity、evidence、suggestion、redaction；无法检查的项目标记 skip 而不是 pass。
- 契约 / API / 数据流：Local config/env -> checks -> redacted diagnostic report -> optional CI failure code。
- 兼容性：复用 release gates 和 runbook 的命令/术语。
- 风险控制：private key path 可 redacted，private key material 永不读取输出；Authorization/token 不输出。

## 4. 实现方法

1. 阅读 Release Gates、local demo runbook、Runtime config/persistence plan。
2. 定义 doctor check registry：toolchain、workspace、DID document、private key permissions、resolver、allowlist、storage/audit path、Host providers、sandbox tests、server health。
3. 实现 redacted diagnostic report 和 exit code 策略。
4. 增加 tests：missing toolchain/config、bad key permissions、missing allowlist、unavailable provider、skip reason、redaction。
5. 更新 README、runbook 和 Phase 5 文档。
6. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-cli` | `doctor` command、check registry、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/anp-adapter` | DID resolver/credential diagnostic helper | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/dock-core` | runtime config diagnostic | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/runbook/local-demo.md` | 同步 doctor 用法 | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | doctor 作为预检 gate | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-5-developer-experience.md` | 同步 doctor contract | 必须 |
| `anp/anp-miniapp-dock/README.md` | 视 CLI 使用说明更新 | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/05-05-cli-doctor-environment.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 03-04、Step 04-04、Step 05-01。
- 外部文档或决策：Release Gates、local demo runbook、Runtime config。
- 环境前提：Rust toolchain 1.88.0；外部 server health checks 可 skip 并记录原因。

## 7. 验收标准

- [ ] `dock-cli doctor` 覆盖 toolchain、DID identity、private key permission、resolver、allowlist、storage/audit path、Host providers、sandbox gates、server health。
- [ ] 输出 pass/warn/fail/skip、evidence 和 suggestion。
- [ ] skip 不被计为 pass；fail 可用于 CI exit code。
- [ ] private key material、token、Authorization、signature 和 secret 不出现在 report。
- [ ] README/runbook/Phase 5 文档与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Doctor tests | `cd anp/anp-miniapp-dock && cargo test -p dock-cli doctor` | doctor checks/report tests 通过 |
| Manual doctor | `cd anp/anp-miniapp-dock && cargo run -p dock-cli -- doctor` | 输出 redacted diagnostic；环境相关 skip/fail 需记录 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-cli crates/anp-adapter crates/dock-core docs/runbook docs/plan README.md` | 无空白错误 |
| 脱敏抽样 | 手工检查 doctor report | 不含 private key material、raw token、Authorization、signature 或真实 secret |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：doctor 是否读取/输出 secret；skip/fail 语义是否清晰；建议是否可执行；是否与 release gates 术语一致。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 doctor command、direct tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase5: add doctor diagnostics`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 05-05 小 Plan | 将 CLI doctor 环境诊断拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：doctor 为了“帮忙”读取 secret 会造成泄露。
- 回滚 / 回退：只检查存在性、权限和 redacted path；不读取 private key material。
- 后续文档：运维 runbook 应引用 doctor 作为故障定位第一步。
