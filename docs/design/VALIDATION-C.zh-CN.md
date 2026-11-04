# Phase C 设计门禁校验（独立）

- 校验对象：[Phase C](phase-c-what-if.zh-CN.md)（下称 C）、[PLATFORM](PLATFORM.zh-CN.md)、[DIGEST](DIGEST.md)、[SCHEMAS](SCHEMAS.md)、[README](README.zh-CN.md)、[Phase B](phase-b-agent-governance.zh-CN.md)；代码 `crates/poie-{ontology,plan,sim,state,viability,digest}`、`apps/desktop/src-tauri`、`apps/desktop/src`、根/桌面 Cargo、CI 与 `scripts/`。
- 快照：`main` HEAD **`69987ca`**；写本文件前 `git status --porcelain` 干净。只读校验，未改源码/设计/git，只写本文件。
- 方法：逐行对照设计契约与现有类型/命令/CI；运行 `check-docs.py`、`check-schema-registry.py`；未跑全量 `cargo test`、Tauri 或 Playwright。
- 结论：**CHANGES_REQUIRED**（4 BLOCKER + 7 REQUIRED-BEFORE-C）。架构方向成立（SAP 唯一执行、静态快照、结构/时序分离、只产 proposed），但不满足“零 C 阻断”。

## 1. 阻断项（BLOCKER，C1 前必须改）

**B1 `whatif_variant_create` 无法创建 `Perturbation`（C:83-89,167）。** 命令输入只有 `snapshotId,label,patch`；`storageDischargeW`/`assetDelayS`/`goal`/`minVerification`/`deadlineS` 才是“延迟修复/储能限功率/截止提前/换目标”四个主场景的载体，却没有创建入口。按现契约只能做 `Patch` 变体（资产状态/库存/质量），C:98 的 timed 结论、C:204 的 `startMode`、验收 #5 的 `power_limited` 都无路径。必须在 create（或显式新命令）接收 `PerturbationDto`，并定义非 timed 模式忽略扰动时是拒绝还是标注。

**B2 Snapshot/证据/结果 digest 无定义，且与 DIGEST 冲突（C:53-66,60-62,121）。**
- C:121 的 `resultSha256` 覆盖 `snapshotSha256`，但 Snapshot（C:53-66）没有该字段，DIGEST 的 domain 表（DIGEST:16-26）也没有 snapshot domain，what-if 只有 `poie.variant/0.1`；谁能对 Snapshot 做 canonical 编码未定义。
- C:60-62 的 `evidenceSha256` 写成 `sha256("poie.evidence/0.1" + canonical_json(...))`：缺少 DIGEST:12-14 强制的 `"\n"`；payload `{scenarioId,scenarioDigest,vecReportSha256,contractsSha256}` 与 DIGEST:47 的 `evidence id + kind + fingerprint + source digest` 是两套语义；`scenarioDigest` 未定义，`contractsSha256` 无来源（仓库无 `contracts.toml`，属 D1）。
- Snapshot 含 `id/createdAt/createdBy/seed`（C:55,63-65），未声明 digest 排除规则；DIGEST:59-61 禁止时间戳/随机 ID 进入 canonical payload，“同输入同输出”因此不可判定。
- `whatif_snapshot_create` 无输入（C:165），C:69 只说“Base-01 + degraded”；`goal/horizonS/startS/deadlineS/minVerification/nowS/maxDepth/maxActions` 无来源或默认值，`seed` 无任何消费者。
- `planSha256` 的 payload 未固定（DIGEST:22 仅写“计划摘要 + actions”，DIGEST:40 只规定把 fingerprint 作为字段再 hash）。须冻结：snapshot 字段/排除项与 domain、evidence 输入、result 绑定范围。

**B3 Promote 审批链不可实现，`plan_to_proposed` 契约仍为空（C:171,182-184,194-195；VALIDATION:70-72,76）。**
- `whatif_variant_promote(variantId,approvalTokenId)` 假定已有 pending token，但 C 无命令铸造/批准/消费；`ApprovalToken` 是 run/call 作用域，只有 `agent_approve/resume/reject/cancel` 会改它且都要求 `RunState`（agent.rs:558-646），而 C:286 说“不要求 Agent”。
- 审批 subject 仍未定义（snapshotSha256/variantResultSha256/planSha256/policyVersion/sessionId/expiry），无法绑定具体结果；C:195 的“重新规划”还可能产出与操作员批准时不同的计划。
- `ProposedActions` 无字段、无 ledger 归属、无 action 状态位置（C:194），与 C:154“不写 SAP 记录/ActionLedger”自相矛盾；这是 VALIDATION:70 已要求“C1 前必须定”的遗留项。
- C:194 把 `simulation=<branchId>;model=<sha>` 放在 `ProposedActions.basis`，而 PLATFORM:134 要求放在 `Authorization.basis`；C 不创建 Authorization，`branchId` 也无来源（未计划使用 `Namespace::Simulation`）。C:195“成功后交给现有 SAP 验证命令”不成立：host 无 `sap_*` 命令（lib.rs:259-289），`validate_plan_through_sap` 又写死自己的 basis（sap.rs:156-164），PLATFORM:136 的可追溯测试无载体。
- `PromotionResult.sapPreview{checks,assumed,rejections}`（C:182）缺少 A/SAP 已有的 `authorizedActions`/`authorizationCreated`（sap.rs:45-52），削弱“未授权”的读回证据。

**B4 timed 变体的基态、`startMode` 与 `assetDelayS` 语义未定（C:55,96-98,111-118,204）。**
- Snapshot 的 `modelSha256` 来自 host `SourceModel`（base-01+pump degraded，lib.rs:236；domain.rs:99-103），timed 却用 `TimedScenario.model`（另一条 `../base-01/model.toml`，timed.rs:238-247）并在起点把 timeline 状态 patch 进去（timed.rs:656-674），且自带 24h `evidence_validity`；快照不记录 timed scenario/case 或第二个 model digest，timed 结果与 promote stale 检查绑定不到实际使用的模型。
- C:96,204 声称结果标注 `startMode`/`nowS`，但 `VariantResult`（C:111-118）没有这两个字段，也没有 `startMode` 枚举；`TimedOutcome` 同样没有（timed.rs:196-208）。
- `assetDelayS`（C:98）把该资产**所有**状态变更 `at += N`，包括故障变更（`scenario.toml` 中 pump 在 6h failed）。延迟修复会同时延迟故障，主场景“泵晚修 2h”语义正好相反；应只移动 availability/repair 变更，或引入独立 repair-completion 字段。
- `AssetTimeline.initial/changes` 私有且无 getter（time.rs:72-76），无 shift API；快照的 `assetTimelines/stockTimelines` 无法构建，`assetDelayS` 无法应用——除非把 ontology 读/移位 API 列入 C1 变更清单。
- `TimedScenario` 只存 `storage_discharge_w`（timed.rs:225-236），capacity/initial 解析后丢弃（timed.rs:307-343），故 `Snapshot.storageLimits.capacitySi/initialSocSi`（C:57）与 C:201 的 `with_storage` 没有来源；`with_storage` 有 `maxChargeW`，Snapshot 却没有该字段。`TimedRunner::run` 需要完整 `CaseDoc`（含必填 `expect`，timed.rs:137-151），C 未定义合成 case；`evidenceValidityS` 与 scenario 的 `evidence_validity` 谁优先也未定。

## 2. C1 前必修（REQUIRED-BEFORE-C，可随 C1 PR 落地，不改变架构）

1. **Compare/结果注册表。** `variant_run` 返回 `runId`，`whatif_compare` 只收 `variantIds`（C:170），结果又只存内存（C:206-208）；同一 variant 的 structure/plan/timed 多次运行无从选择，`baselineResultId`（C:176）与 `delta`（C:177）无类型，`baselineVariantId?` 可空但 `baselineResultId` 非空。须定义 results registry（`AppState` 中有界 map）、以 `resultId` 比较、无结果/跨模式比较的错误码，以及 `sortKey:"verdict"`（C:178）在结构/时序并存时的排序（C:159 又禁止合并）。
2. **schema/错误/DTO wire 完整性。** CompareResult/PromotionResult 在 SCHEMAS 未登记（SCHEMAS:18-20 仅 3 个 whatif schema）；`unknown_snapshot` 不在 PLATFORM 错误表（C:187 vs PLATFORM:48-62）；`schema` 字符串与 `schemaVersion` 的关系未定。`PatchDto` 的 `addAsset/addResource/addLot/addProcess` 及 `AssetStatus/Verification/LotQuality/DegradedPolicy` 在 ontology 全是 Deserialize-only（model.rs:33-59,106-197），而 `Variant` 要返回 patch，必须定义可序列化镜像 DTO，否则无法 derive；PLATFORM:32 的 `PerturbationDto` 与 C 的 `Perturbation` 命名/职责需统一。
3. **隔离与“不写”声明。** C:69 说“C 不写任何 ActionLedger/DB/federation”，与 C:206-208 把 snapshots/variants 写入 SQLite 矛盾；应改成“不写 live/SAP/execution/federation，只写自己的 whatif 表”。C:154“Sim 标签不是安全边界”与 C:158“所有写操作仍被禁止”矛盾：host 没有 mode/`authorize`/`HostPolicy`（lib.rs:259-289），无法按 UI 标签禁写。PLATFORM:88 要求 `whatif_*` 会话+policy+窗口 scope，C:149 只写“需要登录”；`CommandError` 与 `authorize(window,...)` 在 host 均不存在（全项目 `Result<T,String>`）。C 须把该前置列入 C0，或明确依赖 B 收口。
4. **Playwright 策略不可执行。** Playwright 跑 `vite preview`（playwright.config.ts:10-25），`isTauri()` 为 false 时所有 host 调用被跳过（App.tsx:103,1518-1550）；C:261 要在浏览器里“打开 Sim、创建变体、运行、比较”，又要求“浏览器预览禁用”。二选一：提供明确标注 demo 的只读 fallback fixtures，或把 Playwright 限定在禁用态/SIMULATION 徽标，run/compare 放 Rust 集成测试。
5. **SAP preview 的 assumed 语义。** `preview_plan_through_sap` 要求非空 `state_version`（sap.rs:194-199），C 无 live state，却未定义 `AssumedContext` 来源；当前会把 fabricated state 下的 time/preconditions 标成 `observed`。须声明“全部相对快照”，并定义 promote 只在 `failed` checks 非空时拒绝（C:255 只写“SAP 校验不通过时拒绝”）。
6. **plan 模式结果不完整。** `OutcomeKind`（C:103-109）没有 plan 失败值：结构 `candidate` 但 `goals_unmet` 非空（plan.rs:46-60）时只能显示 candidate/completionS=null，易被读成可行；`actions[]`/`rootCauses[]`/`delta` 元素类型也未定义。另需规定 plan/structure 模式遇到 `assetDelayS`/`storageDischargeW` 是拒绝还是显式“已忽略”。
7. **过程门禁缺失。** C:272-279 的实施顺序没有 PLATFORM:185-187 要求的根 `members`、desktop path 依赖+两份 Cargo.lock、test-matrix/status-snapshot、ADR/cargo-deny；`poie-whatif` 未列入 test-matrix 会让 `check-test-matrix.py:80-97` 失败，`desktop-build.yml:7-19` 也没有 `crates/**` 触发路径。PLATFORM:180 的 whatif 依赖表不含 `poie-objects`，但 `EvidenceRef` 在 `poie-objects`（poie-objects/src/lib.rs:75）且 C:132 依赖它；`poie-viability` 在 C 中标“可选”却进 PLATFORM 依赖表。审计事件（C:257“审计事件完整”）无事件清单/字段。DB 迁移也要落到 v3：`Db::migrate` 目前是 `CREATE IF NOT EXISTS` + 无条件 `PRAGMA user_version=2`（db.rs:59-139），C 需要真正的幂等 v2→v3 步骤与回滚说明。

## 3. 可选（OPTIONAL）

- `seed`（C:63）无消费者/默认值；`StateId`（C:58）在无 projection 时恒为 null（projection.rs:61），建议删掉或明确冻结为 null。
- `OutcomeKind.caveat` 是固定散文串（C:108），不是结构化字段；`storageLimits` 缺 `maxChargeW`（C:57）而 `with_storage` 有（C:201）。
- C:214“host 需增加 mode 状态”不必要：mode 是客户端状态；若坚持加，需进 PLATFORM 的 scope 矩阵。
- C:182 的 `sapPreview` 建议直接复用 A 的 `SapPreview`（含 authorizedActions/authorizationCreated）。
- `whatif_snapshot_list/variant_list` 无 limit/分页，与已登记的 `limit_exceeded` 不一致。
- 变更上限“MVP 8 变体/字段数量上限”没有数字（C:94），`budget_exceeded` 不可测。

## 4. 已核对通过（正向）

- **PatchDto 13 字段完整**：`Patch`（model.rs:813-847）与 C:77-81 的 13 个顶层字段一一对应（含 `assetCapabilityVerification/addAsset/addResource/addLot/addProcess`），无遗漏；`Patch` 是 `Deserialize`+`deny_unknown_fields`，转换放在 `poie-whatif` 的方向正确。
- **OutcomeKind 分离正确**：结构 4 值 = `graph::Verdict`（graph.rs:40-46），时序 8 值 = `TimedVerdict`（timed.rs:159-170），未合并进 VEC `VerdictKind`（verdict.rs:16-24）；C:121 的 candidate≠feasible 可测。
- **只读 preview 成立**：`preview_plan_through_sap` 只用局部 ledger、不建 Authorization、返回 `authorized_actions=0/authorization_created=false`（sap.rs:194-281）；`Namespace::Simulation` 与 `ensure_live_dispatch` 存在（record.rs:83-104、isolation.rs:22-31），host 也没有 live ActionLedger，故“C 不写 live ledger”可实现；C:69 删除旧的“模拟前后 live 不变”假阳性是正确的。
- **fingerprint/sha256 命名合规**：`planFingerprint`/`planSha256` 分开（C:114,121），符合 DIGEST:29-40。
- **依赖方向无环**：sim→ontology、plan→state/ontology、whatif→全部；desktop 继续 exclude（根 Cargo.toml:3-30），加 path 依赖即可。DB `PRAGMA user_version = 2`（db.rs:122）可加 v3。
- **UI 方向正确**：三列对比 + `SIMULATION · <snapshot>` + 颜色/文字双编码（C:221-232）；`Mode`/`MODE_ENABLED` 已存在（App.tsx:53,71），启用 sim 只需改开关（但见 §2.3/2.4）。

## 5. 已执行检查与边界

```text
python3 scripts/check-docs.py              OK, exit 0
python3 scripts/check-schema-registry.py   OK (24 schemas), exit 0
git status --porcelain                     clean before this file
```

未跑：全量 `cargo test`、Tauri 构建、Playwright；`poie-whatif` 尚不存在，本报告是设计门禁而非代码验收。已一并核对 A/B 依赖：B1 eval fixtures 未落地（`evals/` 是空目录），但 C 不依赖它；C 依赖的 B 审批原语已实现，但其 run 作用域正是 B3 的问题来源。

## 6. 判定

方向可保留；B1–B4 是契约/可实施性阻断，§2 是 C1 开工前必须写进 C 文档（或 C0）的收口项。修完 B1–B4 并重跑本门禁后再进入 `poie-whatif` 实现。

VERDICT: CHANGES_REQUIRED
