# Step 01-08：高风险 API Host Boundary 与 fail-closed

主 Plan：[../../production-readiness-roadmap.md](../../production-readiness-roadmap.md)
Step index：01-08
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
| Next action | 等待 Step 01-07 完成后，启动高风险 Host provider boundary |

状态取值：`pending`、`in_progress`、`review`、`blocked`、`committed`、`done`。

## 2. 目标

- 结果：为 phone、address、location、media/file、payment、scan、phone call 等 L3/L4 API 建立 Host provider trait、ConsentGate 接线和未配置时 fail-closed 行为。
- 用户 / 系统可见行为：高风险 API 不再是 undefined；无 provider 或无 consent 时返回稳定失败；有 mock/headless provider 时必须显式 dev-only 标识。
- 非目标：不实现真实手机号、地址簿、定位、媒体选择、支付收银台、扫码或拨号 provider；不复刻微信支付和设备 UI。
- 完成标准：所有 P1 高风险 API 的调用路径都经过 provider boundary、permission/consent/audit 检查或 deterministic fail closed。

## 3. 设计方法

- 设计边界：Skill JS 只能请求能力，不能直接拿到 Host 原始隐私数据、真实文件路径、支付密码、设备拨号或扫码结果。
- 核心决策：Host provider trait 先定义最小 input/output/error shape；默认 provider 为 unavailable / unsupported；L3/L4 在 provider 执行前必须过 ConsentGate。
- 契约 / API / 数据流：`wx.getPhoneNumber` 等 API -> broker normalize -> permission/risk classify -> ConsentGate -> Host provider trait -> redacted `WxApiOutcome` / audit summary。
- 兼容性：失败 shape 复用 Step 01-01；unsupported registry 和 provider_unavailable 要可区分。
- 风险控制：opaque file handle 替代本地路径；payment 只返回 Payment Intent 或 merchant API 脱敏状态；audit 不保存隐私原文。

## 4. 实现方法

1. 阅读 `wx-api-compatibility-matrix.md` 中 L3/L4 API 状态和风险等级。
2. 阅读 `docs/security/threat-model.md`、`docs/runbook/release-gates.md` 和 `consent-audit` / `dock-core` 现有 consent flow。
3. 在 `wx-compat` 或 `dock-core` 定义高风险 Host provider trait 和统一 error shape，覆盖 phone、address、location、media/file、payment、scan、phone call 的最小契约。
4. 将 Atomic API VM 中对应 API 接到 broker boundary；未配置 provider 时返回 `provider_unavailable` 或 `consent_required`，不得执行 mock。
5. 增加 tests：无 provider fail closed、无 consent 不执行 provider、mock provider dev-only 标识、audit redaction、opaque file handle、本地路径拒绝、payment 不收集密码。
6. 更新 API 矩阵、Threat Model、Release Gates 和必要 runbook。
7. 回填本 Step 和主 Plan 执行台账。

## 5. 路径

| 仓库 / 模块 / 文件 | 计划变更 | 备注 |
|---|---|---|
| `anp/anp-miniapp-dock/crates/wx-compat` | 高风险 API provider trait、risk/error shape | 代码实现 |
| `anp/anp-miniapp-dock/crates/dock-core` | Orchestrator / ConsentGate / audit 接线 | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/consent-audit` | consent proof、risk policy、redaction tests | 视当前结构修改 |
| `anp/anp-miniapp-dock/crates/js-runtime-quickjs` | Atomic API bridge 接入高风险 broker | 代码实现 |
| `anp/anp-miniapp-dock/docs/architecture/wx-api-compatibility-matrix.md` | 同步 phone/address/location/media/payment/scan/phone call 状态 | 必须 |
| `anp/anp-miniapp-dock/docs/security/threat-model.md` | 同步 Host provider 残余风险和控制 | 必须 |
| `anp/anp-miniapp-dock/docs/runbook/release-gates.md` | 同步 L3/L4 provider gate | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` | 回填执行台账 | 必须 |
| `anp/anp-miniapp-dock/docs/plan/production-readiness/steps/01-08-high-risk-api-host-boundary.md` | 回填状态、证据、Review、commit | 必须 |

## 6. 依赖

- 前置步骤：Step 01-01、Step 01-04、Step 01-05。
- 外部文档或决策：wx API Bridge Contract、wx API 兼容矩阵、Threat Model、Release Gates。
- 环境前提：Rust toolchain 1.88.0；真实 Host provider 不存在时必须以 fail-closed tests 证明边界。

## 7. 验收标准

- [ ] P1 高风险 API 有统一 Host provider boundary 或明确 fail-closed stub。
- [ ] 未配置 provider 时返回 `provider_unavailable` / `unsupported`，不能使用 mock 冒充 production。
- [ ] 未通过 ConsentGate 时返回 `consent_required`，且 provider 不会被调用。
- [ ] phone/address/location/file/media/payment 结果最小化；file/media 只返回 opaque handle，不返回本地路径或原始文件内容。
- [ ] `wx.requestPayment` 不复刻微信收银台，不采集支付密码，只走 Payment Intent / merchant API boundary。
- [ ] audit 只记录脱敏 summary、risk level 和 proof id，不保存隐私原文。
- [ ] API 矩阵、Threat Model、Release Gates 与实现状态同步。
- [ ] Review 发现已经修复或明确记录。
- [ ] 本步骤在进入下一步之前已经创建 focused commit，并回填主 Plan 执行台账。

## 8. 验证方式

| 检查项 | 命令 / 方法 | 预期证据 |
|---|---|---|
| 格式 | `cd anp/anp-miniapp-dock && cargo fmt --check` | 通过 |
| Consent / audit tests | `cd anp/anp-miniapp-dock && cargo test -p consent-audit -p dock-core consent` | consent gate 和 redaction tests 通过；若 filter 不匹配，记录实际命令 |
| VM high-risk tests | `cd anp/anp-miniapp-dock && cargo test -p js-runtime-quickjs high_risk` | 无 provider、无 consent、mock/dev-only、redaction 测试通过 |
| Compat tests | `cd anp/anp-miniapp-dock && cargo test -p wx-compat provider` | provider trait / error shape 测试通过 |
| Workspace 回归 | `cd anp/anp-miniapp-dock && cargo test --workspace` | 通过；如耗时受限，记录 focused 替代和风险 |
| 文档/空白 | `cd anp/anp-miniapp-dock && git diff --check -- crates/wx-compat crates/dock-core crates/consent-audit crates/js-runtime-quickjs docs/architecture docs/runbook docs/security docs/plan` | 无空白错误 |
| 安全抽样 | 手工检查 API result、audit、CLI JSON | 不含手机号、地址、文件内容、精确位置、token、Authorization、signature、private key path |

如果某个命令不能运行，必须记录原因、影响和替代证据。

## 9. Review 环节

- Review 时机：实现、测试、文档同步完成后、commit 前。
- Review 重点：provider 是否能被绕过；ConsentGate 是否在 provider 前执行；mock 是否显式 dev-only；audit 是否脱敏；payment/file/location 是否保持 Host boundary。

| Review 项 | 结果 | 备注 |
|---|---|---|
| 发现问题 | 待记录 | 待记录 |
| 已修复问题 | 待记录 | 待记录 |
| 剩余风险 | 待记录 | 待记录 |
| 新增或缺失测试 | 待记录 | 待记录 |
| 已更新或缺失文档 | 待记录 | 待记录 |

## 10. Commit 要求

- Commit 时机：实现、验证、Review、文档同步完成后。
- Commit 范围：只包含高风险 API Host boundary、直接 tests 和相关文档。
- Commit 前状态：记录 `git status --short`。
- 纳入文件：记录本步骤 commit 包含的文件。
- Commit 后证据：记录 commit hash 和 commit 后 `git status --short --branch`。
- 遗留未提交变更：必须记录原因以及为什么安全。
- 建议消息：`phase1: add high risk api host boundary`

## 11. Blocked 处理

| Blocker | 证据 | 已尝试方案 | 影响范围 | 下一步决策 |
|---|---|---|---|---|
| 待记录 | 待记录 | 待记录 | 当前步骤 / 整体计划 | 待记录 |

## 12. Plan 变更记录

| 日期 | 变更 | 原因 | 主 Plan 变更记录链接 |
|---|---|---|---|
| 2026-06-12 | 创建 Step 01-08 小 Plan | 将 Phase 1 高风险 API Host boundary 拆成可执行 Step | `anp/anp-miniapp-dock/docs/plan/production-readiness-roadmap.md` |

## 13. 风险、回滚与后续文档

- 风险：过早引入 mock provider 容易被误标 production-ready；provider trait 过宽会放大隐私面。
- 回滚 / 回退：真实 provider 不明确时保持 fail closed；任何 mock 都必须 dev-only 并被 release gates 阻断。
- 后续文档：Phase 3/4 需要补真实 Host consent UI、provider conformance tests、persistent audit 和 release profile。
