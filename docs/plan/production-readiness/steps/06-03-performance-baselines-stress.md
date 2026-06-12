# Step 06-03：性能基线与 Stress Tests

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：06-03
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
| Next action | 等待 06-02 完成后，启动性能基线 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：建立 Skill load、API VM call、Component render、Render IR size、token lookup、storage read/write、memory per VM 的性能基线和 stress tests。
- 用户 / 系统可见行为：release notes 能记录实际 P50/P95 和资源边界，性能退化可在 CI 或 release 前发现。
- 非目标：不凭空承诺生产 SLO；不为了性能绕过 sandbox/session 隔离。
- 完成标准：benchmark/stress 脚本可运行，结果可归档，超限 fail closed 不影响其他 session。

## 3. 设计方法

- 设计边界：性能基线是证据，不是营销承诺；数据只来自自动化 benchmark。
- 核心决策：记录 P50/P95、max memory、Render IR size；压力场景覆盖并发 session、多 Skill、多组件、dynamic timer。
- 契约 / API / 数据流：benchmark runner -> Runtime API -> metrics recorder -> baseline artifact -> release notes。
- 兼容性：benchmark 使用 mock/dev-only data，不依赖真实 Host provider。
- 风险控制：stress tests 不输出隐私 payload；资源超限必须 fail closed。

## 4. 实现方法

1. 阅读 Phase 6 performance plan、metrics/tracing 实现和 release gates。
2. 定义 benchmark cases：Skill load、API cold/warm call、component render、token lookup、storage read/write、Render IR size、memory per VM。
3. 定义 stress cases：并发 session、多 Skill、多组件渲染、dynamic timer/request。
4. 实现 benchmark runner 或 cargo bench/test harness，输出 JSON baseline artifact。
5. 增加 CI-friendly smoke benchmark 和可选 full benchmark。
6. 更新 Phase 6 文档、release gates 和 release notes 模板。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-core/benches` | Runtime performance benchmarks | 计划新增或按 workspace 结构调整 |
| `anp/anp-miniapp-dock/crates/component-runtime/benches` | render benchmarks | 计划新增或按 workspace 结构调整 |
| `anp/anp-miniapp-dock/crates/dock-cli` | benchmark command 或 report helper | 视实现结果更新 |
| `anp/anp-miniapp-dock/testdata/perf` | baseline artifact | 计划新增 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | performance gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-6-observability-release.md` | 同步性能基线策略 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/06-03-performance-baselines-stress.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 04-06、Step 06-02。
- 外部文档或决策：Runtime API、metrics recorder、Release Gates。
- 环境前提：Rust toolchain 1.88.0；benchmark 结果受硬件影响，release notes 必须记录环境。

## 7. 验收标准

- [ ] Benchmark 覆盖 Skill load、API VM call、component render、Render IR size、token lookup、storage read/write、memory per VM。
- [ ] Stress tests 覆盖并发 session、多 Skill、多组件、dynamic timer/request。
- [ ] 输出 JSON baseline artifact，包含环境、commit、P50/P95 或明确替代指标。
- [ ] CI-friendly smoke benchmark 与 full benchmark 区分清楚。
- [ ] 超过资源限制时 fail closed，不影响其他 session。
- [ ] Release Gates 和 Phase 6 文档与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Smoke perf tests | `cd anp/anp-miniapp-dock && cargo test --workspace perf` | smoke benchmark/stress tests 通过；若 filter 不匹配，记录实际命令 |
| Full benchmark | `cd anp/anp-miniapp-dock && cargo bench --workspace` | 如环境允许，记录 benchmark artifact；不能运行时记录原因 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates testdata docs/runbook docs/plan` | 无空白错误 |
| 敏感信息扫描 | 手工检查 benchmark output/artifacts | 不含 token、Authorization、signature、private key path 或隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：benchmarks、tests、文档同步完成后、commit 前。
- Review 重点：benchmark 是否可复现；是否记录环境；stress 是否覆盖风险场景；是否为了性能弱化安全隔离。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 performance baseline/stress tests、artifacts 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase6: add performance baselines`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 06-03 小 Plan | 将性能基线与 Stress Tests 拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：不记录环境的 benchmark 没有可比性。
- 回滚 / 回退：性能阈值先作为 release warning，待多环境数据稳定后升级 blocker。
- 后续文档：Step 06-05 release notes 必须引用 benchmark artifact。
