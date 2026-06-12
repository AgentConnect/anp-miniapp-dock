# Step 03-02：QuickJS 沙箱逃逸回归与资源限制

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：03-02
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
| Next action | 等待 03-01 完成后，启动 QuickJS 沙箱加固 |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：把 Atomic API VM 和 Component VM 的 sandbox escape regression 与资源限制升级为 CI/release gate。
- 用户 / 系统可见行为：恶意 Skill 无法通过 constructor/eval/Function/process/fetch/WebSocket/timer/require 逃逸，超限行为返回稳定失败并可审计。
- 非目标：不开放新的 dynamic 能力；不替换 QuickJS-NG 引擎。
- 完成标准：沙箱策略、逃逸回归测试、memory/stack/CPU/Promise/log/result size 限制和 release gate 证据齐备。

## 3. 设计方法

- 设计边界：Skill JS 默认不可信，任何 API 或组件能力都只能由 broker/profile 显式开放。
- 核心决策：Atomic API VM 与 Component VM 共用安全红线，但可有不同能力 profile；dynamic timer/request 只在 Step 02-05 声明后受控开放。
- 契约 / API / 数据流：Skill JS -> QuickJS context -> sandbox policy -> broker/profile -> outcome/audit；limit hit -> stable error + redacted diagnostic。
- 兼容性：保持 coffee demo 和已支持 API 行为；只把逃逸入口和超限路径稳定化。
- 风险控制：console/result/debug 输出必须截断并脱敏；timeout 后不得继续执行高风险 callback/action。

## 4. 实现方法

1. 阅读 `js-runtime-quickjs`、`component-runtime` 当前 sandbox 初始化、timeout 和 tests。
2. 增加或收敛 sandbox policy 配置：禁用 `eval`、`Function`、async/generator constructor、prototype constructor、`process`、`fetch`、`WebSocket`、未授权 timer、remote require。
3. 增加资源限制：memory、stack、CPU/interrupt timeout、Promise job drain、console size、result size。
4. 为 Atomic API VM 和 Component VM 增加 focused escape tests 与 limit tests。
5. 将 sandbox gate 写入 `docs/runbook/release-gates.md` 和 threat model 控制矩阵。
6. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/js-runtime-quickjs` | API VM sandbox policy、resource limit、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/component-runtime` | Component VM sandbox policy、dynamic cleanup regression、tests | 代码实现 |
| `anp/anp-miniapp-dock/docs/security/threat-model.md` | 同步 sandbox 控制与残余风险 | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | sandbox escape regression gate 升级 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/phase-3-security-hardening.md` | 同步 Phase 3 sandbox 状态 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/03-02-quickjs-sandbox-resource-limits.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 02-05、Step 03-01。
- 外部文档或决策：Threat Model、安全红线、Release Gates。
- 环境前提：Rust toolchain 1.88.0；无需外部 Host provider。

## 7. 验收标准

- [ ] Atomic API VM 和 Component VM 都有 constructor/eval/Function/process/fetch/WebSocket/require escape regression tests。
- [ ] memory、stack、CPU timeout、Promise job drain、console size、result size 至少有 focused test 或明确 skip 原因。
- [ ] limit hit 返回稳定脱敏错误，不泄露 JS 源码中的敏感值或 Host private data。
- [ ] Component expire/detach 后不能继续触发事件、timer 或高风险 action。
- [ ] Release Gates 将 sandbox escape regression 列为 required。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| API VM sandbox tests | `cd anp/anp-miniapp-dock && cargo test -p js-runtime-quickjs sandbox` | escape 和资源限制 tests 通过；若 filter 不匹配，记录实际命令 |
| Component sandbox tests | `cd anp/anp-miniapp-dock && cargo test -p component-runtime sandbox` | component escape、timer/default deny、detach tests 通过 |
| Workspace 回归 | `cd anp/anp-miniapp-dock && cargo test --workspace` | 通过；如耗时受限，记录 focused 替代和风险 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/js-runtime-quickjs crates/component-runtime docs/security docs/runbook docs/plan` | 无空白错误 |
| 脱敏抽样 | 手工检查 limit error、console、debug 输出 | 不含 token、Authorization、signature、private key path 或隐私原文 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：逃逸入口是否完整；限制是否可配置且默认安全；timeout 后是否停止后续 action；dynamic 例外是否只通过 capability profile 开放。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 sandbox/resource limit、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase3: harden quickjs sandbox limits`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 03-02 小 Plan | 将 QuickJS sandbox 加固拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：资源限制过严会破坏合法 Skill，过松会留下 DoS 面。
- 回滚 / 回退：限制默认值可配置但 production profile 必须保守；任何放宽都要更新 threat model 和 release gates。
- 后续文档：Phase 6 性能基线需要记录 sandbox limit hit 和 VM resource metrics。
