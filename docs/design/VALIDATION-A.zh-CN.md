# Phase A 设计门禁校验（A0 + A1）

- 校验对象：[A](phase-a-ontology-product.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[README](README.zh-CN.md)；代码 `crates/poie-ontology`、`crates/poie-plan`、`apps/desktop/src-tauri`、`apps/desktop/src`、`scripts/`。
- 快照：分支 `design/phase-a-d`，HEAD **`b7d0eb9`**；工作树干净。本报告只对 `b7d0eb9` 负责。
- 方法：只读对照 A/PLATFORM/SCHEMAS/DIGEST/README 与 `model.rs`/`plan.rs`/`sap.rs`/`lib.rs`/`sandbox.rs`/`execution.rs`/`domain.ts`/`App.tsx`/`i18n.ts`；`check-docs.py` 与 `check-test-matrix.py` 均 exit 0；未跑全量 `cargo test`；未改源码/设计/git，仅写本文件。

## 0. 判定

**CHANGES_REQUIRED。** `b7d0eb9` 已实质闭合 B1/B2/B3 与 B4 的主体：A0 的 read+write no-follow、根内 symlink 逃逸负例、删除 `sandbox_root`、浏览器 fallback 与测试断言均已写入；A1 的 ActionType 只保留 process id 与模型字段。仍有一个 A1 字段契约缺口（`PropertyDef.source` 值域）和一个过程文字缺口（schema checker 时点），均可定点修复。B/C/D 门禁不参与本判定。

## 1. 五个最强点

1. **A0 路径闭合。** A:41 要求 `sandbox_read/write` 都 no-follow（`symlink_metadata`/`O_NOFOLLOW`，拒绝任何 symlink），write 再用临时文件/原子 rename，并加“根内符号链接指向根外”负例；PLATFORM:79 同步，矩阵不再列独立 `sandbox_root`，前端已读 `host_info.sandboxRoot`（App.tsx:1393）。
2. **A0 UI/测试闭合。** A:43 点名 `domain.ts`/`App.tsx`/`i18n.ts`，fallback 改 `synthetic-preview`/`reviewStatus:"blocked"`/`dispatchAllowed:false`/checks 不全 passed；`execution.rs` 旧断言须改 `review_status=="blocked"`、`rejections==actions.len()`、`authorized/dispatched=0`。
3. **A1 ActionType 闭合。** A:98-109 只列 process id、target_types、inputs/outputs、occupies/restores、requiresCapabilities、durationS、requiredVerification；无 `label/description`、无 `sap.*`、无 `process.<id>` 前缀。A:211-217 与 `Process` 字段（model.rs:304-311）一致，`requires_capabilities` 已进结构体。
4. **B4 主体闭合。** A:76 把 `source` 枚举定为 `model|declared|observed|evidence`，模型导出字段 `source=model`；`status_evidence=Some -> evidence`、`None -> declared` 与 model.rs:270-272/708-716 一致；`PropertyValue.source` 已含 `evidence`（A:181）。
5. **DTO/归属/限制闭合。** A:274 明确 `ActionPreview/PlannedActionView` 是 host `objects.rs` DTO、`SapPreview` 归 `poie-plan`；`EvidenceRef` 注明 fingerprint/sha256 至少一个（A:192；PLATFORM:121）；A:396 定义 limit 默认 100/最大 500/page、catalog ≤1 MiB、opaque cursor；O1/O3 已闭。

## 2. 剩余阻塞

### R1 A1 `PropertyDef.source` 仍只有三值（阻塞 A1）

- A:76 说 `source` 枚举固定为四值，A:181 的 `PropertyValue.source` 也已是四值；但 A:177 的 `PropertyDef.source` 注释仍只允许 `model|declared|observed`，没有 `evidence`。文档没有说明 `PropertyDef` 与 `PropertyValue` 使用不同枚举；按 A:76，`Asset.status` 的 property def 无法表达证据来源，SDK 生成器可能产出两个不一致的 source 联合。
- **最小修复**：把 A:177 改为四值，或明确 `PropertyDef.source` 是独立三值枚举并说明与值级 source 的关系；补三分支 fixture（模型导出→`model`、`Some`→`evidence`、`None`→`declared`）。

### R2 schema checker 时点三处不一致（A1 首 PR 验收定义）

- A:396 写 `scripts/check-schema-registry.py` 在 **A1 首 PR**；SCHEMAS:3 说“Phase A1 前”、SCHEMAS:4/45 说“A0 新增”；PLATFORM:9 也说“A0 将新增”。A:395 的 A0 步骤没有该 checker。
- **最小修复**：选一个时点（建议 A1 首 PR，与 A:396 一致），同步 SCHEMAS/PLATFORM；若选 A0，则把 checker 加入 A0 步骤与验收。`limit` 已定义，不需再改。

### R3 门禁记录（过程，不另计为设计阻塞）

- README:103/125 仍写 Phase A PASS（快照 `0527c7f`）；本报告为 CHANGES_REQUIRED。修完 R1/R2 后由 Lead 用新报告同步 README，不能把旧 PASS 带进实现。

## 3. 就绪清单

- [x] A0 是第 0 步；`execution_gate_set` 先鉴权；不生成 Authorization；`dispatch_allowed=false`、`authorized/dispatched=0`（A:40-46、395）。
- [x] `sandbox_read/write` no-follow + symlink 负例；`sandbox_root` 删除并统一 `host_info.sandboxRoot`（A:41；PLATFORM:79）。
- [x] 浏览器 fallback 与 `execution.rs` 测试断言改写（A:43）。
- [x] ActionType 仅 process id/模型字段；无 `label/description/sap.*`；`requires_capabilities`（A:98-109、211-217）。
- [x] `ActionPreview/PlannedActionView` host-owned；`SapPreview` `poie-plan`；`EvidenceRef` 至少一个 digest；limit/cursor 已定义（A:192、274、396）。
- [ ] **R1** `PropertyDef.source` 四值或明确独立枚举 + 三分支 fixture。
- [ ] **R2** checker 时点在 A/SCHEMAS/PLATFORM 统一。
- [ ] **R3** README 门禁记录同步。

现有实现仍是旧状态（[sandbox.rs](../../apps/desktop/src-tauri/src/sandbox.rs):26-35 仍 `metadata`+`read_to_string`；[lib.rs](../../apps/desktop/src-tauri/src/lib.rs):224/274 仍注册 `sandbox_root`；[execution.rs](../../apps/desktop/src-tauri/src/execution.rs):181-197 仍断言 ready/authorized；[domain.ts](../../apps/desktop/src/domain.ts):237-256、[i18n.ts](../../apps/desktop/src/i18n.ts):78-81 仍显示授权/复核通过）。这些是 A0 的实现对象，不是本设计门禁的未闭合项；A0 实施时必须逐项改掉。

**结论：A0 的设计前置已闭合；A1 仍有一个字段契约和一个过程时点未闭合，当前不能宣称 A0+A1 零阻塞。** B/C/D 的后续门禁不在本报告阻断范围。

VERDICT: CHANGES_REQUIRED
