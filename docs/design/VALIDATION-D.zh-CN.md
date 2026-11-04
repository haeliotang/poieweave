# Phase D（D1）设计门禁终验（独立）

- 校验对象：[D1](phase-d-site-delivery.zh-CN.md)、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[README](README.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)；代码 `apps/desktop/src-tauri`（auth/db/sandbox/lib）、`desktop-build.yml`、`tauri.conf.json`。
- 快照：branch `design/phase-d-d1`，HEAD **`23e7ac9`**；写本文件前 `git status --porcelain` 干净；只读校验，只覆盖本文件。
- 方法：核对 `9f1c21b..23e7ac9` 全部 7 文件改动；逐条复验上轮 5 组声称修复；独立重算 DIGEST 固定向量；核对 host auth/db/sandbox/命令注册；运行 `check-docs.py`、`check-schema-registry.py`、`cargo metadata`。D1 未实现，不以未实现计失败。
- 结论：**CHANGES_REQUIRED**。DIGEST/Phase A、SCHEMAS、README 范围与 clear 审计/测试已闭合；但 PLATFORM 未知 minor 例外与 D1 错误码表仍有 3 项未闭合。

## 声称修复核对

| 声称 | 结果 |
| --- | --- |
| DIGEST D1 payload 无扁平 sha、无 integrity；Phase A 对齐；向量真 | **主体成立**：DIGEST:26/91/94 与 A:153 一致；重算 canonical JSON（channel,schema,siteId,target）+ 域前缀得 `0a15…964d`，与 DIGEST:96 相符。向量仅 4 字段片段（见下）。 |
| `delivery_clear` 在 audit/tests；diagnostics 行无审批 | **成立**：D:100/107/190；D:101 已标“无审批”。 |
| PLATFORM delivery 行 main-only D1 auth+audit | **成立**：PLATFORM:90/95、:167；README:67 已对齐。 |
| D1 错误码已登记 | **部分成立**：5 个 D1 码已入 PLATFORM:55；D:104 仍列 `scope_denied`，`invalid_parameter` 未纳入 D1 行（见阻断 2/3）。 |
| SCHEMAS 旧 diagnostics 删除 | **成立**：已无 `poie.diagnostics/0.1`；registry 27 schemas，exit 0。 |
| README release manifest 移至 D2 | **成立**：README:94–95 改为 D2；D 行/P2/退出条件/R17 已对齐。 |
| path 用 resolve+reject_symlinks | **主体成立**：`sandbox.rs:11` resolve、`:26` reject_symlinks（私有）、`:47/:62` read/write 调用；D:159 违规统一 `io_error`。 |

## 阻断

1. **[阻断] PLATFORM §1 未知 minor 容忍规则仍未登记 D1 例外。** PLATFORM:19 仍规定所有 schema“未知 minor 可接受，忽略未知字段并记录警告”；D:50 却要求 D1 manifest 拒绝未知字段，并称“PLATFORM §2 的 general tolerance 不适用”——该规则实际在 §1（D:50 引用错节），PLATFORM 与 SCHEMAS:22 兼容列均未登记例外。同一 `poie.site-manifest/0.1` 存在两套解析契约，自哈希/`stale_manifest` 仍无唯一语义；上轮“在 PLATFORM 登记例外”未完成。需在 PLATFORM §1（或 SCHEMAS:22）写明“自哈希 payload 拒绝未知字段”，并修正 D:50 的节号。

2. **[阻断] `scope_denied` 仍未消解。** D:104 作为 D1 命令错误码列出 `scope_denied`；PLATFORM:50 明确“D1 delivery 不产生”，D:158 的 diagnostics 白名单也不含它，D:103/PLATFORM:95 又声明 D1 无 Window/scope 机制。该码不可达且与 D 自身 §7 矛盾；上轮“消解 `scope_denied`”未完成。需从 D:104 删除（D1 不应恢复 scope 机制）。

3. **[阻断] D1 错误码注册仍不完整。** D:51/104 用 `invalid_parameter` 处理 bundle/manifest 上限；PLATFORM:57 仍把 `invalid_parameter` 限定为 A/B/C，PLATFORM:55 的 D1 行不含它。故“D1 error codes registered”只覆盖实际码中的 5/6。需把 `invalid_parameter` 加入 D1 行，或将 D1 上限错误改为 `manifest_invalid` 并同步 D:51/104/158。

## 其他（非阻断）

- D §8/§10 的 UI 控件无 Clear，但 `delivery_clear` 是 `delivery_conflict` 唯一出口（D:88/128）且已入命令/审计/测试；host 可达，UI 流程却无出口。建议加 Clear 控件并列入 UI 验收。
- DIGEST:96 向量是 `{schema,siteId,target,channel}` 四字段片段，不是完整 SiteManifest（缺 schemaVersion/siteName/releasedAt/modelSha256/evidenceSha256/policy/notes），未钉住 null 与嵌套 policy 的 canonical 行为。
- README:133“D1：设计门禁 PASS 后进入实现”在门禁未过时仍不宜作为当前状态；改为“门禁复验中”更准确。
- `sandbox::reject_symlinks` 是私有函数；设计应写明 delivery.rs 通过公开 `sandbox::{read,write}` 获得 symlink 拒绝，或将其公开。

## 已执行检查

```text
git rev-parse --short HEAD                         23e7ac9
git diff 9f1c21b..23e7ac9 --stat                   7 files
git status --porcelain（写前）                      clean
recompute DIGEST vector                           0a15ba7d…964d，match
python3 scripts/check-docs.py                      OK, exit 0
python3 scripts/check-schema-registry.py           27 schemas, exit 0
cargo metadata --no-deps --offline                 28 members; poie-delivery absent
host                                               auth.rs:156 require->String; db.rs:135 user_version=3; sandbox.rs:11 resolve, :26 reject_symlinks（私有）, :47/:62 read/write
workflow/tauri                                     desktop-build paths 无 crates/**; tauri.conf targets="all"（D1 未实现/D2）
```

边界：只读校验；除本文件外未改任何文件；未跑全量 cargo/Tauri/Playwright，也未把 D1 未实现计为失败。

VERDICT: CHANGES_REQUIRED
