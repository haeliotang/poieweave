# Phase A 设计门禁校验（A0 + A1）

- 校验对象：[A](phase-a-ontology-product.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[README](README.zh-CN.md)；代码 `crates/poie-ontology`、`crates/poie-plan`、`apps/desktop/src-tauri`、`apps/desktop/src`、根 `Cargo.toml`、`scripts/`。
- 快照：分支 `design/phase-a-d`，设计快照 **`0527c7f`**；校验期间出现的 `2510867` 只改 README/VALIDATION-A，不改变本判定。工作树中已有他人写入的 PASS/CHANGES 稿，本报告独立复核，不采纳其结论。
- 方法：只读对照 A/PLATFORM/SCHEMAS/DIGEST/README、`model.rs`/`plan.rs`/`sap.rs`/`lib.rs`/`domain.rs`/`execution.rs`、前端 demo；`check-docs.py` 与 `check-test-matrix.py` 均 exit 0；未跑全量 `cargo test`；未改源码/设计/git，仅写本文件。

## 0. 判定

**CHANGES_REQUIRED。** A0 不再伪造授权、单一 SAP 路径、digest/schema/DTO 收束都已成立；仍有 **2 组 A0 缺口**与 **2 组 A1 契约矛盾**，会让首版产生路径逃逸、错误“已授权”展示或第二套字段。B/C/D 门禁不参与本判定。

## 1. 五个最强点

1. **A0 不生成 Authorization。** A:43-46 明确 execution/federation 的硬编码授权移除、`dispatch_allowed=false`、`authorized/dispatched=0`，并点名现有测试断言需改。
2. **单一 SAP 执行语义。** `preview_plan_through_sap` 只读、不写 ledger/Authorization/transition（A:271-288）；Catalog 只是 Model 投影；C 的 `plan_to_proposed` 只产生 proposed。
3. **Digest/Schema 可复现。** fingerprint 与 SHA-256 分离、domain/canonical 规则、`-0.0`/有限值、固定向量（DIGEST:20-40、58、96-102）；SCHEMAS 是唯一注册表，PLATFORM 不再复制表。
4. **DTO 主体已补齐。** `ObjectPage/ObjectDetail/ActionPreview` 带 model/catalog digest；`EvidenceRef` 有 `asOf/source`；`ObjectInstance.version=model_digest`；`LinkType.cardinality` 固定；`ActionType.duration_s` 直取 `Process.duration_s`（A:186-216、234-263；model.rs:311）。
5. **交付路径可执行。** `poie-sdkgen --check` 零 diff + 黄金文件；A0 是第 0 步；ADR/schema checker/limit/根与桌面 `Cargo.lock`/test-matrix/cargo-deny 已写入 A1 首 PR（A:395-396）。

## 2. A0 阻塞

### B1 `sandbox_read` 仍可被根内符号链接带出根外（A0，阻塞）

- A:41 只把 **`sandbox_write`** 改为 no-follow；[PLATFORM](PLATFORM.zh-CN.md):90 要求 `sandbox_*` 都 no-follow + 原子写。`sandbox_read` 仍用 `std::fs::metadata` + `read_to_string`（[sandbox.rs](../../apps/desktop/src-tauri/src/sandbox.rs):26-35），会跟随根内符号链接；`resolve` 只拒绝 `..`/绝对路径，不拒绝 symlink。`root/link -> /etc/passwd` 仍可读，A0 的“路径限制”不成立。
- **最小修复**：A:41 改为 `sandbox_read/write` 均 no-follow（read 用 `symlink_metadata` 或 `O_NOFOLLOW`），加“根内 symlink 指向根外”和“写失败不留半文件”负例。
- 同组：`sandbox_root` 命令仍在（[lib.rs](../../apps/desktop/src-tauri/src/lib.rs):224、274），PLATFORM:79 只在括号里说“由 host_info 返回”，矩阵无该命令、无登录检查。**最小修复**：A0 删除独立命令并让前端只读 `host_info.sandboxRoot`（App.tsx:1393 已用后者），或在矩阵补 `sandbox_root | 是 | main | 读 | 会话`。

### B2 浏览器 fallback 仍显示“已授权/复核通过”（A0，阻塞）

- A:43/A:338 只点名 `execution.rs`/`federation_view.rs`；但浏览器预览的 [domain.ts](../../apps/desktop/src/domain.ts):237-256 仍是 `state:"ready"`、action `state:"authorized"`、`checks` 全部 passed、`reviewStatus:"ready"`、`dispatchAllowed:true`。App.tsx:1422 使用该 fallback，786-790/820/1054 显示 `authorized`/`reviewReady`；[i18n.ts](../../apps/desktop/src/i18n.ts):78-81 仍定义“复核通过”“已通过 validate/authorize”。
- A:331 明确 Playwright 走浏览器预览；按 A:46 “UI 不再显示复核通过”这条路径不成立。**最小修复（二选一）**：把 `domain.ts`/`App.tsx`/`i18n.ts` 纳入 A:43/§8.2，fallback 改 `validated`/`synthetic-preview`、`reviewStatus:"blocked"`、`dispatchAllowed:false`、authorization check 失败并加 Playwright 断言；或明确 A0 验收仅限 host，并同步删除 A:43/46/338 的全局表述。
- 关联测试：[execution.rs](../../apps/desktop/src-tauri/src/execution.rs):181-197 仍断言 `review_status=="ready"`、0 rejections、全部 authorized、`dispatch_allowed`。**最小修复**：明确改为 `review_status=="blocked"`、`rejections==actions.len()`，否则 UI 的 `reviewReady` 仍可能为 true。

## 3. A1 阻塞

### B3 ActionType 仍有非模型字段、ID 与层次矛盾（A1，阻塞）

- **字段表与结构体不一致。** A:103 说“每个动作类型只包含模型可导出字段”，A:106 仍列 `id, label, description, target_types`；A:211-217 的结构体没有 `label/description`，A:212 还写“不伪造 label/description 字段”。已验证 `Process` 没有 description（[model.rs](../../crates/poie-ontology/src/model.rs):304-315；`ProcessDoc.description` 在 197-200 但未验证）。旧 R1/RA1 只从结构体删了字段，表格没删。
- **ID 格式矛盾。** A:100 定义 `process.<process-id>`（例 `process.replace-sleeve-p1`）；A:212 却说 `id` “即 process id”（暗示无前缀）。SDK 字符串字面量 ID 会二选一错。
- **`sap.*` 是否进 Catalog 未定。** A:100-101 说动作类型分两层并含 `sap.*`；A:165 说动作目录“仅从 `Model.processes` → `Vec<ActionType>`”；`OntologyCatalog` 只有 `action_types: Vec<ActionType>`，没有 `sap.*` 的容器。若 `sap.*` 不进 Catalog，§6.3 的“两层/只读目录”落空；若进，A:165 的“仅从”不成立。
- **最小修复**：A:106 改为 `id, target_types, inputs[], outputs[], occupies[], restores[], requires_capabilities[], duration_s?, required_verification`；A:112/372 补 `requires_capabilities`；明确 `id="process.<Process.id>"`（或同步 A:100）；为 `sap.*` 定义独立静态列表/字段或明确不属于 Catalog；加“ActionType 逐字段来自已验证 Model”的黄金 fixture。

### B4 属性 `source` 的规则与值域冲突（A1，阻塞）

- 极性本身已修正：A:76 的 `Some → evidence`、`None → declared` 与 [model.rs](../../crates/poie-ontology/src/model.rs):270-272、708-716 一致。但 A:76 首句把“没有证据时间的属性”一律标 `source=declared`，会把模型直接导出的属性（unit、duration 等）误标为声明。
- A:76 使用 `source=evidence`，A:179 的 `PropertyDef.source` 注释却只允许 `model|declared|observed`；A:183 的 `PropertyValue.source` 没有值域。`evidence` 未登记，SDK/UI 取值会不一致。
- **最小修复**：把 A:76 首句改为“没有证据来源的属性标 `model`；没有证据时间的属性 `asOf=null`”，或在 A:179/183 登记 `evidence` 并定义；补三分支 fixture：模型导出→`model`、`status_evidence=Some`→`observed`/`evidence`、`None`→`declared`。

## 4. A2 前必须关闭 / 过程残项（不单独升级为 A0/A1 阻塞，但当前不能算 PASS）

- **O1 `ActionPreview` 归属仍与“no objects→plan”冲突。** A:258-266 的 `ActionPreview` 在 §7.1 `poie-objects` 代码块中，A:263 用 `Option<plan::SapPreview>`；A:271 又说 `poie-objects` 不依赖 plan，PLATFORM:163 的依赖表也没有 plan。**最小修复**：明确 `ActionPreview/PlannedActionView` 是 host `objects.rs` DTO（或给 `poie-objects` 一个不依赖 plan 的中性 preview DTO），并把 A:297 的 `plan object 或 planId, actionId` 与 A:125 Plan-only、`preview_plan_through_sap(&Plan)` 的 plan 级返回对齐。
- **O2 过程门禁文字不一致。** [SCHEMAS](SCHEMAS.md):4/45 写 `check-schema-registry.py` “A0 新增并加入 CI”，A:396 写在 A1 首 PR；旧 B6 要求二选一。建议统一为 A1 首 PR 并同步 SCHEMAS。A:396 的 limit 只有上限（对象 ≤500/page、catalog ≤1 MiB），`limit?` 默认值与 `next_cursor` 编码仍未定义。
- **O3 `EvidenceRef` 的“至少一个 digest”未写。** PLATFORM:121 要求 fingerprint/sha256 至少一个；A:191-195 两者都是 `Option` 且无约束。A1 SDK 类型会允许空引用；建议加构造函数/校验或文档约束。
- **O4 其余 A2 细化**：`ObjectDetail.evidence` 的 MVP 填充规则、七项检查逐项状态表、capability 图列、`poie-view-*` 完整标签列表（PLATFORM:68 只列 3 个，代码实际允许 6 个）可在 A2 收尾。
- **O5 README 门禁记录冲突。** [README](README.zh-CN.md):103/125 已记录 Phase A PASS；本报告为 CHANGES_REQUIRED。修完 B1–B4 后重跑并以新报告同步 README，不能保留“PASS”记录把阻塞带进实现。

## 5. 就绪清单

- [x] A0 是第 0 步；gate 先 `auth::require`；不生成 Authorization；`authorized/dispatched=0`、`dispatch_allowed=false`（A:40-46、395）。
- [x] 单一 `SourceModel`；Catalog 只从 Model 生成；预览不写 ledger/Authorization/transition（PLATFORM:98-104；A:38、271-288）。
- [x] Digest/canonical/固定向量；model/catalog digest 覆盖 catalog/page/detail/action（DIGEST；A:205-263）。
- [x] Schema 单注册表、StateId DTO、`preconditions: Vec<String>`、PreviewError 映射（PLATFORM:15、30；A:191-254）。
- [ ] **B1** `sandbox_read` no-follow、`sandbox_root` 入矩阵或删除、no-follow 负例。
- [ ] **B2** `domain.ts`/`App.tsx`/`i18n.ts` 的 demo 授权与“复核通过”清理（或明确 A0 只限 host）；`review_status=="blocked"`/rejection 期望。
- [ ] **B3** ActionType 字段表、ID 前缀、`sap.*` 归属与 `requires_capabilities`。
- [ ] **B4** `source=evidence` 与允许值/三分支 fixture 对齐。
- [ ] **O1–O3** `ActionPreview` 归属；SCHEMAS 的 A0/A1；limit 默认/cursor；EvidenceRef 至少一个。

**结论：当前不能开工。** 先修 B1（路径逃逸）与 B2（UI 仍显示已授权），再修 B3/B4（A1 字段与 provenance）；O1–O3 在 A2 前关闭。B/C/D 的后续门禁不在本报告阻断范围。

VERDICT: CHANGES_REQUIRED
