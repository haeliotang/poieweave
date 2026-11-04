# Phase D（D1）设计门禁复验（独立）

- 校验对象：[D1](phase-d-site-delivery.zh-CN.md)（下称 D）、[PLATFORM](PLATFORM.zh-CN.md)、[SCHEMAS](SCHEMAS.md)、[DIGEST](DIGEST.md)、[README](README.zh-CN.md)、[Phase A](phase-a-ontology-product.zh-CN.md)；代码 `apps/desktop/src-tauri`（auth/lib/db/sandbox）、`crates/poie-digest`、`tauri.conf.json`、`.github/workflows`、`scripts/`。
- 快照：branch `design/phase-d-d1`，HEAD **`684cbaf`**；写本文件前 `git status --porcelain` 干净；只读校验，只写本文件。
- 方法：核对 `028701c..684cbaf` 全部改动（4 文件）；逐条复验上轮 6 组阻断；核对 host 的鉴权/DB/沙箱/命令注册、Tauri 目标、desktop workflow；运行 `check-docs.py`、`check-schema-registry.py`、`cargo metadata`。D1 尚未实现，不以未实现为失败。
- 结论：**CHANGES_REQUIRED**。已核对成立：digest domain 改为 `poie.manifest/0.1`、上限数值、raw-JSON 先解析、审计事件、`truncated` 字段、redaction 规则、`overwrite` 参数、state 摘要化（§4.3 内）、swap、`recalled` 派生、同 sha no-op、`delivery_conflict` 触发条件、diagnostics 路径 sandbox 化（仅输出侧）、D1 只给 workflow 加 `crates/**` 过滤。但上轮阻断 1/3/4/5/6 仍有未闭合项，且 `684cbaf` 的 PLATFORM 改动引入新的内部矛盾。

## 判定

1. **[阻断] 鉴权/审批：D1 的“无审批”仍与 PLATFORM/README 相反，且 PLATFORM 自相矛盾。** D:100 断言 import/rollback “不需要 ApprovalToken/HostPolicy/Window（这与 PLATFORM D1 行一致）”。`684cbaf` 只改了 PLATFORM:195 的实施顺序（把 approval/policy 划给 D2）；授权矩阵 PLATFORM:89 仍写 `delivery_*` = “会话 + 审批 + policy”，PLATFORM:94 仍要求每个命令 Window + `authorize(...)`，PLATFORM:97 仍要求统一 `policy.check`，PLATFORM:166 仍写“D 的 channel/import 使用同一原语”；README:67 的 P2 仍要求交付写入受 ApprovalToken/HostPolicy 约束。于是 PLATFORM §8 与 §4/§6 互相矛盾，D:100 引用的“D1 行”并不是授权契约行。host 也没有 `authorize`/`CommandError`（auth.rs:156 只有 `require`；lib.rs 命令均返回 `Result<_, String>`），D:100 的“统一 `{code,message,details}`”没有类型/映射定义，D:101 还列了无 scope 机制支撑的 `scope_denied`。需二选一：恢复审批/Window/policy，或在 PLATFORM §4/§6 与 README P2 显式登记 D1 例外（含审计与错误包络映射），并删除“与 D1 行一致”的错误断言。

2. **[阻断] manifest digest 仍与 DIGEST 冲突，TOCTOU 无唯一语义。** D:49 的 domain 已改为 `poie.manifest/0.1`，但公式排除扁平 `manifestSha256`；DIGEST:26/91 规定 `canonical_json(manifest_without_integrity)`，DIGEST:94 明说“`integrity.manifest_sha256` 是唯一排除字段；其他字段（包括 artifacts/evidence/contracts）全部纳入”，Phase A:153 同。D1 manifest 没有 `integrity` 对象（D:35–46），按 DIGEST §5 实现会排除空集并把待算字段纳入自哈希，所得 sha 与 D:49 不同；`expectedManifestSha256`/`stale_manifest` 仍无唯一实现。`684cbaf` 未改 DIGEST/Phase A，也未给 DIGEST §6.5 要求的固定 fixture 向量（D §10 仅“自哈希稳定”）。另 D:50 的“拒绝未知 minor 字段”只在 D 内声明，PLATFORM:19 的容忍规则未登记该自哈希例外。需在 DIGEST §5 写清 D1 payload/排除字段（或声明 D1 不适用该节）、补固定向量，并在 PLATFORM §2 登记例外。

3. **[阻断] DeliveryState 仍有直接矛盾，冲突出口不可执行。** D:17 目标 3 仍写“`DeliveryState` 保存当前/上一份**完整 manifest** 与摘要”，D:85 却写“state **只存摘要**，不重复存完整 manifest”；上轮阻断 3 的原文矛盾未删。D:72–78 的结构体仍只有 `schema/current/previous/recalled/updatedAt`，缺 D:88 与 PLATFORM:20 强制要求的 `schemaVersion` 字段（仅正文口头“必填”）。D:77 的 `importedAt/importedBy` 仍无类型/来源（D:104 审计与 D:152 诊断边界依赖它）。D:125 给 `delivery_conflict` 的出路是“先 rollback/清空”，但 D1 无 clear/reset 命令，且 D:86 的 swap 在已有 previous 时只能在两个 site 间来回切换、永远到不了 `current=null`；正常历史下换站点是死局。需统一 state 语义、补 struct 字段、定义可执行的冲突出口（如 `delivery_reset` 或显式换站规则）。

4. **[阻断] 诊断/路径策略仍自相矛盾，inspect/import 未绑定 sandbox。** D:155 要求 `outputDir` 是“sandbox 下已存在目录、无 symlink 逃逸”，D:164 却让用户“Export diagnostics 到用户选择目录”；D:161 的“文件选择器”也拿不到 sandbox 相对路径——`sandbox::resolve` 只接受相对路径（sandbox.rs:14–25），host 只注册 `core:default`（capabilities/default.json）且无 dialog 插件，D 未说明新增依赖/权限（PLATFORM:186 要求 ADR）。D:82/83 的 inspect/import `path` 与 §9 的“文件读取（UTF-8/大小）”仍未绑定 relative/no-follow/尺寸上限（manifest 128 KiB 与 sandbox 1 MiB 不同）。另 D:146/154 的 `detailCode`“固定枚举”未列取值，`overwrite=false` 且目标存在时的错误码也未定义。需统一导入源/导出目的地的路径契约（含 UI 能力），并补枚举与错误码。

5. **[阻断] D1/D2 范围仍未落到 README，“README 已更新”不成立。** `684cbaf --stat` 只改 PLATFORM/SCHEMAS/VALIDATION-D/phase-d，`README.zh-CN.md` 未动：README:12 仍把 D 写成“通道、站点 bundle…发布流水线”，README:67 仍要求交付写入受 ApprovalToken/HostPolicy 约束，README:83 退出条件仍含“通道”，README:119（R17）仍要求 D 增加 inspect/plan/recall/channel，README:133 仍写“D 门禁未跑”。PLATFORM:181 的 `poie-delivery` 依赖仍是 `ontology/state/viability`，与 D:171–173 的“仅 digest/serde、无 Tauri/网络/fs”冲突；SCHEMAS:26 还留旧 `poie.diagnostics/0.1`（owner D），与 `poie.delivery-diagnostics/0.1` 重复。需更新 README 的 D 行/P2/退出条件/R17、PLATFORM:181 与 SCHEMAS 旧行。D:174–175 已明确 D1 只给 desktop workflow 加 `crates/**` 过滤、不改发布产物/安装器，该子项可接受。

## 已执行检查

```text
git rev-parse --short HEAD                        684cbaf
git status --porcelain（写本文件前）                clean
git diff 028701c 684cbaf --stat                   4 files: phase-d/PLATFORM/SCHEMAS/VALIDATION-D
python3 scripts/check-docs.py                      OK, exit 0
python3 scripts/check-schema-registry.py           28 schemas, exit 0（不校验 wire/digest 区分与重复用途）
cargo metadata --no-deps --offline                 28 members, poie-delivery absent
host                                               auth.rs:156 require；db.rs:135 user_version=3；sandbox.rs:14 relative-only
workflow/tauri                                     desktop-build.yml paths 无 crates/**；tauri.conf.json:31 targets="all"（D2）
```

边界：只读校验；除本文件外未改任何文件；未跑全量 cargo/Tauri/Playwright。

VERDICT: CHANGES_REQUIRED
