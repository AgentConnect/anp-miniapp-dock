# Step 02-03：WXML/WXSS P1 语法增强

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：02-03
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
| Next action | 等待 Step 02-02 完成后，启动 WXML/WXSS P1 语法增强 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：补齐 WXML `wx:elif`、`wx:else`、`catchtap`、disabled button 抑制、简单表达式，以及 WXSS P1 selector/style 子集。
- 用户 / 系统可见行为：交易型组件可以表达更完整的条件渲染、受控事件和基础布局样式，Render IR 输出稳定 warning。
- 非目标：不支持 template/import/include、slot、自定义组件嵌套、任意 JS 表达式、animation/transition/media query/filter/mask/custom font。
- 完成标准：parser/compiler/style matching 有 focused tests，unsupported 语法产生 warning 或 fallback，不执行任意 JS。

## 3. 设计方法

- 设计边界：WXML/WXSS 子集是安全编译器，不是浏览器或微信完整 runtime；表达式只能使用受限 evaluator。
- 核心决策：支持 `!flag`、`a === b`、literal、simple boolean；禁止 function call、member mutation、arbitrary JS；`catchtap` 阻止向 Host 冒泡但仍只产生受控 event。
- 契约 / API / 数据流：WXML/WXSS source -> parser -> compiler -> Render IR node/style/events/warnings。
- 兼容性：现有 `wx:if` / `wx:for` / class selector 行为不回归；unsupported property 保持 warning 而非 panic。
- 风险控制：disabled button 必须阻断 action；style selector 不得越权读数据；warning 不泄露源文件绝对路径或敏感内容。

## 4. 实现方法

1. 阅读 `component-runtime` 的 `wxml.rs`、`compiler.rs`、`wxss.rs` 和现有 `wxml_bindings.rs` tests。
2. 阅读组件兼容矩阵中 WXML / WXSS P1 能力和 unsupported-by-design 边界。
3. 增强 WXML parser/compiler：`wx:elif` / `wx:else` 链、`catchtap`、disabled button event 抑制、简单表达式 evaluator。
4. 增强 WXSS parser/matcher：id selector、tag selector、simple descendant selector、`gap`、`justify-content`、`align-items`、min/max、`box-shadow`、`overflow-x`。
5. 增加 tests：条件链、disabled action 抑制、catchtap event kind、表达式白名单、表达式黑名单、selector specificity、style output、unsupported warning。
6. 更新组件兼容矩阵和 Phase 2 文档。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/component-runtime/src/wxml.rs` | P1 WXML parser 支持 | 代码实现 |
| `anp/anp-miniapp-dock/crates/component-runtime/src/compiler.rs` | condition/event/expression 编译 | 代码实现 |
| `anp/anp-miniapp-dock/crates/component-runtime/src/wxss.rs` | P1 selector/style 支持 | 代码实现 |
| `anp/anp-miniapp-dock/crates/component-runtime/tests` | parser/compiler/style focused tests | 必须 |
| `anp/anp-miniapp-dock/docs/architecture/component-compatibility-matrix.md` | 同步 WXML/WXSS 状态 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-2-component-runtime-alignment.md` | 同步 Phase 2 P1 状态 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/02-03-wxml-wxss-p1-syntax.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 02-01。
- 外部文档或决策：Component Runtime Alignment、组件兼容矩阵、Render IR contract。
- 环境前提：Rust toolchain 1.88.0；无需真实 Host renderer。

## 7. 验收标准

- [ ] `wx:elif` / `wx:else` 条件链编译和渲染正确，边界条件有测试。
- [ ] `catchtap` 与 `bindtap` 有可区分事件语义，disabled button 不产生可执行 action。
- [ ] 简单表达式白名单可用，function call / arbitrary JS expression fail closed 或 warning fallback。
- [ ] WXSS id/tag/simple descendant selector 和 P1 style 属性输出稳定 RenderStyle 或 warning。
- [ ] unsupported WXML/WXSS 不 panic，不执行 JS，不静默成功。
- [ ] 组件兼容矩阵和 Phase 2 文档与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Parser/compiler tests | `cd anp/anp-miniapp-dock && cargo test -p component-runtime wx` | P1 WXML/WXSS tests 通过；若 filter 不匹配，记录实际命令 |
| Component 回归 | `cd anp/anp-miniapp-dock && cargo test -p component-runtime` | 全部 component runtime tests 通过 |
| Coffee E2E | `cd anp/anp-miniapp-dock && cargo test -p dock-cli --test coffee_order_flow` | 通过 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/component-runtime docs/architecture docs/plan` | 无空白错误 |
| 安全抽样 | 手工检查表达式 evaluator 和 warnings | 未执行任意 JS；warning 不含本地绝对路径或敏感内容 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：表达式 evaluator 是否安全；disabled/catchtap 是否正确；selector matching 是否可预期；unsupported 语法是否 warning/fallback 而非 silent success。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 WXML/WXSS P1 语法、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase2: add wxml wxss p1 syntax`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 02-03 小 Plan | 将 Phase 2 WXML/WXSS P1 语法增强拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：表达式支持扩大后可能意外接近任意 JS 执行。
- 回滚 / 回退：遇到不确定表达式直接 warning/fallback；不在本 Step 支持复杂表达式。
- 后续文档：Step 02-06 snapshots 应覆盖 P1 语法输出，防止 Render IR drift。
