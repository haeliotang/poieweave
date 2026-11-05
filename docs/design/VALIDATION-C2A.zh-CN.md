# Phase C2a 设计门禁（独立复验）

- 校验对象：[phase-c2-timed-what-if](phase-c2-timed-what-if.zh-CN.md)（C2a）、[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)/[PLATFORM](PLATFORM.zh-CN.md)、C1；代码 `poie-sim::timed`、`poie-ontology::time`、host `sandbox.rs`/`db.rs`/`domain.rs`。
- 快照：branch `design/phase-c2a`，HEAD **`ad722e9`**；写本文件前工作树 clean。只读校验，未改源码/设计/git，只写本文件。
- 方法：逐条复核上轮 P1–P4；重算 scenario 向量并构造 result 向量；对照 `sandbox::resolve`、`Model::load`、`Model::from_doc`、host SourceModel、DB v4；运行 docs/schema 检查与 whatif/timed lib 测试。
- 结论：**CHANGES_REQUIRED（4 个 C2a 阻断）**。四项修复都有进展，但 P1/P2/P3/P4 均有残留，尚不能判 PASS。

## 已核对（非阻断）

- **scenario 固定向量正确**：`sha256("poie.timed-scenario/0.2\n" + raw bytes)` 对 `scenarios/base-01-timed/scenario.toml`（2925 bytes，UTF-8）实测为 `5cbfc5a5711b316318fd244c89149b9ef58b6f5d8c417c4747638dc7c1c205f3`，与 C2a:57 一致；DIGEST §2.1 已增补 raw-file profile。
- **模型路径方向正确**：`from_source(text, model)` + `sandbox::read` + `ModelDoc`/`Model::from_doc` 不再走 `TimedScenario::load` 的路径拼接；`resolve_inside` 允许归一化后的 `../base-01/model.toml`，并绑定 `state.model` digest。base-01 的 pump 本身已是 degraded，host 的同一 patch 不改变 digest，导入基准成立。
- **startMode 归属已修**：`startMode` 在 TimedVariant，snapshot 只有默认 `startS`；同 snapshot compare 与“3 个 startMode/startS 变体”验收不再互相矛盾。
- **schemaVersion 基本补齐**：TimedVariant/TimedResult 有字段，`whatif_timed_scenarios`/`variants`/`results` 有列；delta/baseline 的基本规则已写入。
- 检查：`check-docs.py` OK；`check-schema-registry.py` OK（35 schemas）；`cargo test -p poie-whatif --lib` 5/5；`cargo test -p poie-sim --lib timed` 6/6。

## 阻断（C2a BLOCKER）

1. **B1 — `..` 规则自相矛盾，base-01-timed 导入仍未真正闭合。** C2a:46 明确允许 `../base-01/model.toml`，但 C2a:163 仍写“拒绝绝对路径/`..`/symlink/超限”；同一设计两套相反规则，按 §8 实现仍会拒绝该 fixture。C2a:45 还把 `path` 说成“sandbox 相对目录（含 scenario.toml）”，而 `sandbox::read` 只能读普通文件；未定义 `path` 是目录还是 `base-01-timed/scenario.toml`，也未写归一化后如何把 root 相对路径交给 `sandbox::read`。需统一为“拒绝归一化后逃逸 root 的 `..`”，并补 fixture 落盘/导入测试。
2. **B2 — variant startS/deadline 与 horizon/now 语义仍不完整。** C2a:95 的 `deadlineS > startS` 未说明 `beginning` 时用 0 还是 variant/snapshot 的 `startS`；C2a:93（beginning=0、at=startS??snapshot）与 C2a:96（startS 覆盖 snapshot）仍有优先级歧义。variant `startS` 没有 `[0,horizonS]` 上界，也没有负值/NaN 规则。C2a:36 的 `TimedCase` 不含 `horizonS`/`nowS`，所以 horizon 只用于 snapshot 校验，无法约束 run/completion；`nowS` 在 run/result 中没有用途；snapshot `deadlineS` 也没有校验（C2a:76）。
3. **B3 — 固定向量不可独立复现，且 `snapshotSha` 命名未修。** C2a:79 的 `scenarioSha=5cbf…` 是省略号，缺 label/minVerification/nowS/deadlineS，按 C2a:77 无法唯一构造 payload，固定向量不能作为测试输入。C2a:112 的 result 向量只有在补上未写的 `startS=0, horizonS=86400, deadlineS=null, completionS=null, warnings=[]` 时才匹配 `e7e6…`（我已实测）；文本没有给出这些值，向量不可验证。另 C2a:77 仍排除 `snapshotSha`，实际字段是 `snapshotSha256`（C2a:72），自哈希字段排除规则错误。
4. **B4 — compare/list/DB/schema 契约仍有缺口。** C2a:126 只定义 delta 形状，未定义正负号（row-baseline？）、timed verdict 的“固定顺序”枚举，以及 baseline 不在 rows 时是否 warning/错误。C2a:133 的 `whatif_timed_scenario_list` 输出仍只是 “imported scenarios”，无 DTO/字段；TimedCompareResult（C2a:118）无 `schemaVersion`；`whatif_timed_snapshots`（C2a:151）仍无 `schema_version` 列。SCHEMAS:23–24 仍把 `poie.variant/0.2`/`poie.snapshot/0.2` 登记为 C2a，与“独立 timed 类型、不扩展 C1”直接矛盾；SCHEMAS:36–50 的 digest domain 表未登记 timed 域。`check-schema-registry.py` 只查未注册，不会发现这些漂移。

## 边界

只读设计/源码校验；除本文件外未改任何文件。未运行 Tauri/Playwright，未验证真实 app sandbox/UI；`cargo test` 仅覆盖现有 lib。C2b 延后声明通过。

VERDICT: CHANGES_REQUIRED
