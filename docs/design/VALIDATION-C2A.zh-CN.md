# Phase C2a 设计门禁（最终独立复验）

- 校验对象：[phase-c2-timed-what-if](phase-c2-timed-what-if.zh-CN.md)、[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)/[PLATFORM](PLATFORM.zh-CN.md)、C1；代码 `poie-sim::timed`、`poie-whatif`。
- 快照：branch `design/c2a-final`，HEAD **`9431c48`**；写本文件前工作树 clean；只读校验，只写本文件。
- 方法：逐条复核 B1/B2/B3；用 canonical JSON 重算 §4.3/§5.1/§5.3 向量；比对 DIGEST §1 与 SCHEMAS digest 域集合；运行 docs/schema 检查与两个 lib 测试。
- 结论：**CHANGES_REQUIRED（2 个 C2a 阻断残留）**。B1 与 B4 的 delta 语义已闭合；B2/B3 仍有契约/注册表残留。

## 已核对（非阻断）

- **B1 已闭合（seed）**：C2a:68–77 的 TimedSnapshot/TimedSnapshotInput 与 C2a:80 的固定向量均无 `seed`；按 canonical JSON 实测 `b4a1459860896c689d26c4ffd752cfb2d071891d63f4f3df07860a31505e9835`，与 C2a:80 一致；旧含 seed 的 `34dd4a21…01ef` 不再被声称。C2a:78 仍写 `without .../seed/...`，seed 已非字段，属死引用，建议删除（非阻断）。
- **now/start/deadline 规则主体已闭合**：C2a:36 删除 `now_s` 并固定 now=0、`start_s∈[0,horizon_s]`；C2a:77 固定 `nowS=0`、`startS∈[0,horizonS]`、`deadlineS∈(startS,horizonS]`；C2a:94 的 beginning=0/at=startS 与 C2a:96–97 的 resolved-start deadline 一致；仅 §6 命令表仍残留 nowS，见阻断 1。
- **B4 delta 已闭合**：C2a:127 明确 row/baseline 任一 `completionS` 为 null 时 `completionDeltaS=null`（不做减法），baseline 不在 rows 时 `delta=null` 并 warning；排序仍为 verdict 顺序 → completionS null 最后 → resultId。SCHEMAS:22 的 `poie.timed-scenario/0.1` 行已删、digest 表已增 timed 行，但域集合仍与 DIGEST 不一致，见阻断 2。
- 检查：`check-docs.py` OK；`check-schema-registry.py` OK（32 schemas）；`cargo test -p poie-whatif --lib` 5/5；`cargo test -p poie-sim --lib timed` 6/6；scenario 向量 `5cbfc5a5…c205f3`、result 向量 `e7e6e69a…29d423` 重算一致。

## 阻断（C2a BLOCKER）

1. **B2 残留 — `whatif_timed_snapshot_create` 仍把 `nowS` 列为输入。** C2a:135 写 “TimedSnapshotInput（含 startS/horizonS/nowS）”，而 C2a:77 的输入 DTO 不含 `nowS` 且规定 `nowS=0`；C2a:72 的 `nowS` 只是 snapshot 的固定字段。调用方能否/必须传 nowS、传非 0 如何处理均未定义；按 §6 实现会给 host 留下 nowS 来源，重现 B2 的 start 校验歧义。需把该行改为仅 `startS/horizonS`（nowS 固定 0）。
2. **B3 残留 — digest 域注册表仍不一致。** DIGEST:24 仍登记 `poie.timed-scenario/0.1`（payload 为 canonical JSON，C2a 无使用；SCHEMAS 已删该行），而 SCHEMAS:51 新增 `poie.timed-variant/0.1`：DIGEST §1 无此域，C2a:86–90 的 TimedVariant 无 `variantSha256`，§5.3 只定义 result digest，故该域无 payload/算法。两张域表在 timed 集上各多一项（0.1 vs variant）；且 DIGEST:25–28 的 timed 行仍是两列（payload 写成 “C2a”、用途空）。需统一为实际使用的域并删除或补定义未使用行。

## 边界

只读设计/源码；除本文件外未改任何文件；未运行 Tauri/Playwright，未验证真实 app sandbox/UI；`cargo test` 仅覆盖现有 lib。`sandbox::resolve_inside` 仍未实现（现只有拒绝 `..` 的 `resolve`），不改变本门禁结论；C2b 延后声明不受影响。

VERDICT: CHANGES_REQUIRED
