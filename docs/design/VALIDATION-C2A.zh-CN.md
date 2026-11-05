# Phase C2a 设计门禁（独立校验）

- 校验对象：[phase-c2-timed-what-if](phase-c2-timed-what-if.zh-CN.md)（下称 C2a）、C1、`PLATFORM`/`DIGEST`/`SCHEMAS`；代码 `poie-ontology::time`、`poie-sim::timed`、`poie-whatif`、desktop host/DB/UI。
- 快照：branch `design/phase-c2a`，HEAD **`4feaba8`**；写本文件前工作树 clean。只读校验，未改源码/设计/git，只写本文件。
- 方法：逐条核对 C2a §4–§11；对照 `time.rs`/`timed.rs`/`whatif` lib 与 host、`db.rs`、`App.tsx`/`domain.ts`、`scenarios/`；运行 schema/docs 检查。
- 结论：**CHANGES_REQUIRED（5 个 C2a 阻断）**。C2b 延后声明诚实，未见 promote/checkpoint 泄漏；但 C2a 的 storage 模型、场景持久化、`assetDelayS` 语义、结果类型/compare 与 digest 契约均未闭合。

## 阻断（C2a BLOCKER）

1. **B1（BLOCKER）Storage 模型与真实 `TimedScenario` 冲突，且双权威/丢字段。** C2a:43–47 要求 `storage: BTreeMap<String, StorageLimit>{resource, capacity_si, initial_soc_si, max_discharge_w}`；实际 `timed.rs:225–235` 只有 `pub storage_discharge_w: BTreeMap<String,f64>`，`StorageDoc` 的 `id/asset/capacity/initial/max_charge_power`（timed.rs:115–123,307–337）解析校验后被丢弃，仓库中不存在 `StorageLimit`。C2a 未说明 `initial_soc_si` 与 `StockTimeline.initial_si` 谁是权威（timed.rs:62–70 明确“energy availability remains the stock timeline”），未要求两者相等，也未说明 `capacity_si` 在 runner 中的用途；当前 `power_limited` 只按步骤平均功率判断（timed.rs:521–528）。按现文实现会出现两套 SOC，digest 不含 id/asset/max_charge，且 variant 只能改 discharge。修法：以 `storage_discharge_w` 为唯一功率源；在 loader 加不变量（storage.initial==stock.initial、initial≤capacity）；`StorageLimit` 保留 id/asset/max_charge 并进 digest；或明确把 capacity/SOC 移出 C2a。

2. **B2（BLOCKER）场景来源/持久化/重启重建不可实现。** C2a:74 说 host 从“C2a 场景文件（Base-02 的 case 文件）”加载，但 `scenarios/base-02/` 只有 `model.toml` 与结构 `cases.toml`（schema `poie.scenario-cases/0.3`），无 `scenario.toml`；唯一 timed 场景是 `scenarios/base-01-timed/`。C2a:114 的 `SnapshotInput` 只有 `scenarioSha256`，没有路径/JSON；§6 没有 `whatif_scenario_import/list`；§7 的 `whatif_scenarios` 没有写入命令，sha 无法定位文件。C2a:128 存 `scenario_json`，但 `TimedScenario`/`AssetTimeline`/`StockTimeline`（timed.rs:225–235,72–78,180–185）都没有 `Serialize`，C2a 也未按 PLATFORM:24–36 定义 scenario DTO；重启后如何重挂 `Model`（scenario.model 只是相对路径）未定义。若接受用户路径，还存在任意读/路径穿越面。修法：增加 host allowlist 的 `whatif_scenario_import/list`（仅 `scenarios/`、限大小、重算 digest）；定义 `poie.timed-scenario/0.1` canonical DTO（modelSha + timelines + storage + description + validity）并以其作为 `scenario_json` 与 digest 输入；运行使用已校验的 live model 或存 model digest；修正 Base-02 引用或新增 `scenarios/base-02-timed/`。

3. **B3（BLOCKER）`assetDelayS` 既误判恢复，又对 timed 结论无效。** C2a:36–37 用 `status != Failed && status != Unknown` 判定“恢复”，但 Available→Degraded 的磨损变化也会被移动；谓词没有看前一状态/转移方向。更关键：`TimedRunner` 只在 `start_s` 调 `patched_model_at`（timed.rs:405–406,656–675），asset timeline 的后续 change 只用于 prediction warning（timed.rs:573–599），`schedule_route` 使用静态 model；因此把恢复从 10h 推到 20h，在 start=0 时对 verdict/completion 完全无影响。shipped `base-01-timed` 的 timeline 只有 `status="failed"`（scenario.toml:25,38,51），没有任何 restoration change，`assetDelayS` 在该场景是纯 no-op。C2a:153/158 只测“failure 窗口不变/可比较”，没有测延迟修复改变结论。修法：按每步 start 求 asset status（真正时间语义），或明确 `assetDelayS` 只影响 `start_s` 前状态并给出跨 start 的验收用例；按前一状态定义 restoration；无 restoration 时拒绝/告警。

4. **B4（BLOCKER）结果/compare 复用 C1 类型不成立。** C2a:101 的 outcome 是 `{timed, caveat}`，C1 `OutcomeKind` 只有 `structure/plan/caveat`（whatif lib:262–266）；host `whatif_result_get/list` 与 `whatif_compare` 都按 C1 `VariantResult` 反序列化（host whatif.rs:162–183,185–200），而 C1 类型要求 `actions/rootCauses/externalDependencies`（whatif lib:295–312），C2a:97–104 没有这些字段，会直接反序列化失败；即使补齐，`timed` 也会被 serde 忽略（`VariantResult` 无 `deny_unknown_fields`），UI 显示空 outcome（domain.ts:401–405）。C2a:108 “compare 复用 C1 CompareResult”同样不成立：C1 `compare` 只接受 `&[VariantResult]`、按 `outcome.plan` 排序（whatif lib:551–590），timed 没有 plan，也没有 mode 过滤。修法：定义 Result 0.2 tagged union（structure/plan/timed）及 host/UI 按 schema 分派；定义 CompareResult 0.2（timed 排序键、同 mode 过滤、跨 mode delta=null）；持久化 `violation/detail`（见 O2）。

5. **B5（BLOCKER）digest 契约不闭合，无法唯一复现。** C2a:107 只列“绑定 snapshotSha/scenarioSha/variant/...”，没有给 domain；C1 result 固定用 `poie.variant/0.1`（whatif lib:540–549），C2a 新增 warnings/storage/assetDelay/startMode 后必须升 `poie.variant/0.2`，但 `DIGEST.md` 本次只加 `poie.timed-scenario/0.1`（DIGEST:24），没有 snapshot/result 0.2。C2a:76 把 kind/scenarioSha 纳入 snapshot digest，却仍用 C1 `poie.snapshot/0.1`（whatif lib:114–121）；`tickDigest`（C2a:102）未定义，DIGEST §1.1 要求 fingerprint 与 SHA-256 分名；caveat/detail/violation 是否入 digest 未定。C2a:67/84/102 用 `?` 表示可选，未重申 DIGEST §2.6 的“缺省写 null、不得省略”（C1:104 曾明确）。scenario canonical JSON（C2a:56）只列语义项，没有字段名/顺序/固定向量。修法：在 DIGEST/SCHEMAS 登记 `poie.snapshot/0.2`、`poie.variant/0.2`（或 `poie.whatif-result/0.2`），固定 payload/排除项/空值规则，定义 `tickFingerprint` 或 `tickSha256`，给 scenario/result 固定测试向量。

## 动工前（REQUIRED-BEFORE）

- **R1（REQUIRED-BEFORE）`TimedCase`/settings 不对齐，`evaluate` 与 `run` 可能分叉。** C2a:51 的 `TimedCase` 没有 `max_depth/max_actions/horizon_s`；当前 `run` 固定 `DEFAULT_MAX_DEPTH`（timed.rs:413），无 horizon/action cap，而 Snapshot 0.2 与 Result settings（C2a:67,100）包含这些值，resultSha 会绑定未生效设置；`nowS` 同样未用。T0/T1 用 `deadline="prediction"`（cases.toml:15,26），由私有 `prediction_deadline`（timed.rs:636–654）解析，`evaluate(&TimedCase{deadline_s:Option<f64>})` 无法表达。C2a:101 的 verdict 顺序也与 runner 实际优先级（capacity 459 → insufficient 498 → power 530 → too_late 546）不同。修法：把 depth/actions/horizon 加入 `TimedCase` 并真正传给 `Query`/做 horizon 检查，或从 timed snapshot/result 删除；明确 `run` 先解析 prediction 再调 evaluate；写明 verdict 优先级并测试。
- **R2（REQUIRED-BEFORE）`shift_restorations` 签名/边界/validity 交互未定。** C2a:34 返回 `Result<(), TimeError>`，同段又说“不修改原 timeline（返回新对象或 shifted_restorations）”，二者矛盾；`TimeError` 没有 delta>24h/horizon 越界变体（time.rs:220–226）。`horizon_s` 是截断、丢弃还是保留越界 change 未定义；delta=0、负值、`at_s+delta` 超 horizon、证据有效期（`evidence_validity_s`）在恢复推迟后使 status 变 `Unknown` 而非 `Failed` 的交互均未说明（time.rs:124–148）。修法：二选一签名 `shifted_restorations(...) -> Result<AssetTimeline, TimeError>`（推荐）或明确 mutate；新增 ShiftOutOfRange；定义 horizon/validity 规则并加单测。
- **R3（REQUIRED-BEFORE）DB v5、错误包络、UI/隔离细节。** C2a:125–134：`whatif_scenarios` 缺 `schema_version`/`created_by`（PLATFORM:20 要求持久化结构带 schemaVersion）；`whatif_snapshots` 新增 `kind/scenario_sha256` 的 static 回填未写；`whatif_results` “增加 mode/settings_json 已存在”自相矛盾（db.rs:130–134 已存在）；当前迁移是无条件 batch + `user_version=4`（db.rs:60–141），4→5 需 `has_column` 门控 ALTER、备份/回滚（PLATFORM:22），并更新 `db.rs:545/560` 的 v4 断言/表清单。C2a:119 的 `scenario_mismatch`/`power_limit_exceeded` 不在 PLATFORM:49–59；后者与 verdict `power_limited` 重名，建议映射 `invalid_parameter`/`model_mismatch`；缺 `scope_denied`/`unknown_result`；命令仍 `Result<T,String>` 与 PLATFORM:42–45 的 CommandError 未声明。UI：无 `whatif_scenario_list` 无法“场景选择”；新文案未提 i18n；C2a:157 的“创建/运行/compare”在浏览器无法执行，应像 C1:196/207 一样只让 Playwright 测 host-only 占位。
- **R4（REQUIRED-BEFORE）Snapshot 0.2 字段/kind/模式校验。** C2a:60–76 静默删掉 C1 的 `evidenceSha256`（C1:52；whatif lib:44），需明确保留或删除理由与迁移；`kind=static` 的 0.2 与 0.1 并存时何时用哪个、`timed` variant 能否跑 static snapshot、`structure/plan` 能否跑 timed snapshot、`startMode=beginning` 时 `startS` 是否必须为 0、`startS<0`/`nowS>horizonS` 校验均未写。C2a:119 的 `scenario_mismatch` 应覆盖 `scenario.modelSha == snapshot.modelSha == live model digest`；现文只校验 scenario digest，未校验模型相等。
- **R5（REQUIRED-BEFORE）SCHEMAS 把 digest domain 当 wire schema 登记且版本错位。** SCHEMAS:22 登记 `poie.timed-scenario/0.1` 为 C2a schema，但实际 wire 常量是 `poie.timed-scenario/0.2`（timed.rs:28），0.1 只是 DIGEST domain（DIGEST:24）；`check-schema-registry.py` 不扫 `poie-sim` 所以 CI 不报错，但注册表与代码已漂移。`poie.whatif-snapshot/0.2` 等 0.2 已登记，digest domain 0.2 未登记。修法：SCHEMAS 只登记 wire schema 0.2（或明确 0.1 是 canonical scenario JSON 的 schema），DIGEST 登记 0.2 domains，并让 checker 覆盖 `poie-sim`/`poie-whatif`。

## 可选（OPTIONAL）

- **O1（OPTIONAL）C2b 延后声明诚实。** C2a:24–27/75/160–164 明确无 promote/plan_to_proposed/Authorization/checkpoint，命令表 §6 与 DB §7 也没有；`startMode` 只有 beginning/at。仅 C2a:6 把 B1 审批列为上游但 C2a 不用，建议删掉以免误读。
- **O2（OPTIONAL）结果可解释性/文案。** C2a:101–104 只存 outcome/completion/warnings，丢掉 `TimedViolation{at_s,constraint,detail}`（timed.rs:187–206）；UI 的 power_limited/capacity_conflict“单独文案”没有时间/约束可显示。C2a:140 的 caveat 是英文，C1:88 是中文；若 caveat 入 outcome/digest，本地化会改 sha，建议固定常量或声明非规范字段。

## 测试/验收缺口（并入上述修法）

C2a:152–158 缺：scenario DTO 固定向量、warnings 顺序、v4→v5 迁移、timed/static 与 mode 交叉拒绝、`assetDelay` 影响结论的用例、restoration shift 跨 start/超过 horizon/evidence validity、verdict 优先级。`base-01-timed/cases.toml` 的预注册 verdict 只有 feasible/too_late/insufficient_resource/blocked/unknown，没有 power_limited/capacity_conflict；新增 fixture 还会动 `scenarios/base-01-timed/digests.toml` 重放，需写明。

## 已执行检查与边界

```text
python3 scripts/check-schema-registry.py   OK (31 schemas)
python3 scripts/check-docs.py              OK
git status --porcelain（写本文件前）       clean
```

边界：只读设计/源码校验；未运行 cargo/Tauri/Playwright，未验证实际 UI。C2b 诚实性通过。

VERDICT: CHANGES_REQUIRED
