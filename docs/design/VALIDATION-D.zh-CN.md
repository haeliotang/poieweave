# Phase D（D1）设计门禁终验（独立复验）

- 对象：[D1](phase-d-site-delivery.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[README](README.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)。
- 快照：branch `design/phase-d-d1`，HEAD **`6ef4e7e`**；写本文件前 `git status --porcelain` 干净；只读校验，仅覆盖本文件。
- 方法：核对 `53b63cb..6ef4e7e` 全部 3 文件改动；复验上一轮 1 阻断与更早 3 阻断的闭合；独立重算 DIGEST 向量；复跑检查器并核对 host/workflow。D1 未实现，不以未实现计失败。
- 结论：**PASS**。上一轮唯一阻断（`invalid_parameter` 未登记 D1）已闭合，未发现新的 D1 阻断。

## 上一轮阻断复验

| 声称 | 结果 |
| --- | --- |
| PLATFORM 错误码行改为 `A/B/C/D1` | **成立**：PLATFORM:57 现为 `invalid_parameter`/`invalid_patch` ｜ `A/B/C/D1` ｜ 参数/变体非法/大小超限；对应 D:51。 |
| D UI 验收包含 clear | **成立**：D:192 已改为“rollback/clear”，与 D:169 的 Conflict 时 Clear、D:100/107/190 一致。 |
| 上一轮仅此 1 个阻断 | **成立**：校验 `53b63cb` 的报告（提交于 `6ef4e7e`）仅列 1 阻断；更早 3 阻断中的 §1 例外与 `scope_denied` 已在 `53b63cb` 闭合。 |

## 独立复核（零阻断）

- 错误码：D:104/158 的 8 个码均在 PLATFORM 登记且含 D1（:49/:51/:55/:57）；`scope_denied` 明确不由 D1 产生（PLATFORM:50）。
- 自哈希例外：PLATFORM:19、SCHEMAS:22–23、D:50 三处一致；D:50 已正确引用 §1。
- DIGEST：`poie.manifest/0.1` 域与 D:49、A:153、DIGEST:26/91/94 一致；四字段向量重算得 `0a15…964d`，命中 DIGEST:96。
- Clear/状态：D:88/128/169/190/192 与 README:83 对齐；clear 是 `delivery_conflict` 的显式出口。
- host/workflow：`auth::require`（auth.rs:156）、`user_version=3`（db.rs:135）、`sandbox::resolve/read/write`（sandbox.rs:11/46/58）与设计相符；无 `delivery_*` 实现；`targets="all"`、workflow 无 `crates/**` 属 D2/实现项。

## 非阻断残留

- SCHEMAS:24 的“同上”易被读成 `delivery-state` 也自哈希；建议只对 manifest 声明。
- DIGEST:96 仍是四字段片段，未钉住 `schemaVersion`/`policy`/null 的 canonical 行为。
- D:182 的“D0”未在 README/PLATFORM §8 定义；建议统一命名。
- `sandbox::reject_symlinks` 为私有，D:159 可注明 delivery 经公开 read/write 间接取得该保证。

已执行：`git rev-parse --short HEAD`→6ef4e7e；写前 status 0 行；diff 3 files；DIGEST 向量 match；`check-docs.py` exit 0；`check-schema-registry.py` 27 schemas exit 0；`cargo metadata --no-deps --offline` 28 members、poie-delivery absent；`git log -S 'A/B/C/D1'` 命中 6ef4e7e。

边界：只读校验；除本文件外未改任何文件；未跑全量 cargo/Tauri/Playwright；D1 未实现不计失败。

VERDICT: PASS
