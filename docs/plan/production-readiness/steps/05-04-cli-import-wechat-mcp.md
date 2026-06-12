# Step 05-04：CLI import-wechat-mcp

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：05-04
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
| Next action | 等待 05-03 完成后，启动 import-wechat-mcp |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：新增 `dock-cli import-wechat-mcp`，复制/导入小程序 MCP Skill 到容器测试目录，保留原字段并输出兼容报告与 ANP `_meta` 建议 patch。
- 用户 / 系统可见行为：开发者能导入已有 Skill，看到缺失文件、unsupported API、权限建议和需要改造的 ANP DID / Host provider 点。
- 非目标：不自动强制修改业务逻辑；不承诺完整微信 runtime 兼容。
- 完成标准：导入流程不破坏原字段，不越过 path/supply-chain gate，并可接 validate/test-skill。

## 3. 设计方法

- 设计边界：import 是迁移辅助工具，不能把不安全能力自动改成 production-ready。
- 核心决策：只复制到受控目录或输出 patch；默认 dry-run；写入时保留 `SKILL.md`、`mcp.json`、`index.js`、components，并记录改动清单。
- 契约 / API / 数据流：source dir -> validate structure -> safe copy/dry-run -> compatibility report -> optional ANP `_meta` suggestion。
- 兼容性：识别 `app.json agent.skills[]`，但不复刻微信页面路由。
- 风险控制：拒绝绝对路径逃逸、symlink escape、remote require；输出 redacted。

## 4. 实现方法

1. 阅读微信 MCP 协议参考、skill-loader path rules、validate report schema。
2. 定义 import options：source、dest、dry-run、overwrite policy、generate-patch、include fixtures。
3. 实现结构检查：`SKILL.md`、`mcp.json`、`index.js`、`apis/*.js`、components、`app.json agent.skills[]`。
4. 输出兼容报告和 ANP `_meta` / permission / provider 建议 patch，但默认不改业务代码。
5. 增加 tests：dry-run、safe copy、path escape deny、symlink deny、missing file report、patch output redacted。
6. 更新开发者迁移文档、Phase 5 文档和 README。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-cli` | `import-wechat-mcp` command、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/skill-loader` | safe copy/path validation helper 复用 | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/weichat-miniapp-mcp-protocol` | 只读参考 | 不修改或视链接更新 |
| `anp/anp-miniapp-dock/docs/developer/import-wechat-mcp-skill.md` | 新增迁移文档 | 计划新增 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-5-developer-experience.md` | 同步 import contract | 必须 |
| `anp/anp-miniapp-dock/README.md` | 视 CLI 使用说明更新 | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/05-04-cli-import-wechat-mcp.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 05-01、Step 05-02。
- 外部文档或决策：微信 MCP 协议参考、Skill package integrity gate、validate report schema。
- 环境前提：Rust toolchain 1.88.0；导入样例必须 mock-only。

## 7. 验收标准

- [ ] `dock-cli import-wechat-mcp` 支持 dry-run 和 safe copy。
- [ ] 导入不会删除或强制改写原业务字段，patch 建议单独输出。
- [ ] 识别 `SKILL.md`、`mcp.json`、`index.js`、API JS、components 和 `app.json agent.skills[]`。
- [ ] path escape、symlink escape、overwrite risk fail closed 或需要显式确认。
- [ ] 输出兼容报告可接 `validate` / `test-skill`。
- [ ] 迁移文档说明 ANP DID、Host provider、unsupported API 和权限声明改造。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Import tests | `cd anp/anp-miniapp-dock && cargo test -p dock-cli import` | import dry-run/copy/path tests 通过 |
| Loader path tests | `cd anp/anp-miniapp-dock && cargo test -p skill-loader` | path boundary tests 不回归 |
| Manual dry-run | `cd anp/anp-miniapp-dock && cargo run -p dock-cli -- import-wechat-mcp examples/coffee-skill --dry-run` | 输出报告；若命令参数不同，记录实际命令 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-cli crates/skill-loader docs/developer docs/plan README.md` | 无空白错误 |
| 安全抽样 | 手工检查 import report/patch | 不含 token、Authorization、signature、private key path 或真实隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：import 是否默认 dry-run 安全；是否保留原字段；patch 是否不伪造 production-ready；path/copy 是否安全。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 import-wechat-mcp、直接 tests 和迁移文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase5: add wechat mcp import command`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 05-04 小 Plan | 将 CLI import-wechat-mcp 拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：自动改写业务逻辑会破坏开发者原始 Skill。
- 回滚 / 回退：默认 dry-run + patch suggestion；写入必须显式目标目录和 overwrite policy。
- 后续文档：迁移指南应把 import 作为第一步，而不是替代人工安全 review。
