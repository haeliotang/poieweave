# Phase C2a 设计门禁（独立复验）

- 校验对象：[phase-c2-timed-what-if](phase-c2-timed-what-if.zh-CN.md)（C2a）、[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)/[PLATFORM](PLATFORM.zh-CN.md)、C1；代码 `poie-sim::timed`、`poie-ontology::time`、host `sandbox.rs`/`db.rs`/`domain.rs`。
- 快照：branch `design/phase-c2a`，HEAD **`981e588`**；写本文件前工作树 clean。只读校验，未改源码/设计/git，只写本文件。
- 方法：逐条重核上轮 D1–D5；实际重算 scenario 固定向量；对照 `sandbox::resolve`、`Model::load`、host SourceModel、DB v4；运行 docs/schema 检查与 whatif/timed lib 测试。
- 结论：**CHANGES_REQUIRED（4 个 C2a 阻断 + 3 个动工前）**。D1/D2/D5 主体已闭合；D3/D4 仍有残留，并发现 startMode 与持久化契约两处新阻断。

## 已核对（非阻断）

- **D1 固定向量正确**：`sha256("poie.timed-scenario/0.2\n" + raw bytes)` 对 `scenarios/base-01-timed/scenario.toml`（2925 bytes，UTF-8）实测为 `5cbfc5a5711b316318fd244c89149b9ef58b6f5d8c417c4747638dc7c1c205f3`，与 C2a:54 完全一致；DIGEST §2.1 已增补 raw-file profile；resultSha（C2a:105）已纳入 variant 的 startS/deadlineS/storageDischargeW。域登记见 R2。
- **D2 独立性成立**：C2a 不再扩展 C1 Snapshot/Variant/OutcomeKind；`git diff e9271f3..981e588` 仅改 docs；无 PatchDto/assetDelay/checkpoint；`whatif_timed_*` 为独立表，4→5 建表不改 C1 JSON/表。
- **D5 成立**：TimedCompareResult 独立；C2a:118 跨 snapshot 拒绝、不混 C1 静态结果；timed PatchDto/assetStatus 明确延后 C2b（C2a:26,180）。
- **D4 部分成立**：host 流程改用 `from_source` + `sandbox::resolve`，不再走 `TimedScenario::load` 的 path join；`from_source`/`evaluate` 属 C2a §4 前置，HEAD 未实现，符合设计。但见 P1。
- 检查：`check-docs.py` OK；`check-schema-registry.py` OK（35 schemas）；`cargo test -p poie-whatif --lib` 5/5；`cargo test -p poie-sim --lib timed` 6/6。

## 阻断（C2a BLOCKER）

1. **P1 — base-01-timed 按 §4.2 无法导入，固定向量与验收互相矛盾。** `scenarios/base-01-timed/scenario.toml:7` 的 `model = "../base-01/model.toml"`；C2a:44 要求 model 是 **scenario 所在目录下**的相对路径并拒绝 `..`，而 `sandbox::resolve`（sandbox.rs:17–21）对任何 `ParentDir` 直接报错。C2a:54 的固定向量正是该文件原文，C2a:173 又要求 UI 导入 base-01-timed。结果只能是：导入被拒（验收/测试失败），或 host 复制/改写 scenario 的 model 字段（设计未定义，且改写后 raw bytes 不再等于固定向量）。另 C2a:45 允许用 `Model::load`，它按路径重新 `std::fs::read_to_string`（model.rs:397–401），绕过 `sandbox::read` 的 no-follow/1 MiB；应改为只对 `sandbox::read` 得到的文本调用 `Model::from_doc`。修法：定义 sandbox 布局（model 位于 scenario 目录内并更新 fixture/向量）、或允许“规范化后仍在 root 内”的解析（不能用当前 `sandbox::resolve`）、或改为绑定 host SourceModel；二选一并写清向量。
2. **P2 — startMode 是 snapshot 级，验收却要求“startMode 变体”可比较。** C2a:67/90 规定 `startMode` 由 snapshot 决定，variant 只覆盖 startS/deadlineS/storageDischargeW；C2a:118 又只接受同 snapshot 的 TimedResult，跨 snapshot 直接 `invalid_parameter`；但 C2a:174 验收要求“3 个 timed 变体（储能限制/截止提前/startMode）可比较”。startMode 变体在模型上不存在；若用不同 snapshot 表示 startMode，compare 会拒绝。另有冲突：C2a:73 规定 `beginning` 时 startS=0，C2a:90 却允许 variant startS 覆盖 snapshot，beginning+startS 的优先级未定义。修法：把 startMode 下放到 variant 并定义校验，或从 3 变体验收/同 snapshot compare 中移除；明确 beginning 与 variant startS 的组合规则。
3. **P3 — §5.1 的 SnapshotInput/horizonS 仍未定义，D3 只是从 timedRange 改名。** C2a:126 使用 `TimedSnapshotInput`，全文无字段/默认值/校验定义；C2a:68/73/89 的 `horizonS`/`nowS` 只作为上界出现，没有来源（调用者输入还是场景推导）、取值范围、与 scenario 时间线/`nowS` 的关系，也没有 snapshot 固定向量。host 无法确定 `startS ∈ [0,horizonS]` 与 `deadlineS ≤ horizonS` 的校验基准，也无法测试 snapshot 创建。另 C2a:73 写 `snapshotSha`，字段实际为 `snapshotSha256`（C2a:69）。修法：定义 TimedSnapshotInput 全部字段与校验，明确 horizon/now 的来源与推导（若推导需给只读 accessor 和固定向量），统一字段名。
4. **P4 — 持久化 DTO 违反 schemaVersion 规则，compare/命令契约仍不完整。** PLATFORM:20 要求每个持久化结构带 `schemaVersion`；C2a 只有 TimedSnapshot 有（C2a:65），TimedVariant（C2a:80–85）与 TimedResult（C2a:95–102）均无，`whatif_timed_variants`/`whatif_timed_results`（C2a:144–145）也没有 `schema_version` 列（C1 表有）。此外 TimedCompareResult 的 `delta` 未定义类型/含义（C2a:113），`baselineResultId?` 缺省时如何算 delta 未定义（C2a:112,118）；`whatif_timed_scenario_list` 输出形状未定义（C2a:125）。这些使 API/数据模型不满足门禁的“完整、可测试”。修法：补 schemaVersion 字段/列，定义 delta 与 baseline 语义，定义 scenario list 输出。

## 动工前（REQUIRED-BEFORE）

- **R1 模型身份/SourceModel。** PLATFORM:103–112 规定桌面唯一 SourceModel（Base-01 + pump-p1 degraded），A/C/D 只有一套模型来源；C2a:44–46/75 却从 sandbox 读取任意 model.toml 并只与 snapshot 自身的 modelSha 比对，从不要求等于 `state.model`（host 用 `include_str!` 的 Base-01 + patch，domain.rs:12,93–102）。需明确 C2a 模型就是 SourceModel，或更新 PLATFORM 并把 sandbox 模型纳入可信 digest/白名单；否则可在不改变 host 当前模型的情况下用另一份模型得出结论。
- **R2 digest domain 与 schema 注册表。** C2a:73/105 使用 `poie.timed-snapshot/0.1` 与 `poie.timed-result/0.1` 作为 digest domain，但 DIGEST §1/§1.2 及 SCHEMAS 的 “Digest domains” 表均未登记；SCHEMAS:23–24 仍把 `poie.variant/0.2`/`poie.snapshot/0.2` 登记为 C2a，和“不扩展 C1 类型”直接矛盾。`check-schema-registry.py` 只查 wire schema，不会发现这两类漂移。需增补/删除并重跑检查。
- **R3 digest 可复现性与向量。** TimedSnapshot digest 纳入 `scenarioPath`（C2a:66,73）；同一内容放在不同 sandbox 路径会得到不同 snapshotSha，进而不同 resultSha，与 DIGEST §2.7“不包含机器路径”冲突。需把 path 移出 digest（仅作为定位/展示）或明确其属于被测输入；并补 snapshot/result 固定向量、4→5 旧库迁移测试、base-01-timed 导入测试（当前只有 scenario 向量）。

## 边界

只读设计/源码校验；除本文件外未改任何文件。未运行 Tauri/Playwright，未验证真实 app sandbox 或 UI。C2b 延后声明通过。

VERDICT: CHANGES_REQUIRED
