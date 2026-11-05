# Phase C2a 设计门禁（最终独立复验）

- 校验对象：[phase-c2-timed-what-if](phase-c2-timed-what-if.zh-CN.md)、[DIGEST](DIGEST.md)/[SCHEMAS](SCHEMAS.md)/[PLATFORM](PLATFORM.zh-CN.md)、C1；代码 `poie-sim::timed`、`poie-whatif`。
- 快照：branch `design/c2a-final`，HEAD **`bfeee6b`**；写本文件前工作树 clean；只读校验，只写本文件。
- 方法：复核 9431c48 的两项残留与 seed 清理；用 canonical JSON 重算 §4.3/§5.1/§5.3 向量；比对 DIGEST §1 与 SCHEMAS digest 域集合；跑 docs/schema 检查与两个 lib 测试。
- 结论：**PASS（0 个 C2a 阻断）**。

## 已核对

- **(a) 已闭合**：C2a:135 现为 `TimedSnapshotInput（startS/horizonS；nowS 固定 0）`，输入仅 `startS/horizonS`；`nowS` 是 C2a:72 的 snapshot 固定字段，C2a:77 规定 `nowS=0`，不再是调用方输入。`nowS` 字样仍作固定值说明，与 DTO 不矛盾。
- **(b) 已闭合**：DIGEST:24–27 四个 timed 域均有 payload/用途，`poie.timed-scenario/0.1` 已删；SCHEMAS:48–51 digest 域与 DIGEST §1 集合完全一致（15/15）；`poie.timed-variant/0.1` 仅保留在 wire schema 表（SCHEMAS:24），未进 digest 域。
- **seed/向量**：C2a:78 已删 seed；snapshot 向量 `b4a1459860896c689d26c4ffd752cfb2d071891d63f4f3df07860a31505e9835`、result `e7e6e69a44e8b31bd01af60f754b85173aa2fe64b9b27c148c7cd010ce29d423`、scenario `5cbfc5a5711b316318fd244c89149b9ef58b6f5d8c417c4747638dc7c1c205f3` 重算一致；`check-docs.py` OK、`check-schema-registry.py` OK（32 schemas）、`cargo test -p poie-whatif --lib` 5/5、`cargo test -p poie-sim --lib timed` 6/6。

## 非阻断备注

- DIGEST:24 的 payload 摘要未复述 C2a:78 的 `scenarioPath` 排除；C2a 规则与固定向量一致，属摘要表述，不阻断。
- `poie.timed-compare/0.1` 已补 payload 并标注“当前未使用”；符合上一轮“删除或补定义”的要求。

## 边界

只读设计/源码；除本文件外未改任何文件；未运行 Tauri/Playwright，未验证真实 app sandbox/UI；`cargo test` 仅覆盖现有 lib。`sandbox::resolve_inside` 仍未实现（只有拒绝 `..` 的 `resolve`），不改变结论；C2b 延后声明不受影响。

VERDICT: PASS
