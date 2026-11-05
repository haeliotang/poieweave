# Phase C2a 设计门禁（独立复验）

- 校验对象：[phase-c2-timed-what-if](phase-c2-timed-what-if.zh-CN.md)（下称 C2a）、C1、`PLATFORM`/`DIGEST`/`SCHEMAS`；代码 `poie-ontology::time`、`poie-sim::timed`、`poie-whatif`、desktop host/DB/UI。
- 快照：branch `design/phase-c2a`，HEAD **`e9271f3`**；写本文件前工作树 clean。只读校验，未改源码/设计/git，只写本文件。
- 方法：逐条核对 C2a §4–§11；对照 `time.rs`/`timed.rs`/`whatif` lib 与 host、`sandbox.rs`、`db.rs`、`scenarios/`；运行 docs/schema 检查与 whatif/timed lib 测试。
- 结论：**CHANGES_REQUIRED（5 个 C2a 阻断）**。上轮 B1 的 `StorageLimit` 双权威与 B3 的 `assetDelay` 已在设计层消除；B2/B4/B5 仍有关键缺口，尚不能开工。

## 已核对（非阻断）

- **B1 Storage**：设计不再引入 `StorageLimit`；真实 `scenario.toml` 的 `[[storage]]` 只投影为 `storage_discharge_w`（timed.rs:225–235），`power_limited` 由该值触发。容量/初始量的权威表述仍有问题，见 R1。
- **B3 assetDelay**：C2a 无 shift API、无 assetDelay 字段/验收；checkpoint/promote 也在 C2b，范围声明诚实。
- **B2 场景来源**：`scenarios/base-01-timed/scenario.toml` 存在，schema 常量为 `poie.timed-scenario/0.2`；C2a 明确不序列化 `TimedScenario`，run 重载路径，方向正确。
- **DB/范围**：`PRAGMA user_version=5`、4→5 条件 ALTER、静态行回填、无 promote 均写入；`whatif_results` 的 `mode/settings_json` 确实已存在（db.rs:130–134）。
- 检查：`check-docs.py` OK；`check-schema-registry.py` OK（34 schemas）；`cargo test -p poie-whatif --lib` 5/5；`cargo test -p poie-sim --lib timed` 6/6。

## 阻断（C2a BLOCKER）

1. **BLOCKER D1 — scenario/result digest 仍不闭合。** C2a:48 定义 `sha256("poie.timed-scenario/0.2\n" + raw_file_bytes)`，但 DIGEST:10–13 统一规定输入为 `domain + "\n" + canonical_json(payload)`；DIGEST:25–27 的 0.2 行没有 payload、排除项或固定向量，SCHEMAS:22–28 还把 digest domain 登记成 wire schema。C2a:104 的 result payload 又漏掉 variant 的 `patch/goal/minVerification/deadlineS`（C1 whatif:543 明确纳入），不同变体输入可产生同一 `resultSha`。修法：在 DIGEST 增补 raw-file digest profile（固定算法/命名/向量）或改回 canonical DTO；result payload 补 variant 输入；登记 0.2 固定 fixture。

2. **BLOCKER D2 — 扩展 C1 类型会破坏 C1 0.1 持久化与 digest，且 static 0.2 无结果路径。** C2a:102 直接扩展 `OutcomeKind`；`outcome` 进入 `poie.variant/0.1` payload（whatif:262–266,540–548），新增 `timed:null` 会改变所有 C1 结果 Sha。Snapshot/Variant 新增 required `kind/startMode/storageDischargeW` 后，旧 JSON 行经 `whatif.rs:65–70,178–181` 反序列化会失败；设计只回填 DB 列，未回填 `snapshot_json/variant_json`/未定义 serde default。C2a:60 允许 `kind:"static"`，但 C2a:92 只有 `mode:"timed"` 的 Result 0.2，static 0.2 无对应结果路径。修法：为 C2a 定义独立 0.2 outcome/result DTO，或明确 0.1 serializer 排除 `timed`；给旧行默认值/迁移；补 static 0.2 路径或禁止。

3. **BLOCKER D3 — `timedRange` 未定义且现有 API 不可计算。** C2a:47/111 要求 import 返回 `timedRange`，C2a:85 用 `[0, timedRange]` 校验 `startS`；但 `TimedScenario` 无 range 字段（timed.rs:225–235），`AssetTimeline`/`StockTimeline` 的 changes 私有（time.rs:72–77,181–185），公开 API 只有 `status_at/available_at`。无法确定是最大变更时刻、`evidence_validity` 还是 `horizonS`，也无法在 host 实现。修法：定义 `timedRange` 语义并加只读 accessor（或删除并改用 snapshot.horizonS），给固定场景向量。

4. **BLOCKER D4 — 场景/模型路径未真正限制在 sandbox，run 也未重验 model digest。** C2a:47 只校验 import 的 `path`；`TimedScenario::load` 用 `path.parent().join(doc.model)`（timed.rs:238–248），`model` 可为 `../../…` 或绝对路径；import/run 若直接调 `load` 会绕过 sandbox 的 no-follow/大小限制（sandbox.rs:17–42）。C2a:48 的 run 只校验 scenario digest，未要求再次校验 `scenario.model` 与 `snapshot.modelSha256`，模型文件变更可静默改变结论。另 goal 5 说“不复制 Model”，但 `load` 需要模型文件；真实 repo 场景/模型如何进入 app sandbox 未说明。修法：host 用 sandbox resolver 读 scenario/model，拒绝 model 绝对/`..`/symlink，run 重验 model digest；明确真实 fixture 的 sandbox 落点。

5. **BLOCKER D5 — timed `PatchDto` 的 assetStatus 会被 timeline 覆盖，compare 过滤语义与 C1 冲突。** C2a:86 说 PatchDto 先应用；但 runner 的 `patched_model_at` 对每个资产用 timeline status 覆盖 `patch.asset_status`（timed.rs:656–674），对有 timeline 的资产（如 pump-p1）`assetStatus` 变体静默 no-op；`evaluate` 也不接收 patch。C2a:103 要求 compare 先按 `result.mode` 过滤，但 C1:181 明确“不同 mode 可并列”，且 `whatif_compare` 没有 mode 参数（whatif.rs:185–198），无法决定过滤哪个 mode。修法：定义 patch 与 timeline 的优先级（建议 patch 后应用或拒绝 timed 资产的 assetStatus patch），加 no-op 回归；定义 compare 是 family filter 还是新增 mode 参数/CompareResult 0.2。

## 动工前（REQUIRED-BEFORE）

- **R1（REQUIRED-BEFORE）storage 权威表述。** C2a:17/38/139 说容量/初始 SoC 以 `StockTimeline` 为准，但 `StockTimeline` 只有 `initial_si` 与 changes（time.rs:181–185），没有 capacity；`StorageDoc.capacity/initial`（timed.rs:115–123）仍被解析、校验后丢弃。若 C2a 不需要 capacity，明确“capacity 不参与 C2a”；否则补 `StockTimeline` capacity 并定义不变量/不重复来源。
- **R2（REQUIRED-BEFORE）错误/命令契约。** `scenario_mismatch`/`stale_scenario` 未登记 PLATFORM:49–68，C2a:118 也缺 `scope_denied`；`whatif_scenario_list` 输出未定义；`whatif_scenarios` 以 digest 为 PK，同一内容不同 path 的导入/覆盖语义未定。补契约与测试。
- **R3（REQUIRED-BEFORE）测试缺口。** `base-01-timed/cases.toml` 没有 `power_limited`/`capacity_conflict` 预注册用例；新增 fixture 会改 `digests.toml`。补 scenario/result 固定 digest 向量、4→5 旧 JSON 迁移测试、patch-vs-timeline no-op 测试、跨 mode compare 测试。

## 边界

只读设计/源码校验；除本文件外未改任何文件。未运行 Tauri/Playwright，未验证实际 UI 或真实 app sandbox 路径。C2b 延后声明通过。

VERDICT: CHANGES_REQUIRED
