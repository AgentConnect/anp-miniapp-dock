# Step 04-02：IPC / SDK 形态与 Host 进程边界

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：04-02
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
| Next action | 等待 04-01 完成后，启动 IPC / SDK 边界 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：定义 Host 集成形态：Rust library embedding、local HTTP/JSON-RPC sidecar、headless CLI JSON mode 的边界和首个可测试实现。
- 用户 / 系统可见行为：非 Rust Host 可以通过稳定 IPC 或 CLI JSON 调用 Runtime API，且不把 CLI human output 当生产协议。
- 非目标：不同时实现 gRPC、HTTP 和所有 SDK；先选择最小可验证路径。
- 完成标准：IPC/SDK contract 复用 Step 04-01 Runtime facade，包含 version、auth/binding、error/redaction、lifecycle 和 conformance tests。

## 3. 设计方法

- 设计边界：IPC 只是 Runtime facade 的传输层，不允许绕过 permission、consent、audit、redaction。
- 核心决策：先稳定 Rust facade，再选择 local HTTP/JSON-RPC 或 headless CLI JSON 作为首个 Host 接入候选。
- 契约 / API / 数据流：Host process -> IPC transport -> RuntimeService -> RuntimeResult -> redacted JSON response。
- 兼容性：CLI developer output 与 machine JSON output 分离；现有 CLI 行为保持兼容。
- 风险控制：local IPC 默认 bind loopback 或 Unix socket；不在 IPC payload 暴露 raw token/private key。

## 4. 实现方法

1. 阅读 Step 04-01 Runtime API facade 和 Phase 4 IPC/SDK 计划。
2. 选择首个 IPC/SDK 形态并在计划中记录理由：Rust embedding、local HTTP/JSON-RPC 或 headless CLI JSON。
3. 定义 request/response envelope：apiVersion、requestId、session、method、params、error、redaction marker。
4. 实现首个 transport 或 headless JSON mode，并复用 Runtime facade。
5. 增加 tests：version mismatch、invalid method、redacted error、loopback binding、CLI JSON schema。
6. 更新 Host adapter guide 计划、Phase 4 文档和 runbook。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/dock-core` | IPC envelope / service trait 复用 Runtime facade | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/dock-cli` | headless JSON mode 或 sidecar command | 代码实现 |
| `anp/anp-miniapp-dock/crates/demo-server` | 若选择 local HTTP/JSON-RPC，可复用或新增 transport | 视决策修改 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-4-runtime-host-integration.md` | 同步 IPC/SDK 决策 | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/local-demo.md` | 同步本地 Host/sidecar 启动方式 | 视实现结果更新 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/04-02-ipc-sdk-host-process-boundary.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 04-01。
- 外部文档或决策：Runtime API contract、Threat Model、Release Gates。
- 环境前提：Rust toolchain 1.88.0；无需真实 Mac/Flutter Host。

## 7. 验收标准

- [ ] 选定首个 IPC/SDK 形态并记录为什么先做它。
- [ ] IPC/SDK envelope 包含 version、requestId、method、params、error、redaction 标记。
- [ ] Transport 复用 Runtime facade，不绕过 permission/consent/audit。
- [ ] Machine-readable JSON 与 human CLI output 分离。
- [ ] version mismatch、invalid method、redaction、loopback/local binding 有 tests 或明确记录替代证据。
- [ ] 文档记录 Host 集成方式、限制和生产安全要求。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| IPC/CLI tests | `cd anp/anp-miniapp-dock && cargo test -p dock-cli ipc` | IPC/headless JSON tests 通过；若 filter 不匹配，记录实际命令 |
| Runtime tests | `cd anp/anp-miniapp-dock && cargo test -p dock-core runtime` | facade 仍通过 |
| Coffee 回归 | `cd anp/anp-miniapp-dock && cargo test -p dock-cli --test coffee_order_flow` | 通过 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/dock-core crates/dock-cli crates/demo-server docs/runbook docs/plan` | 无空白错误 |
| 脱敏抽样 | 手工检查 IPC/CLI JSON | 不含 token、Authorization、signature、private key path 或隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：transport 是否仅传输 facade；是否有本地绑定安全；machine JSON 是否稳定；错误是否脱敏。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 IPC/SDK transport、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase4: add runtime ipc boundary`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 04-02 小 Plan | 将 IPC / SDK 形态与 Host 进程边界拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：过早冻结复杂 IPC 会拖慢 Runtime API 收敛。
- 回滚 / 回退：若 sidecar 未成熟，保留 Rust facade + headless JSON mode 作为首个稳定集成面。
- 后续文档：Phase 5 Host adapter guide 需要引用本 IPC/SDK envelope。
