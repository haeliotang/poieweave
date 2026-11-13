# 当前状态快照（2026-11-07 UTC）

本页是给发起方/外部评审者的一页式状态；测试计数随代码更新（由 `check-test-matrix.py` 校验），最近一次干净克隆验证提交见 §3。逐项证据与索引见 [stage-gates](stage-gates.zh-CN.md)、[test-matrix](test-matrix.zh-CN.md) 与 [external-actions](external-actions.zh-CN.md)。

## 0. 版本与发行状态（2026-11-07）

| 项 | 值 |
| --- | --- |
| 当前源码版本 | **0.4.20（未发布）** |
| 最新已验证发行版 | **v0.4.19**（`release-verification.json` `ok=true`，flat `SHA256SUMS`） |
| 上一已验证发行版 | v0.3.4 |
| README 旧标记 | v0.1.1 已过时，README 已更正为 v0.4.19 |
| 签名/公证 | **未签名/未公证**；不得表述为已签名 |
| 许可 | 分层：应用/运行时 AGPL-3.0-or-later；契约/测试数据 Apache-2.0；文档 CC-BY-4.0；CLA 1.0 |

> README 的版本徽章、已验证发行版和源码版本现在明确区分；发行 receipt 只证明资产/摘要/版本，不证明 authenticity。

## 1. 里程碑

| 阶段 | 里程碑 | 状态 |
| --- | --- | --- |
| Foundation | M0–M5 | **完成** |
| Autonomy | M6–M8 | **完成**（H2 为负结果；H2b 发现母线修复环，H2c 小样本方向性支持；均已如实报告） |
| Embodiment | M9–M10 | 库级/模拟完成；M9c BaSyx 非零影子 3/3、bench v2 B6/B7/B5R 3/3、Modbus 虚拟从站 5/5；**物理试验台、真实故障与安全演练待外部** |
| Expansion | M11 | 库级与 H4 模拟完成（2026-11-01 按 VEC 修订重跑：净增益为负；原判定不确定）；**物理实施待外部** |
| Federation | M12 | 库级/模拟 + 本机 TCP 完成；**真实多节点待外部** |
| Product A–D MVP | A0/A1、B0/B1、C1、D1 | **完成**（C2/D2 明确延后；真实外部/发布验收另列） |

阶段门槛：Foundation 与后续门槛仍被 M1.8/M1.9/M1.10 与 M3.3 外部事项阻塞，因此**不宣称通过阶段门槛**（证据见 [stage-gates](stage-gates.zh-CN.md)）。

## 2. 测试与证据

真实环境验收：macOS app 已构建/启动；验收时 app-data DB 为 `user_version=4`，当前迁移已到 v13；宿主文件/模型验收测试 16 通过；真实 DeepSeek 会话已完成一次（Completion），但未做模型能力评测；UI 自动化仍受 macOS 辅助功能权限阻塞。详见 [real-environment-acceptance](real-environment-acceptance.zh-CN.md)。

| 项 | 当前值 |
| --- | --- |
| Rust 列表测试 | **463**（库 crate 447 + 其他包 16） |
| Python 桥测试 | 9（CI 执行） |
| 随机性质测试 | 图求解 120、记录 120、投影 120、调度 120、仿真 80、VEC 60 轨迹 + 报告 20×16 变异、规划器 80、重规划/基线 80+20+60、HLC 80、联邦分区 120、执行门 120、运行时 80、扩张 40+40、桥 40 trials |
| 预登记场景用例 | Base-01 40、Base-02 8、时间版 20 |
| 生成型实验产物 | **81** 个：117 个 CI 路径可重放/引用，live 容器预演按规则豁免（见 §2.1/§2.3） |
| 检查器 | 链接、测试矩阵计数、实验重放覆盖（均 CI 执行） |

### 2.1 AI 预演（非正式实验，不计入里程碑）

由 AI 子代理/主线程为外部参与者准备的起点；**不等同于正式预登记实验或外部验收**。逐项限制见 [autonomous-scope-audit §6](autonomous-scope-audit.zh-CN.md)、[ai-preview-tasks](../progress/ai-preview-tasks.md)。

| 事项 | 预演结果 | 产物 |
| --- | --- | --- |
| M1.8 契约评审 | 3 个互不共享上下文的评审视角，36 条、去重 25 项，其中 13 项为高严重度 | [评审汇总](ai-preview/contract-review-summary.zh-CN.md) |
| M3.3 外部用例 | 子代理独立编写 10 例，预登记后运行 10/10 通过 | [ai-preview-cases](../../scenarios/base-01/ai-preview-cases.toml)、[运行输出](ai-preview/base01-ai-cases-run.txt) |
| M1.10 商标检索 | 普通法、包名、域名补充检索；USPTO 50154930 已核实为真实申请 | [补充检索](ai-preview/trademark-supplement.zh-CN.md) |
| M9.2 适配器（旧预演） | BaSyx AAS 本机容器只读适配器 9/9；影子计划动作数为 0，C4 为平凡满足 | [basyx-shadow](../../experiments/basyx-shadow/REPORT.md) |
| M12.6 多节点（旧预演） | 两个 Docker 容器 + 网络层分区，D0–D4 共 14/14；共享内核与时钟 | [federation-docker](../../experiments/federation-docker/REPORT.md) |

### 2.2 正式预登记实验（内部，尚未外部复核）

这些实验有先于运行的 PREREG、封存/留出集与公开原始输出，但**不是外部独立复现**。

| 实验 | 结果 | 产物 |
| --- | --- | --- |
| H2b | 母线 + 泵串行 4 h 在 4 h 截止下不可行；发现 Base-01 母线修复环；留出未运行 | [DIAGNOSIS](../../experiments/h2b-joint-planning/DIAGNOSIS.md) |
| H2c | 开发 5/16 vs 0/16；留出 3/8 vs 0/8（+37.5pp，按预登记支持；区间含 0，样本小） | [REPORT](../../experiments/h2c-joint-planning/REPORT.md) |
| S1–S4 Self-Sustaining Core | 能力闭合、生存边界、再生、审批回执与独立验证的模型级综合 | [综合文档](../research/self-sustaining-core.zh-CN.md) |

### 2.3 内部模拟台架/适配器验证（非物理验收）

| 项 | 结果 | 产物 |
| --- | --- | --- |
| M9c BaSyx 非零影子 | failed→1 个影子动作、stale→Unknown/expired=6、drift 恢复；3/3 PASS | [REPORT](../../experiments/basyx-shadow-m9c/REPORT.md) |
| M9 模拟台架 | B1–B5 5/5；B6/B7/B5R 3/3；Modbus 虚拟从站 5/5 | [bench-v2](../../experiments/m9-bench-v2/REPORT.md)、[modbus](../../experiments/m9-modbus/REPORT.md) |
| 桌面 M9 台架 | 命令 + UI 卡 + 真实 audit/gate 接入；UI smoke 8/8；**live 点击验收未完成** | [REPORT](../../apps/desktop/README.md) |

> **已裁定（2026-11-01）**：去重后 25 项中 **22 项接受、3 项推迟、0 拒绝**；按修复单元登记为 #105–#112。其中 #105（守恒，P4/P5）优先修复，合并后已**解除**研究纲领 §11 的守恒暂停；#106–#111 已合并（`blocked` 语义、SAP 执行安全、VEC 核算与统计、状态时间线），#112 的三项推迟。AI 预演不替代 M1.8 的真人评审。

## 3. 复现与验证运行

- 每次合并：CI（fmt/clippy/test/rustdoc/Python/链接/矩阵/重放覆盖/全部场景与实验/`cargo-deny`）全绿后合并，并确认 `main` CI run 成功。
- 2026-11-07 独立核对：`main` `b8f9a75` 的 [CI run 42363447638](https://github.com/haeliotang/poieweave/actions/runs/42363447638) 为 success；`55dbd77` 的 [CI run 42363128835](https://github.com/haeliotang/poieweave/actions/runs/42363128835)、`0f6d36c` 的 [CI run 42261109323](https://github.com/haeliotang/poieweave/actions/runs/42261109323) 与 [Desktop build run 42261109346](https://github.com/haeliotang/poieweave/actions/runs/42261109346) 也均为 success；`v0.4.19` 全部资产已本地下载并由 `scripts/verify_release.py` 复验 `ok=true`。
- 本地一键：`scripts/verify-all.sh`。
- 干净克隆：`scripts/check-fresh-clone.sh` —— 最近在 `d0d449c` 上通过（约 1 分钟）。
- GitHub runner：手动 dispatch scheduled workflow 的 [run 41246880552](https://github.com/haeliotang/poieweave/actions/runs/41246880552) 五个 job 全绿（verify-all、release tests、fresh-clone、advisories、licenses/bans/sources）。
- 定期：每周一 04:17 UTC scheduled workflow（`verify-all` + `cargo test --release` + `cargo-deny`；手动触发时额外 fresh-clone）。

## 3.1 现实有效性缺口（2026-11-07）

| 优先级 | 缺口 | 下一步 |
| --- | --- | --- |
| P0 | 缺少真实物理闭环 | 小型泵阀/PLC 台架，独立急停与安全保护；BOM 见 [M9 testbed](../design/m9-minimal-testbed.zh-CN.md) |
| P0 | 缺少独立契约审查 | 邀请工业自动化、过程安全、分布式系统研究者审阅三份契约 |
| P1 | 外部实验不足 | 获取 ≥10 个独立贡献的预登记场景 |
| P1 | 真实多节点未验证 | 两个独立节点测试分区、时钟偏差、重连与接管 |
| P1 | 过程模型未标定 | 用真实测量数据校准并记录误差与有效范围 |
| P2 | 治理/发布验收 | 双人审批 live UI、签名公证、正式合规复核 |

## 4. 外部阻塞项（需要发起方/外部参与）

| 优先级 | 事项 | 材料/下一步 |
| --- | --- | --- |
| 1 | **M1.8 三契约外部评审** | [自审问题包](contract-self-audit.zh-CN.md)、[contract-review 模板](../../.github/ISSUE_TEMPLATE/contract-review.yml)、[邀约文本](outreach-templates.zh-CN.md#a-m18-三份契约外部评审)；[AI 三方评审汇总](ai-preview/contract-review-summary.zh-CN.md) 可作对照起点 |
| 1 | **M1.9 ADR-0001 由 Inceptless 确认** | [ADR-0001](../adr/0001-open-source-boundary.zh-CN.md)、[邀约文本](outreach-templates.zh-CN.md#b-m19-adr-0001-开源边界确认inceptless) |
| 2 | **M3.3 外部贡献用例 ≥1/4** | [用例模板](case-contribution-template.zh-CN.md)；[AI 独立用例 10 例](../../scenarios/base-01/ai-preview-cases.toml) 仅供参考、不占配额 |
| 3 | **M1.10 正式商标检索** | [ADR-0002 §7](../adr/0002-rename-to-poieweave.zh-CN.md)；[AI 补充检索](ai-preview/trademark-supplement.zh-CN.md) |
| 4 | M9.0 合规复核；M9.2 真实适配器；M9.6/M10 试验台与安全演练；M11.6 物理实施；M12.6 真实多节点 | 见 [external-actions](external-actions.zh-CN.md) 与 [邀约文本](outreach-templates.zh-CN.md)；已有 [BaSyx 预演](../../experiments/basyx-shadow/REPORT.md) 与 [双容器预演](../../experiments/federation-docker/REPORT.md) 作为接入起点 |

完整审计见 [autonomous-scope-audit](autonomous-scope-audit.zh-CN.md)：所有不需要外部参与的里程碑工作已完成；剩余项全部需要外部参与。

## 5. 建议的下一步（按解锁价值）

0. AI 预演问题**已裁定并修复**（22 接受 / 3 推迟；#105–#111 已合并，守恒暂停已解除）；下一步是邀请真人评审验证修复、判断 AI 误报与遗漏；
1. 指定 M1.8 评审者（2–3 位）并发起审读；同日请发起方确认 ADR-0001（M1.9）；
2. 邀请独立贡献者提交 ≥10 个预登记用例（M3.3）；
3. 并行推进商标检索（M1.10）；
4. 按可获得的设备/网络安排 M9.2/M9.6/M10/M11.6/M12.6。

> 本快照不替代各证据文档；如与提交内容不一致，以提交与 CI 为准。更新方式：在每个里程碑合并或外部事项状态变化后刷新本页，并同步 DEVELOPMENT 修订记录。
