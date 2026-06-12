# Step 06-02：Metrics / Tracing 与请求链路关联

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：06-02
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
| Next action | 等待 06-01 完成后，启动 metrics/tracing |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：建立 metrics 和 tracing correlation，覆盖 API latency、VM time、render latency、request status、fallback rate、consent rate、unsupported count、sandbox limit、token refresh/fail。
- 用户 / 系统可见行为：一次用户请求能通过 traceId 串起 Host message、Skill/API call、wx.login/request、merchant response、render、action、audit。
- 非目标：不绑定具体 APM vendor；不上传敏感 payload。
- 完成标准：metrics 名称、label、cardinality、trace propagation、redaction 和 tests 完成。

## 3. 设计方法

- 设计边界：metrics/traces 用于健康度和性能定位，不记录业务隐私内容。
- 核心决策：低 cardinality labels；DID 默认 hash 或按 policy redacted；traceId 在 Runtime API/IPC/Host adapter/request broker 中传递。
- 契约 / API / 数据流：Runtime request -> trace context -> operations -> metrics recorder -> export/sink boundary。
- 兼容性：可以先用 in-memory/test recorder，Phase 6 后续再接 Prometheus/OpenTelemetry 或 Host sink。
- 风险控制：禁止把 URL query、headers、request body、address/phone/file content 放进 labels。

## 4. 实现方法

1. 阅读 Step 06-01 event model 和 Phase 6 metrics/tracing 计划。
2. 定义 metrics registry、labels 和 trace context propagation。
3. 在 API call、VM execution、render、request、fallback、consent、sandbox、token refresh/fail 路径记录 metrics。
4. 建立 tracing span 关联：Host message -> Runtime API -> broker/provider/render/action/audit。
5. 增加 tests：metrics increments、latency recording、label redaction/cardinality、trace propagation。
6. 更新 Phase 6 文档、runbook 和 release gates。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-core` | metrics recorder、trace context、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/anp-adapter` | request/token metrics hooks | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/js-runtime-quickjs` | VM metrics hooks | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/component-runtime` | render metrics hooks | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/consent-audit` | consent/audit metrics hooks | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | metrics/tracing gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-6-observability-release.md` | 同步 metrics/tracing contract | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/06-02-metrics-tracing-correlation.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 04-02、Step 06-01。
- 外部文档或决策：Runtime API/IPC envelope、observability event model。
- 环境前提：Rust toolchain 1.88.0；可先使用 in-memory recorder。

## 7. 验收标准

- [ ] Metrics 覆盖 Phase 6 列出的核心指标。
- [ ] Labels 低 cardinality，不包含敏感 payload、headers、URL query 或隐私原文。
- [ ] Trace context 能串起 Runtime API、request broker、render、action、audit 的核心路径。
- [ ] In-memory/test recorder 有 unit tests，未来 exporter 有明确 boundary。
- [ ] Phase 6 文档和 runbook 与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Metrics tests | `cd anp/anp-miniapp-dock && cargo test -p dock-core metrics` | metrics/labels tests 通过 |
| Trace tests | `cd anp/anp-miniapp-dock && cargo test -p dock-core trace` | trace propagation tests 通过；若 filter 不匹配，记录实际命令 |
| Coffee E2E | `cd anp/anp-miniapp-dock && cargo test -p dock-cli --test coffee_order_flow` | 通过并可观察 trace/metrics evidence 或记录替代 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-core crates/anp-adapter crates/js-runtime-quickjs crates/component-runtime crates/consent-audit docs/runbook docs/plan` | 无空白错误 |
| 安全抽样 | 手工检查 metrics labels/traces | 不含 token、headers、query、phone/address/file/location 原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：label cardinality 是否可控；trace 是否覆盖关键路径；是否泄露 headers/query/body；是否能定位失败阶段。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 metrics/tracing、direct tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase6: add metrics tracing correlation`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 06-02 小 Plan | 将 Metrics / Tracing 与请求链路关联拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：高 cardinality label 会造成运行成本和隐私风险。
- 回滚 / 回退：不确定字段只进 redacted event，不进入 metric label。
- 后续文档：Step 06-05 canary/rollback 使用这些 metrics 作为决策信号。
