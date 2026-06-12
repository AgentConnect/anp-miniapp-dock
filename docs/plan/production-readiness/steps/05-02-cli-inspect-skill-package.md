# Step 05-02：CLI inspect Skill package

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：05-02
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
| Next action | 等待 05-01 完成后，启动 CLI inspect |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：新增或增强 `dock-cli inspect`，展示 Skill package 文件、API 注册与 manifest 对照、组件路径、权限需求、风险等级和 `wx.*` 使用痕迹。
- 用户 / 系统可见行为：开发者能定位 manifest/注册不一致、组件路径、风险 API 和 potential fallback，而不读 Rust 源码。
- 非目标：不执行 Skill API；不做完整 JS 静态分析器。
- 完成标准：inspect 输出 machine-readable JSON 和必要 human summary，敏感路径/参数 redacted。

## 3. 设计方法

- 设计边界：inspect 是静态和轻量 runtime metadata 检查，不调用高风险 provider。
- 核心决策：优先基于 manifest、loader、registration trace 和简单 `wx.*` scan；无法确定时输出 unknown-with-reason，而不是猜测 supported。
- 契约 / API / 数据流：Skill package -> loader/schema -> inspect graph -> JSON/human output。
- 兼容性：复用 validate report 的状态枚举和 redaction helper。
- 风险控制：不输出 private key path、raw local absolute secret path、token、Authorization 或真实隐私 payload。

## 4. 实现方法

1. 阅读 `dock-cli validate` report 和 skill-loader manifest/registration evidence。
2. 定义 inspect output：files、apis、registeredApis、components、permissions、riskLevels、wxApiUsage、warnings。
3. 实现 package file tree 安全展示，路径相对 Skill root，拒绝包外路径。
4. 增加 `wx.*` usage scan 或 runtime trace summary，并标注静态扫描限制。
5. 增加 tests：coffee inspect、registration mismatch、component path、permission/risk summary、redaction。
6. 更新 Phase 5 文档和 README/CLI docs。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-cli` | `inspect` command、output schema、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/skill-loader` | package file tree / registration metadata | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/mcp-schema` | inspect DTO/status enum 复用 | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-5-developer-experience.md` | 同步 inspect contract | 必须 |
| `anp/anp-miniapp-dock/README.md` | 视 CLI 文档更新 | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/05-02-cli-inspect-skill-package.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 05-01。
- 外部文档或决策：validate report schema、Skill package integrity contract。
- 环境前提：Rust toolchain 1.88.0；无需外部 Host provider。

## 7. 验收标准

- [ ] `dock-cli inspect` 输出 Skill package 文件、API/registration 对照、componentPath、permissions、risk、wxApiUsage。
- [ ] 输出支持 JSON，可由 CI/文档示例复用。
- [ ] 路径以 Skill root 相对路径展示，包外路径 fail closed 或 redacted。
- [ ] 静态扫描无法确定的能力标注 unknown-with-reason，不误标 supported。
- [ ] 输出 redacted，不含 token、Authorization、signature、private key path 或隐私原文。
- [ ] Phase 5 文档和 README/CLI docs 与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| CLI inspect tests | `cd anp/anp-miniapp-dock && cargo test -p dock-cli inspect` | inspect tests 通过 |
| Loader tests | `cd anp/anp-miniapp-dock && cargo test -p skill-loader` | package path/loader tests 不回归 |
| Manual inspect | `cd anp/anp-miniapp-dock && cargo run -p dock-cli -- inspect examples/coffee-skill` | 输出 JSON/human summary，redacted |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-cli crates/skill-loader crates/mcp-schema docs/plan README.md` | 无空白错误 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：inspect 是否不执行高风险代码；路径是否安全；unknown 是否不被误标 supported；输出是否可读且 redacted。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 inspect command、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase5: add skill inspect command`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 05-02 小 Plan | 将 CLI inspect Skill package 拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：过度静态分析容易给出错误确定性。
- 回滚 / 回退：不确定时输出 unknown-with-reason，并建议运行 test-skill。
- 后续文档：迁移指南应展示 inspect + validate 的组合用法。
