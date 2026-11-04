# Poieweave 产品化设计：Phase A–D

本目录把 Palantir 产品逻辑中最值得吸收的部分转化为四个可执行阶段。它不复制 Palantir 的闭源/咨询模式；它保留 Poieweave 的正式物理契约、SAP 执行语义、可重放证据和开放实现。

## 阶段与顺序

| 阶段 | 文档 | 目标 | 依赖 | 预计首个可验收切片 |
| --- | --- | --- | --- | --- |
| A | [Ontology 产品层](phase-a-ontology-product.zh-CN.md) | 类型化对象/链接/动作/函数目录、SDK、对象浏览器 | 现有 PO/SAP | `poie-objects` + host commands + Capabilities 对象详情 |
| B | [Agent 治理与评测](phase-b-agent-governance.zh-CN.md) | 受控工具、权限/审批、引用、审计、离线 evals | A、现有 auth/inference | `poie-governance` + scripted eval + Agent 面板审批 |
| C | [What-if 工作区](phase-c-what-if.zh-CN.md) | C1 快照/Patch/结构/计划/比较；C2 timed/promote | A、B、plan | `poie-whatif` + Sim 模式 + 变体比较 |
| D | [站点交付](phase-d-site-delivery.zh-CN.md) | D1 manifest/离线导入/状态/回滚/诊断；D2 channel/签名/安装器 | A–C | `poie-delivery` + 设置面板 + 导入/导出 |

严格按 **A → B → C → D** 的阶段顺序实现；每阶段内部按 PLATFORM §8 的切片（A0→A1→A2、B1→B2、C1→C2、D1→D2）推进。后一阶段可以复用前一阶段的类型/策略，但不得绕过前一阶段的安全边界；B/C/D 的设计门禁在各自阶段开始前重跑。

## 跨阶段不变式

1. **SAP 是唯一执行语义**：任何阶段都不能从 UI/Agent/What-if 直接派发；只能生成 proposed/validated 动作，经现有七项复核。
2. **正式契约优先**：Catalog、工具、变体、bundle 都是投影/策略，不是第二套本体；未知 schema/版本必须拒绝。
3. **模拟隔离**：Phase C 的变体不得写 live model/state/ledger/federation；simulation 命名空间必须显式。
4. **证据可追溯**：Agent 引用、What-if 结果、site bundle 都绑定对象/动作/证据 digest；缺失即拒绝。
5. **默认保守**：Agent 无 dispatch 工具；自动更新/遥测默认关闭；未签名 bundle 明确标记未认证。
6. **可重复**：Catalog、What-if digest、bundle manifest、eval fixtures 都必须确定性；CI 校验重生成/重放。
7. **桌面 crate 保持独立**：`apps/desktop/src-tauri` 继续被根 Cargo workspace 排除，新领域 crate 进入根 workspace；新增依赖需过 `cargo-deny` 与许可策略。

## 设计门禁

本套文档在进入开发前必须通过独立的 **design validation subagent** 校验，输出：

- `docs/design/VALIDATION.zh-CN.md`：最终独立校验报告（逐项判定、REQUIRED/OPTIONAL、Phase A 就绪清单），由 validator 独立给出 verdict；Lead 不要求固定结论。快速预校验见 [`VALIDATION-FAST.zh-CN.md`](VALIDATION-FAST.zh-CN.md)（当前 CHANGES_REQUIRED，R1–R5 已修订）。
- 校验内容：
  - 与现有 PO/SAP/VEC/源码边界的一致性；
  - 阶段顺序与依赖是否可执行；
  - API/数据模型/错误码是否完整、可测试；
  - 安全/威胁模型是否覆盖；
  - 测试/验收标准是否可验证；
  - 是否过度设计或遗漏关键迁移；
  - 逐文档的修改建议。
- 校验不通过时，先修改设计文档并重新校验；**不开始代码实现**。
- 校验通过后，才进入 Phase A 开发；每个阶段的 PR 必须附带设计文档链接和验收证据。
- 已知修订：R1 删除无模型来源的 Catalog 字段/策略元数据；R2 增加 `poie-digest` canonical JSON/SHA-256 规范；R3 预览改为 host dry-run `SapPreview`；R4 增加 A0 宿主鉴权前置；R5 本 README 与校验报告对齐。

## 交叉规范

- [Digest 规范](DIGEST.md)：fingerprint 与 SHA-256、canonical JSON、domain 分离、测试向量。
- [平台契约](PLATFORM.zh-CN.md)：schema 注册、DTO/序列化、错误包络、命令 scope、模型来源、审批原语、workspace/依赖规则。

## 研究依据

三份独立研究备忘（均由子 agent 完成，未修改源码）：

- [Palantir 产品模式研究与 Poieweave 取舍](research/palantir-patterns.zh-CN.md)：Ontology/OSDK/AIP/Apollo + 概念映射、采纳/改造/拒绝项；
- [Agent 治理与评测](research/agent-governance.zh-CN.md)：MCP/OAuth、OWASP、Rule of Two、τ-bench、PROV-O、NIST 等来源与 Phase B 架构；
- [仓库差距审计](research/repo-gap-audit.zh-CN.md)：逐阶段现有代码路径、缺口、依赖方向、最小切片与验收。

公开资料：
- [Palantir Ontology 系统](https://www.palantir.com/docs/foundry/architecture-center/ontology-system/)
- [Palantir AIP 架构](https://www.palantir.com/docs/foundry/architecture-center/aip-architecture/)
- [Palantir 三平台/FDE 架构](https://www.palantir.com/docs/foundry/architecture-center/overview/)
- [Apollo 介绍](https://www.palantir.com/docs/apollo/core/introduction/)

## 产品原则（研究结论）

| 原则 | 含义 | 可测门禁 |
| --- | --- | --- |
| P1 正式契约唯一真源 | Catalog 是带 digest 的投影，不是第二套本体 | 两次生成 digest 相同；未知 schema 拒绝；UI 不能建类型 |
| P2 单一写路径 | 物理/运营变更都是动作，SAP 是唯一执行语义；配置写受审批；D1 交付元数据写需登录+审计，D2 远程/策略变更再加 ApprovalToken/HostPolicy | 领域对象无 CRUD/执行写接口；模型工具中 dispatch 数量为 0；配置写有审批/审计 |
| P3 无环境权限 | 每个工具/动作声明 scope，默认拒绝 | 未知工具/越界参数拒绝并留痕；审批绑定参数摘要 |
| P4 无证据不结论 | 事实性输出必须可追溯到本轮工具结果 | 无 citation 被拒；未知 citation 触发 `citation_missing` |
| P5 模拟不触达现场 | SIMULATION 是显式命名空间 | C 不写 live/ledger；promote 只经 `plan_to_proposed` 产生 proposed |
| P6 交付即版本+证据 | bundle 绑定 model/contracts/evidence | 任一不匹配拒绝导入；回滚可用 |
| P7 断连优先 | 远程能力默认关闭 | 核心流程离线可用；遥测/自动更新默认关闭 |
| P8 可解释不行动 | 显示约束/拒绝原因 | 七项检查逐项原因；“为什么没有更新”可解释 |
| P9 现场反馈变 fixture | 每个现场问题进入预登记/回归 | CI 重放；无证据不宣称修复 |

## 阶段退出条件

| 阶段 | 退出条件（全部满足） |
| --- | --- |
| A | 桌面内消费者仅凭 Catalog/SDK 完成枚举与读取；动作预览不 dispatch；Catalog/SDK 漂移使 CI 失败；六个旧视图不回归 |
| B | 越权工具 100% 被拒并留痕；无引用结论被拒；审批暂停/恢复可测且参数变更失效；离线 eval 在 CI 确定通过；模型工具中 dispatch 数量为 0 |
| C | 同输入同 digest；stale snapshot 拒绝 promote；live 模型/ledger/federation 零写入；比较结果完整可重放 |
| D1 | 版本/证据不匹配拒绝导入；回滚/clear 可测；离线导入可启动；诊断不含密钥；D2 再做通道/签名/安装器 |

## 实施前必须清理的既有漂移

来自仓库审计（`research/repo-gap-audit.zh-CN.md`），这些不是新设计，但会污染 Phase A–D 的验收：

- `apps/desktop/src/App.tsx` 仍写 384 个测试（实际 388）；
- `docs/status/stage-gates.zh-CN.md` 的 per-crate 计数与 T1–T11 旧表；
- `docs/status/status-snapshot.zh-CN.md` 的日期/提交与“#112 三项推迟”旧描述；
- `apps/desktop/src/domain.ts` 把 Base-02/timed 标成 planned，但 CI 已在跑；
- `crates/poie-state/src/lib.rs` 第 31 行说 M12 联邦未实现，但 `poie-federation` 已有实现；
- core crate 版本 0.1.0、desktop 0.1.1、tag v0.1.1 没有单一版本源；Phase D 必须先定义 release manifest；
- tag release workflow 不生成 macOS DMG/便携 Windows exe（当前 Release 中的 DMG 是手动补的），Phase D 必须让产物清单与 workflow 一致。

处理这些漂移不改变 A–D 的安全边界，但必须在对应阶段 PR 中一并修复，不能把旧数字/旧声明带进新页面。

## 修订记录（针对校验）

| 编号 | 修订 |
| --- | --- |
| R1 | 设计文档已提交到 `design/phase-a-d` 分支；Phase A 门禁 PASS（`VALIDATION-A.zh-CN.md`，`907b246`） |
| R2 | `DIGEST.md` 区分 fingerprint 与 SHA-256，增加 approval/evidence/contracts domain 与指纹规则 |
| R3 | `PLATFORM.zh-CN.md` 定义 schema 注册、`schemaVersion`、`OutcomeKind` |
| R4 | 公开 API 使用 DTO，领域类型不加 `Serialize`；`PatchDto` 转换 |
| R5 | 统一 `{code,message,details}` 错误包络与错误码表 |
| R6 | A0/PLATFORM §4 定义命令 scope 矩阵；gate/sandbox/inference 鉴权前置 |
| R7 | 当前模型固定为 Base-01 + degraded 的 `SourceModel`；D 不做模型导入 |
| R8 | 删除通用 `parameters`/`can_submit`；ActionType 仅模型字段；新增只读 `preview_plan_through_sap` |
| R9 | SDK 唯一路径 `poie-sdkgen`；删除外部 JSON API 声明 |
| R10 | B crate 统一 `poie-governance`；六工具与 A 函数映射；无模型 dispatch |
| R11 | `agent_run/status/approve/reject/cancel/resume` + `RunState` 持久化 |
| R12 | per-tool approval digest；append-only 审计 + SQLite 索引；`user_version` 迁移 |
| R13 | C Snapshot 补全 now/seed/horizon/timelines/storage/StateId 可选；`OutcomeKind` 分离 |
| R14 | C Compare/Promotion/Approval schema；`plan_to_proposed`；checkpoint/storage builder |
| R15 | D manifest digest 来源、`contracts.toml`、manifest 自哈希 |
| R16 | D 不做模型/二进制导入；bundle 为 JSON 元数据；policy loader、DB 迁移 |
| R17 | D1 增加 inspect/import/state/rollback/clear/diagnostics；D2 再做 channel/recall/发布产物 |
| R18 | 各阶段实施顺序加入 workspace/Cargo.lock/test-matrix/cargo-deny/ADR/rustdoc/链接检查 |

## 当前状态

- 四份阶段设计 + `DIGEST`/`PLATFORM`/`SCHEMAS`/三份研究备忘：已起草并入库预冻结
- **Phase A 门禁：PASS**（[`VALIDATION-A.zh-CN.md`](VALIDATION-A.zh-CN.md)，快照 `907b246`）
- **A0 已落地**（PR #137）
- **A1 已落地**（PR #139 + #141）：Catalog/SDK/action preview；测试矩阵 396
- **Phase B 设计门禁 PASS**（`VALIDATION-B`，快照 `053f529`）
- **B0 部分落地**（PR #144/#146）：session id、approval/run 表、user_version 迁移、`poie-governance` 工具/审批/审计/评测 schema
- **B0 host runtime/UI 已落地**（PR #150）：六工具调用、RunState/审批 token、只读工具、answer.emit、Agent 面板切到 `agent_run`
- B0 剩余：approval resume/draft 执行、审计日志文件/链、真实 fixtures、cancel/reject UI
- **Phase C1 设计门禁 PASS**（`VALIDATION-C`）
- D1：设计门禁 PASS 后进入实现
