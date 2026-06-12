# Step 01-07：Device/App Info Atomic API

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：01-07
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
| Next action | 等待 Step 01-06 完成后，启动 device/app info Atomic API；若 Step 01-06 blocked，可按主 Plan Blocked 规则串行跳转到本 Step |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：在 Atomic API VM 中补齐 `wx.getDeviceInfo()` 和 `wx.getAppBaseInfo()` 的最小实现。
- 用户 / 系统可见行为：Skill 原子接口可读取容器和 Host 提供的最小 app/runtime/device snapshot，且不暴露真实设备指纹。
- 非目标：不复刻微信客户端完整环境；不实现 `wx.getAccountInfoSync`、网络状态监听或完整设备传感器 API。
- 完成标准：Atomic API 与 Component VM 的 device/app info 字段口径一致，字段最小化，默认 headless fallback deterministic。

## 3. 设计方法

- 设计边界：device/app info 只能暴露低风险、最小化、非指纹化字段；真实 Host 可覆盖但必须受字段 allowlist 限制。
- 核心决策：优先复用 `wx-compat` 现有 device/app info helper，Atomic API VM 和 Component VM 保持同一结构来源或有防漂移测试。
- 契约 / API / 数据流：Atomic API JS 调用 sync API -> runtime snapshot provider -> 返回脱敏 object；provider 缺失时返回 deterministic headless/container 默认值。
- 兼容性：按 Step 01-01 sync API 规则，不使用 callback，不返回 Promise，失败抛出脱敏 Error。
- 风险控制：不得返回真实 device id、local IP、MAC、广告标识、文件路径、Host 用户账号或精细地理信息。

## 4. 实现方法

1. 阅读 `anp/anp-miniapp-dock/docs/architecture/wx-api-compatibility-matrix.md` 中 device/app info 状态。
2. 阅读 `anp/anp-miniapp-dock/crates/wx-compat` 和 `anp/anp-miniapp-dock/crates/component-runtime` 中 Component VM 已支持的 device/app info helper。
3. 在 `wx-compat` 中冻结 Atomic API 和 Component VM 共用字段结构、默认值和 redaction 规则。
4. 在 `js-runtime-quickjs` 注入 `wx.getDeviceInfo()` 和 `wx.getAppBaseInfo()` 同步 API。
5. 增加 tests：字段最小化、headless fallback、Component VM 与 Atomic API 防漂移、callback 被忽略或 invalid、敏感字段不出现。
6. 更新 API 矩阵和必要 runbook。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/wx-compat` | device/app info schema、snapshot helper、tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/js-runtime-quickjs` | Atomic API VM sync bridge 和 tests | 代码实现 |
| `anp/anp-miniapp-dock/crates/component-runtime` | 如需共用 helper 或防漂移测试 | 视当前结构修改 |
| `anp/anp-miniapp-dock/docs/architecture/wx-api-compatibility-matrix.md` | 同步 Atomic API device/app info 状态 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/01-07-device-app-info-atomic-api.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 01-01。
- 外部文档或决策：wx API Bridge Contract、wx API 兼容矩阵、Threat Model。
- 环境前提：Rust toolchain 1.88.0；无需外部 Host provider。
- 独立性说明：本 Step 不依赖 Step 01-06 storage bridge 的实现。若 Step 01-06 被标记为 `blocked`，执行者可以在主 Plan Blocked 记录中说明原因、确认无未提交完成工作后，串行转入本 Step；这不授权并行执行。

## 7. 验收标准

- [ ] Atomic API VM 支持 `wx.getDeviceInfo()` 和 `wx.getAppBaseInfo()` 同步调用。
- [ ] 返回字段最小化且 deterministic，缺省 headless 环境不返回真实设备指纹。
- [ ] Atomic API 与 Component VM 的 runtime/app info 字段不漂移，已有测试或共用 helper。
- [ ] 不返回 local IP、MAC、device id、广告标识、Host 账号、credential path 或私钥路径。
- [ ] API 矩阵与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Focused VM tests | `cd anp/anp-miniapp-dock && cargo test -p js-runtime-quickjs info` | Atomic API device/app info tests 通过；若 filter 不匹配，记录实际命令 |
| Compat tests | `cd anp/anp-miniapp-dock && cargo test -p wx-compat device` | helper / redaction tests 通过 |
| Component 回归 | `cd anp/anp-miniapp-dock && cargo test -p component-runtime` | Component VM 现有 device/app info 不回归 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/wx-compat crates/js-runtime-quickjs crates/component-runtime docs/architecture docs/plan` | 无空白错误 |
| 敏感字段抽样 | 手工检查返回 JSON 和测试 fixture | 不含真实设备指纹或 Host credential 信息 |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：字段是否过多；是否暴露 fingerprint；Atomic API 与 Component VM 是否漂移；sync API 是否误走 Promise/callback。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含 device/app info Atomic API、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase1: add device app info atomic api`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 01-07 小 Plan | 将 Phase 1 device/app info Atomic API 拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |
| 2026-06-12 | 标注 01-07 独立性 | 按 Review 发现，避免 storage blocked 时错误阻塞独立 device/app info work | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：过度兼容微信字段可能变成设备指纹面。
- 回滚 / 回退：若字段口径不清，先保持 unsupported 或最小 headless snapshot，不引入真实 Host 字段。
- 后续文档：Phase 4 Host adapter contract 需要定义 Host 可覆盖字段和 conformance tests。
