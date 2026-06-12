# Step 02-04：表单与静态媒体节点

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：02-04
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
| Next action | 等待 Step 02-03 完成后，启动表单与静态媒体节点 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：在 Render IR 中新增 P1 表单和静态媒体节点：`input`、`textarea`、`radio`、`checkbox`、`picker`、`map-preview`、`canvas-static`。
- 用户 / 系统可见行为：组件可以表达地址表单、选择控件、静态地图预览和静态 canvas 占位，Host 可按 node kind 渲染或 fallback。
- 非目标：不实现完整输入法事件模型、复杂 form submit、MapContext 交互、任意 canvas 脚本、真实位置 provider 或真实媒体 provider。
- 完成标准：新增 node kind 有 parser/compiler/Render IR tests，Host 不支持时有 warning/fallback，表单值不能绕过 Orchestrator input validation。

## 3. 设计方法

- 设计边界：表单节点是 Render IR 数据，不直接执行高风险动作；用户提交仍要回到 Host / Orchestrator 受控路径。
- 核心决策：表单节点只输出安全 props、value、placeholder、options、disabled、events；`map-preview` 和 `canvas-static` 只表达静态预览，不开放交互 API。
- 契约 / API / 数据流：WXML node -> compiler -> `RenderNodeKind` -> Host renderer；用户事件 -> controlled action -> Orchestrator / Host provider。
- 兼容性：未知 Host renderer 可以 fallback placeholder 或整卡 fallback；现有 view/text/image/button/scroll-view 不回归。
- 风险控制：位置、地址、文件/media 值不直接从节点进入 Skill API；真实 L4 数据需 Step 01-08 provider boundary 和 consent。

## 4. 实现方法

1. 阅读 `phase-2-render-ir-and-fixtures.md` 中 P1 Node Kind Registry。
2. 阅读 Step 01-08 高风险 Host boundary，确认表单/媒体节点不绕过 provider。
3. 在 `component-runtime` 扩展 `RenderNodeKind`、parser/compiler、props normalization 和 warnings。
4. 为 `input`、`textarea`、`radio`、`checkbox`、`picker` 定义最小 props 和 event 输出；disabled 时阻断 action。
5. 为 `map` preview 和 `canvas` static 定义 `map-preview` / `canvas-static` node，不支持 `MapContext.*` 或 canvas script。
6. 增加 tests：每类 node output、disabled、options、unknown props warning、Host fallback reason、敏感字段不进入 Render IR。
7. 更新组件兼容矩阵、Phase 2 文档和 Render IR 子文档。
8. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/component-runtime/src/render_ir.rs` | 新增 node kind 和 props | 代码实现 |
| `anp/anp-miniapp-dock/crates/component-runtime/src/compiler.rs` | 表单/media node 编译 | 代码实现 |
| `anp/anp-miniapp-dock/crates/component-runtime/src/wxml.rs` | 如需 parser 支持 | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/component-runtime/tests` | node / fallback / warning tests | 必须 |
| `anp/anp-miniapp-dock/crates/dock-core` | 如需 fallback reason 映射 | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/architecture/component-compatibility-matrix.md` | 同步 P1 node 状态 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-2-render-ir-and-fixtures.md` | 同步 Node Kind Registry | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/02-04-form-static-media-nodes.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 01-08、Step 02-01、Step 02-03。
- 外部文档或决策：Render IR contract、组件兼容矩阵、高风险 API Host boundary。
- 环境前提：Rust toolchain 1.88.0；无需真实 Host renderer 或真实位置/media provider。

## 7. 验收标准

- [ ] Render IR 支持 `input`、`textarea`、`radio`、`checkbox`、`picker` node kind 和最小 props。
- [ ] Render IR 支持 `map-preview`、`canvas-static` 静态节点，不开放 MapContext/canvas script。
- [ ] disabled 表单或按钮节点不会产生可执行 action。
- [ ] 表单值进入后续 API 前仍需 Orchestrator input validation、permission、ConsentGate 和 audit。
- [ ] Host 不支持新增 node kind 时 fallback / warning 可观测，不静默执行未知 action。
- [ ] 组件兼容矩阵和 Render IR 子文档与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Node tests | `cd anp/anp-miniapp-dock && cargo test -p component-runtime node` | 新 node tests 通过；若 filter 不匹配，记录实际命令 |
| Component 回归 | `cd anp/anp-miniapp-dock && cargo test -p component-runtime` | 全部 component runtime tests 通过 |
| Core / CLI 回归 | `cd anp/anp-miniapp-dock && cargo test -p dock-cli --test coffee_order_flow` | 通过 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/component-runtime crates/dock-core docs/architecture docs/plan` | 无空白错误 |
| 安全抽样 | 手工检查 Render IR props/actions | 不含手机号、地址原文、精确位置、文件内容、本地路径、token、Authorization、signature |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：新增 node 是否只是数据；是否绕过 consent/input validation；Host unknown node 是否 fallback；map/canvas 是否没有交互 API。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 P1 表单/静态媒体 node、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase2: add form static media nodes`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 02-04 小 Plan | 将 Phase 2 表单与静态媒体节点拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：表单节点如果被误当作直接 API input，会绕过验证和 consent。
- 回滚 / 回退：保持表单输出为 Render IR 数据；Host event 必须回到 Orchestrator。
- 后续文档：Step 02-06 address-form 和 location-map-preview snapshots 必须覆盖这些 node。
