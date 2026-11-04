# Poieweave Phase A–D 设计校验（快速版）

- 日期：2026-11-04
- 依据：`docs/design/{README,phase-a..d}`、`research/repo-gap-audit`、指定源码文件（只读；未改源码或 git 状态）
- 方法：逐条对照现有 PO/SAP 类型、宿主命令面与 digest 实现；未运行完整测试套件

## 总体判定

**CHANGES_REQUIRED**（4 处设计/顺序修改，1 处门禁产物名）。

方向正确：A 的只读投影是低风险起点，B/C/D 都围绕"SAP 唯一执行语义 + 证据绑定"收束。阻塞点不在愿景，而在两类会立刻退化成"第二套本体/第二套执行语义"的细节：**目录字段没有模型来源**，以及 **digest 没有可复现定义**。

## 5 个最强点

1. **单一执行点守得住**：A/B/C 只产 proposed/草稿，`dispatch` 仍只在 `poie-state::review` + `poie-exec`；B 的工具目录里没有任何 dispatch（B §5.2）。
2. **A 定位克制且可验收**：只读 Catalog + 预览，明确"不复制七项复核"，并要求 SDK 重生成零 diff（A §6.3/§10）。
3. **声明/观测分离**：`Asset.status_evidence: Option`（`model.rs:266-279`）与 A §6.1 的 `source/asOf/verification` 对齐 PO §3.1，避免把合成状态当事实展示。
4. **C 复用既有隔离原语**：`Namespace::Simulation`（`record.rs:83-104`）+ `ensure_live_dispatch`（`isolation.rs:22-31`）已足以支撑"变体永不派发"的可测断言，无需新标志位。
5. **D 边界诚实**：manifest 只做 SHA-256 完整性并显式"未认证"，不做 updater/遥测，且保留"为什么没有更新"的可解释性（D §3/§5.1）。

## 必须修改（阻塞 Phase A）

**R1 Catalog 字段缺模型来源（A §5.1 / §5.3）。** `Lot` 只有 `id/resource/si/quality`（`model.rs:290-295`），设计却给出 `location`、`evidence`；`Process` 没有 `risk_class/approval_class/allowed_in_modes/evidence_required/executor_kind`，也没有 `parameters` 结构（`model.rs:304-316`）。这些只能靠 `poie-objects` 内置映射表，正是 P1 与 README 不变式 2 禁止的第二套本体。必须二选一：给出确定性派生表（并说明是否升 PO 版本），或从 MVP 目录删除并同步删掉 §10"参数来自模型"的验收项。

**R2 `model_digest`/`catalog_digest` 无可复现定义。** `ModelDoc` 只有 `Deserialize`（`model.rs:87-104`），`Model` 持有 `&'static Unit`（`model.rs:237-243`），领域 crate 无 sha2；既有 `Plan::digest`/`Simulation::digest` 是格式化字符串而非哈希（`plan.rs:64-76`、`simulation.rs:315`+）。A §6.1 的 `model_digest`、C 的结果 digest、D 的 `manifest_sha256` 全押在它上面。A1 前须冻结：算法、canonical 编码（建议对 canonical JSON 取 SHA-256）、与既有 digest 字符串的组合关系。

**R3 预览路径越界（A §6.1:135 vs 审计 §A.3:60-63/§C.3:178）。** A 允许 `poie-objects` 调 `poie-plan::validate_plan_through_sap`，但审计要求 plan 对 objects 保持独立；更实际的是该函数会伪造 `Authorization{by:"planning-validation"}` 并把动作推进到 `authorized`（`sap.rs:77-82`、`120-140`），与 A §7.2"移除演示性授权"直接冲突。应改为 host 调用并把 `SapValidation`/拒绝项作为入参传给 catalog，或新增只读 `review_preview`，并把 ownership/reservations/authorization 标为**假定**而非"已通过"。

**R4 宿主命令面在 A 阶段就要定界。** `execution_gate_set` 先改 gate 再"可选"审计（`lib.rs:184-206`，196 行 `if let Ok`），`sandbox_read/write` 完全无 auth（`lib.rs:213-226`）。A §9:238 却说"写路径仍由 `execution_gate_set` 控制"，这是错误陈述。要么把 gate 鉴权提为 A0 前置（B §9:275 亦要求），要么改写 A §9 并说明 A 只读命令为何免会话、以及按命令 scope 的落点。

**R5 门禁产物名不一致。** README §"设计门禁"（28-39）要求 `VALIDATION.zh-CN.md` 且"校验通过才开发"，README:92-96 仍写"最终 validator 运行中"。本文件是快速校验，须由 README 引用（或另出正式 VALIDATION）后才能宣告通过。另：`docs/design/` 整体仍未纳入 git（HEAD `bee82b7`，`git status` 显示未跟踪），门禁结论需随文档一并入库才可追溯。

## 5 项断言核对

| # | 断言 | 判定 | 依据 |
| --- | --- | --- | --- |
| 1 | `poie-objects`+`poie-sdkgen` 可加而不反转依赖 | **确认（前提 R3）** | ontology 仅依赖 serde/toml，state→ontology，plan→state；新增 objects→ontology/state 无环，desktop 仍 `exclude` |
| 2 | B 六工具/无 dispatch 与 SAP 一致 | **确认（需删残留）** | B §5.2 六工具全为 read/propose/simulate，不写 transition；但 B §9:277 与审计 §B.3:115 仍写 `dispatch.request` 为 agent 产物，与 §5.2 及 README 不变式 5 冲突，须删 |
| 3 | `execution_gate_set` 缺 auth，B 前必须修 | **确认** | `lib.rs:185-205` 先写 gate/snapshot，196 才尝试 `require`；未登录同样生效且无审计 |
| 4 | C 的 Patch/Perturbation 切分符合 Patch 能力 | **确认（覆盖不全）** | Patch 13 个字段，envelope 取 11 个；storage/goal/deadline 确实不在 Patch 而在 `timed.rs:225-235`/`graph.rs:32-37`。但 §4"泵延迟 2h"无处可放（Patch 无时间线、Perturbation 无 `assetTimelines`），"移除备用泵"也无 `remove_asset`；须补字段或改场景措辞 |
| 5 | D manifest 可绑定版本/证据而无需 updater | **确认** | §3 明确不做 updater；§5.1 同时记录 desktop 与 core 版本、contracts revision、scenario/evidence digest、artifact SHA-256。缺的只是 `HostInfo` 的 releaseId/channel/manifestDigest（`lib.rs:46-54`）与 build.rs 嵌入，属实现而非设计 |

## Phase A 就绪清单

- [ ] R1–R4 落入 A 文档；R5 由 README 引用本报告
- [ ] 冻结 digest 规范（算法/canonical 编码/model↔catalog↔plan 组合），带"两次生成相等"用例
- [ ] ActionType 派生表定稿；删除或显式声明 `lot.location`/`lot.evidence`
- [ ] 预览只经 host 传入 `SapValidation`，假定上下文单独渲染，不得显示"7/7 通过"
- [ ] 新 crate 进根 `members`、desktop 保持 `exclude`、`cargo-deny` 通过；生成物提交且重生成零 diff
- [ ] 错误码 `unknown_type`/`unknown_object`/`unsupported_schema`/`invalid_parameter` 有 Rust 枚举与 JSON 快照用例
- [ ] 六个旧视图（overview/capabilities/plan/execution/federation/evidence）Playwright 不回归
- [ ] 先修 `execution_gate_set` 鉴权，或删除 A §9"写路径由 gate 控制"的表述

VERDICT: CHANGES_REQUIRED
