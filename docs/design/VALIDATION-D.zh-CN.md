# Phase D（D1）设计门禁终验（独立）

- 校验对象：[D1](phase-d-site-delivery.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[README](README.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)。
- 快照：branch `design/phase-d-d1`，HEAD **`53b63cb`**；写本文件前 `git status --porcelain` 干净；只读校验，只覆盖本文件。
- 方法：核对 `23e7ac9..53b63cb` 全部 4 文件改动；逐条复验 3 组声称修复；独立重算 DIGEST 固定向量；运行 `check-docs.py`、`check-schema-registry.py`、`cargo metadata`。D1 未实现，不以未实现计失败。
- 结论：**CHANGES_REQUIRED**。§1 自哈希例外与 UI Clear 控件已落地；`invalid_parameter` 的 D1 注册仍缺失，未达零阻断。

## 声称修复核对

| 声称 | 结果 |
| --- | --- |
| PLATFORM §1 登记 D1 自哈希 payload 拒绝未知字段；SCHEMAS D1 行登记；D 引用 §1 | **成立**：PLATFORM:19 明列 `SiteBundle`/`SiteManifest` 例外；SCHEMAS:22 登记、:23“同上”；D:50 改引 §1。 |
| D1 错误码移除 `scope_denied`；PLATFORM `invalid_parameter` 改 A/B/C/D1；diagnostics 白名单已排除 | **部分成立**：D:104 已删 `scope_denied` 并声明不产生，D:158 白名单无该码；但 PLATFORM:57 仍为 `A/B/C`，PLATFORM:55 的 D1 行不含 `invalid_parameter`，`git log -S 'A/B/C/D1'` 无记录（见阻断 1）。 |
| UI 增加 Clear 控件 | **成立（§8）**：D:169 增加 Conflict 时 Clear；但 D:192 的 UI 验收仍只列 import/rollback，未列 clear（非阻断）。 |

## 阻断

1. **[阻断] `invalid_parameter` 仍未登记为 D1 错误码。** D1 对 bundle/manifest/字符串/数组/嵌套超限统一用 `invalid_parameter`（D:51），命令错误码（D:104）与 diagnostics 白名单（D:158）也包含它；但 PLATFORM:57 仍把该码限定为 A/B/C，PLATFORM:55 的 D1 行只有 `manifest_invalid/stale_manifest/delivery_conflict/rollback_unavailable/io_error`。`53b63cb` 对 PLATFORM 只改了 §1（:19），未改 §3，故声称的 `A/B/C/D1` 不存在。按 PLATFORM 实现将无法为 D1 超限错误映射合法码，错误契约仍非单一真源。修法：把 `invalid_parameter` 加入 PLATFORM:57 与/或 D1 行（A/B/C/D1），或把 D1 上限错误改为 `manifest_invalid` 并同步 D:51/104/158。

## 其他（非阻断）

- D:192 的 UI 验收未随 D:169 的 Clear 控件更新，建议补“Conflict 时 Clear 后重新 import”。
- SCHEMAS:23–24 的“同上”把“自哈希”描述一并继承给 `site-bundle`/`delivery-state`；建议仅对 manifest 声明自哈希，其余只声明拒绝未知字段。
- DIGEST:96 向量仍为 `{schema,siteId,target,channel}` 四字段片段，未钉住 `schemaVersion`/`policy`/null 字段的 canonical 行为（既有非阻断）。
- README:133“D1：设计门禁 PASS 后进入实现”在门禁未过时仍宜标为“复验中”。

## 已执行检查

```text
git rev-parse --short HEAD                         53b63cb
git status --porcelain（写前）                      0 行
git diff --stat 23e7ac9..53b63cb                    4 files（含本文件）
recompute DIGEST vector                            0a15ba7d…964d，match
python3 scripts/check-docs.py                      OK, exit 0
python3 scripts/check-schema-registry.py           27 schemas, exit 0
cargo metadata --no-deps --offline                 28 members; poie-delivery absent
git log -S 'A/B/C/D1' -- PLATFORM                  无记录
```

边界：只读校验；除本文件外未改任何文件；未跑全量 cargo/Tauri/Playwright；D1 尚未实现，未以未实现计为失败。

VERDICT: CHANGES_REQUIRED
