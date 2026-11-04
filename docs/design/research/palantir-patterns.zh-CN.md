# Palantir 产品模式研究与 Poieweave 取舍

> 研究备忘录：阶段决策以 Phase A–D 与 PLATFORM 冻结版为准；本文的外部事实不替代本地验证。

> 版本 0.1 / 2026-11-04 / 研究备忘，供 [Phase A–D](../README.zh-CN.md) 设计评审。
> 范围：Palantir 公开产品文档 + 本仓库现有契约/实现。未修改应用源码或 git 状态。
> 方法：2026-11-04 直接抓取下列官方页面；[1]–[26] 为可点击来源。第 1 节只陈述文档明示事实；第 2–6 节为笔者推断，不代表 Palantir 官方立场。
> 一句话：Palantir 值得学的是**把决策写成可治理的写路径**，而不是它的交付组织、数据接入广度或闭源商业模型。

## 1. 公开文档中的产品逻辑

> 以下均为厂商文档的自述，未经独立验证；“文档支持”不等于在 Poieweave 的离线、物理场景中可用。

### 1.1 Ontology：把“决策”而不只是“数据”建模

- Foundry Ontology 把数据集/模型映射为 **object type、property、link type、action type**；object 是实体或事件的实例，link 是两对象间关系，action 是对对象的受控修改；另有 function、interface、object view、role。[1][2]
- Ontology 被描述为组织的“operational layer / digital twin”，同时包含语义要素（对象、属性、链接）与动能要素（动作、函数、动态安全）。[1]
- 架构中心把它概括为 **data、logic、action、security 的四合一**：数据是名词，动作是动词，logic 可为规则/ML/LLM/多步编排，security 在运行时对人和 Agent 同时计算。[3]
- 设计意图是“决策中心”而非报表中心：捕获决策数据与 decision lineage，动作可先 staged as scenarios、受治理、再写回运营系统。[4]
- action type 是**一次事务**：定义参数、规则、submission criteria、权限、副作用（通知、webhook）；提交后写入 writeback dataset，并进入 action log。文档示例是“修改员工角色”，属业务对象编辑。[5]
- function 是服务端隔离执行的代码（TypeScript/Python），可读属性、遍历链接、做 Ontology edits，也可被 action 调用（function-backed action）。[6]
- interface 提供对象类型多态，使相同 shape 的对象可被一致地建模和交互。[2]

### 1.2 OSDK：把本体契约分发到应用

- OSDK 支持 npm/TypeScript、pip/Conda、Maven/Java 与 OpenAPI；类型和函数从本体的**相关子集**生成，token 只覆盖目标实体并叠加用户权限；可读可写。[7]
- 文档明确一条限制：行/列等控制是 **read-time enforcement only**，不延伸到 OSDK 响应载荷或应用下游；需要 marking/分类控制配合。[7]
- 支持 Ontology-as-code / SuperRepo，把对象、链接、动作、函数、前端放在同一仓库并本地再生成 SDK。[7]

### 1.3 AIP：Agent、自动化、评测、治理

- AIP 的 builder（AIP Logic、Chatbot Studio、Evals）建立在 Ontology 与开发工具链上；AIP 声称复用平台的安全、审计与资源管理框架。[9]
- Agent Stack 把任务建模为 **session**：强类型 context items、events、状态机与 state transitions；Agent Manager 可检查上下文、动作与 trace。[10][12]
- Agent Builder 的组成是 inputs、pre-computed variables、prompt、tools；工具可调函数、执行 action、查询 Ontology。文档特别提示：**变量里的 action 会在会话开始时自动执行**，即使变量从未被引用。[11]
- Automate 用“条件 + 效果”做业务自动化，效果包括提交 action、调 Logic/function、发通知；文档要求规划时写明目的、数据范围、执行身份、影响与恢复路径，并建议把有后果的变更 **staged for human review**。[13]
- AIP Logic 的 staged writes 可把编辑包成 action，再由 Automate 以 proposal 形式等人审；文档警告未审查的错误 proposal 可能大规模改坏运营数据。[14]
- AIP Evals 的结构是：evaluation suite、target function、test cases、evaluation function、metrics、experiments；可比较不同模型、不同版本与多次运行的方差。[15]
- 治理侧的可核实手段包括审计轨迹、函数语义化版本与发布管理、回滚、Ontology/AIP observability 的 execution trace；AIP 声称沿用与数据相同的安全措施。[9][16]

### 1.4 Apollo：交付是“计划 + 约束 + 订阅”，不是 push

- Apollo 是 Hub/Spoke 架构：Hub 的 Orchestration Engine 向 Spoke agent 下发 **Plan**，Plan 只有在全部 constraints 满足时才发送。[18][20]
- 环境没有单一“目标版本”：环境订阅 **Release Channel**，由约束求解出可安装的版本；默认通道为 DEV、RELEASE_CANDIDATE、RELEASE，按版本类型自动归入，也支持带健康要求的 promotion pipeline 与人工提升。[19][18]
- Plan-based 范式被明确拿来对比“control loop”：它要求变更以可见的 Plan 呈现，并在 Activity 中解释“为什么现在没有工作”。[20]
- 环境可以处于 **disconnected** 状态（例如车载环境只在停靠时连 Hub）；Apollo 支持 connected、disconnected、air-gapped 环境，并面向边缘做低带宽的版本差分更新、canary、SLO 驱动的 rollout 与 rollback。[17][21][26]
- 变更管理把审批绑定到组织身份（SAML 组）与合规要求，记录全部 change request；召回（recall）阻止坏版本继续扩散，并按 roll-off 策略替换。[17][23][22]
- 可观测性可与 Datadog/Prometheus 集成，监控器随软件一起打包。[17]

### 1.5 前向部署工程（FDE）的反馈模型

- Palantir 架构中心称平台通过 **Forward Deployed Engineering** 持续演进，并称之为“人类版反向传播”：工程师尽量贴近问题，与核心工程团队一起综合反馈、快速交付新功能。[24]
- S-1 的表述是：FDE 去过阿富汗基地和中西部工厂，**firsthand 观察用户挑战**；“Time in the field adds to the continuous improvement of our platforms.” [25]
- 【推断】FDE 的本质是一个高频、低损耗的**现场反馈通道**，不是一种头衔。开源项目无法复制其人力密度，但可以复制其产物：现场问题 → 可复现用例 → 回归证据。

## 2. 概念映射：Palantir → Poieweave

| Palantir 概念（文档事实） | Poieweave 现状（仓库事实） | 判断 |
| --- | --- | --- |
| object type / property | PO §3 的 Asset/Resource/Lot/Process/Service/Capability 与 `poie-ontology::model`；前端只有固定 `DomainOverview` 类型，无对象 API | **改造**：由模型生成只读 Catalog 与对象投影，不新建第二套本体 |
| link type | 能力图已有 `requires`/`requires_any`/`occupies`/`restores` 语义，但按查询计算，无稳定 link ID 与遍历 API | **改造**：稳定链接 ID + 有界遍历，端点类型校验 |
| action type（参数、submission criteria、副作用、权限） | SAP `Action` 已有 target/command/precondition/budget/reservations/reversibility/verification/authorization 与七项派发复核；但没有参数 schema、动作目录、表单 | **采纳**：Action Catalog 作为产品对象；策略编译到 SAP，不另写求值器 |
| function | Rust solver、planner、版本化 process bridge；无函数注册表，也不允许任意用户代码 | **改造**：只读/提案函数的注册表，不执行不可信代码 |
| security（marking、row/column、purpose） | 本地操作员认证（PBKDF2、8 小时会话）、审计、sandbox、动作 `policy_version`；无对象级策略引擎与 Agent scope | **改造**：先做工具/动作 scope 与审批；MVP 不复制 marking 体系 |
| OSDK | 桌面仅 Tauri IPC + 手写 TS 类型；无对外 SDK 或 HTTP 服务 | **采纳**：生成 TS SDK 与 catalog digest；MVP 仍只走 IPC |
| 读写闭环 / writeback dataset | 追加日志 + 投影 + 状态版本 + 重放 digest；写入带幂等键、租约与独立验证 | **采纳闭环，拒绝**把 writeback dataset 当事实源 |
| Agent tools | 推理只有单条 user message（≤4000 字符），无工具、无引用；仅审计元数据 | **采纳**：受控工具循环，而非自由文本助手 |
| Agent session / context items / trace | 只有审计表与推理请求审计；无 session、工具调用或上下文模型 | **改造**：持久化 run/tool-call/citation，不存储隐藏推理 |
| Automate（条件→效果） | planner + execution gate + 七项检查；无事件自动化引擎 | **改造**：只做 proposal staging，不做自动写 |
| AIP Evals | 预登记场景用例 + CI 重放 digest；无 LLM 评测框架 | **采纳**：离线 suite + scripted provider + 指标 |
| AI 治理（审计、版本、回滚、可观测） | 认证、审计、`policy_version`、接管/降级；无模型/提示版本注册与回滚 | **采纳子集**：审批、审计、版本绑定、回滚 |
| Ontology scenarios（what-if） | SAP §7 仿真隔离、`poie-sim`、Phase C 设计；无产品化工作区 | **采纳沙箱/比较/提升；改造** rebase 与 TTL |
| Release channels | CI 在 `v*` tag 构建三平台产物与 `SHA256SUMS`；无通道、提升、回滚、召回 | **采纳**：stable/canary/site + 证据门禁 |
| Plans / constraints | 规划与 SAP 检查已有；无“部署计划 + 约束解释”模型 | **改造**：delivery plan 与 why-no-update |
| disconnected / air-gapped | 本地优先运行时、联邦日志与冲突报告；无 bundle 导入/导出与更新机制 | **采纳**：离线 bundle、原子导入、回滚 |
| observability | 健康检查、重放 digest、诊断设计；无 fleet 遥测 | **改造**：opt-in、脱敏、默认关闭 |
| FDE | 开源协作 + 外部用例配额（M3.3）+ AI 预演；无现场工程师组织 | **拒绝组织模式；采纳**“现场问题→fixture” |

## 3. 四个阶段真正有用的部分

顺序仍应是 A→B→C→D：B 的工具 schema 依赖 A 的 Catalog，C 的 promote 依赖 B 的审批，D 的 bundle 依赖 A–C 的 digest。

### A. [Ontology 产品层](../phase-a-ontology-product.zh-CN.md)

真正有用的是把动作做成**有 schema 的产品对象**，让 UI、SDK、Agent 共用同一套参数、审批与证据要求；不是做可视化建模器。

| 可借鉴 | Poieweave 的落地 | 不照搬 |
| --- | --- | --- |
| action type 是带参数、submission criteria、权限的事务 | Action Catalog 由 PO 生成：参数单位/维度来自 PO，审批类别来自风险类，evidence_required 来自验证等级；submission criteria **编译为** SAP precondition + 七项检查 | 第二套策略求值器；通知/webhook 等副作用；对任意对象的无界写 |
| 对象/链接类型 + Object View | 对象详情、链接遍历、证据引用；只读 Object API；`process.*` 与 `sap.*` 动作目录 | 通用图数据库；UI 内自由建类型/属性；CRUD 写接口 |
| OSDK 从本体子集生成强类型 SDK | 生成 TS SDK + `catalog_digest`，CI 做漂移检查 | MVP 引入 HTTP 服务/远程多租户 |
| function 作为一等逻辑 | 注册只读查询、可行性、规划提案函数；输出带 digest | 让函数直接写 Ontology 或绕过 SAP |

验收：外部消费者只靠 Catalog 就能枚举对象/链接/动作；前端不再手写 Base-01 实体列表；动作预览无法 dispatch；Catalog/SDK 漂移使 CI 失败。

### B. [Agent 治理与评测](../phase-b-agent-governance.zh-CN.md)

真正有用的是把 Agent 的每次工具调用纳入与数据同级的**权限、引用、审批、审计**；不是让 Agent 更自由。

| 可借鉴 | Poieweave 的落地 | 不照搬 |
| --- | --- | --- |
| 工具继承对象/属性/链接的访问与 submission criteria | Tool Catalog：名称、JSON Schema、scope、审批类别、调用/字符预算；scope 由 Catalog 元数据派生 | `dispatch.*`、`shell.*`、`http.*`、`secrets.*`；从模型输出动态生成工具 |
| session + 强类型 context items + trace | 持久化 run、tool call、result、citation；审计可见“用了什么、改了什么、谁批准” | 存储隐藏推理/CoT；不可检查的长期记忆 |
| staged proposal / human review | 审批是持久状态：绑定 run、call、参数摘要、操作员会话、过期时间；恢复后继续 | 自动应用、一次批准复用、把“通知人”当审批 |
| Evals：suite、test case、evaluation function、experiment | 离线、确定性、scripted provider；覆盖工具序列、引用覆盖、越界拒绝、注入抵抗、预算 | CI 调真实模型；只靠 LLM-as-judge；用合成用例宣称现场安全 |

验收：所有越权工具尝试 100% 被拒并留痕；无引用的最终结论被拒（`citation_missing`）；审批可暂停/恢复且参数变更后失效；eval 在 CI 中确定性通过。

### C. [What-if / 场景工作区](../phase-c-what-if.zh-CN.md)

真正有用的是把“假设”变成**有 digest、可比较、可提升为 proposed action** 的隔离对象；不是做数据版本管理。

| 可借鉴 | Poieweave 的落地 | 不照搬 |
| --- | --- | --- |
| scenario 是动作沙箱，不是数据版本工具 | Snapshot + Variant + 独立 Model/ledger；固定 `now_s`；显式 SIMULATION 命名空间 | 10 分钟自动 rebase（会静默改变物理结果）；用 TTL 自动删除证据 |
| submission criteria 区分 scenario 与主本体 | 模拟策略与 live 策略分离；promote 本身是一个受审动作 | 放宽的模拟策略泄漏到 live；无 SAP 预览的 merge |
| 多场景比较、持久化场景元数据 | 比较 baseline/多 variant；结果带 digest 与 evidence refs；场景元数据可追溯 | 把 scenario 输出当现场事实；把 compare 做成纯 UI 排序 |
| 用场景做 Agent 评测 | 复用 variant 作为 Phase B 的评测 fixture | 让 what-if 直接产生可执行动作 |

验收：同输入同 digest；live 模型/证据变化后 promote 被拒（`stale_snapshot`）；variant 运行不改变 live digest；比较结果包含全部差异而非仅最优项。

### D. [站点交付](../phase-d-site-delivery.zh-CN.md)

真正有用的是**订阅通道 + 约束求解 + 可解释的不行动 + 召回/回滚**；不是做一套 Kubernetes 发布器。

| 可借鉴 | Poieweave 的落地 | 不照搬 |
| --- | --- | --- |
| 环境订阅 channel，约束解析出实际版本 | `stable/canary/site` 订阅；记录 channel、解析版本与原因，而不只记“目标版本” | 向环境 push 固定版本；只记目标版本 |
| promotion pipeline 带健康/标签要求 | 提升门禁绑定 eval、重放 digest、证据与合同版本 | 仅“CI 绿=可提升”；无证据的人工提升 |
| Plan + constraints + Activity | Delivery Plan；UI 必须解释“为什么没有更新/为什么被约束挡住” | 静默自动更新；只显示成功/失败 |
| recall / rollback | 召回坏 bundle，回滚上一 bundle，保留审计 | 原地不可逆覆盖；召回后不阻断继续扩散 |
| disconnected / air-gapped / edge | 离线 bundle 导入、原子发布、可选差分；默认关闭遥测 | 只支持 K8s；强制联网；默认开启 fleet 遥测 |

验收：bundle 的 model/contracts/evidence digest 不匹配即拒绝；回滚可用且留痕；诊断不含密钥/提示词/文件内容；完全离线可导入并启动。

## 4. 照搬 Palantir 的风险

1. **闭源/咨询模式不可复制。** Palantir 的交付模式高度依赖高密度 FDE 与专有平台；开源项目照搬会退化为“每个站点一个 fork、没有支持团队”。缓解：把可移植契约、fixture 和证据当作产品，而不是把关键能力放进托管服务。
2. **本体漂移压倒正式契约。** Palantir 的本体可在 UI 中快速演化；Poieweave 的 PO 有量纲、守恒、质量类与验证等级。若产品层成为可写事实源，物理不变量会先退化为“字段”，再退化为“演示”。缓解：Catalog 只生成、schema 有版本、未知即拒绝、语义变更走 ADR。
3. **决策支持 ≠ 物理自主。** 动作文档中的示例是业务对象编辑；这些公开页面没有给出物理命令的幂等键、租约、独立验证、接管与降级语义。[5] 照搬“洞察→决策”会得到一个**不能安全执行**的驾驶舱。缓解：SAP 是唯一执行语义，Catalog/Agent/What-if 只能 propose。
4. **数据集成偏好。** Palantir 的引力中心是数据源与集成；Poieweave 的引力中心是能力图与物理闭环。先做大量连接器会得到“接了很多系统但没有可验证能力”的平台，也会把本体降格为 schema mapping。缓解：模型优先、适配器后置、外部依赖显式。
5. **安全模型错位。** marking/row-column/purpose 是数据访问控制；物理安全需要命令授权、互锁、现场接管。OSDK 文档自己就说明 read-time enforcement 不控制下游副本。[7] 只复制 RBAC 会系统性高估安全性。缓解：数据访问与执行授权分离；每个动作单独授权；Agent 不继承环境权限。
6. **隐式自动化。** pre-computed variable 可在会话开始执行 action，Automate 可自动写；在物理系统里，“未展示的写”不可接受。[11][13] 缓解：禁止隐式写；所有写必须是以 proposed action 形式出现、可审、可回滚。
7. **Beta 语义与评测幻觉。** Ontology scenarios 与 Agent Builder 都标注 beta；离线 eval 不能证明现场安全。[8][12][15] 缓解：不把 beta 语义写进长期契约；评测分三层——离线 fixture、故障注入、现场证据，分别标注结论等级。

## 5. 产品原则与验收标准

| # | 原则 | 可测验收标准 |
| --- | --- | --- |
| P1 | 正式契约是唯一真源，Catalog 是带 digest 的投影 | 两次生成的 `catalog_digest` 相同；未知 schema/类型/版本返回稳定错误码；UI 不能新建类型；CI 重生成并 diff |
| P2 | 单一写路径：一切变更都是动作类型，SAP 是唯一执行语义 | 无 CRUD/写接口；每个写命令必须带 action id 与 policy version；工具白名单中 `dispatch.*` 数量为 0；UI 无法直接调用 executor |
| P3 | 无环境权限：每个工具/动作声明 scope，默认拒绝 | Tool Catalog 全量声明 scope；未知工具/越界参数被拒并留痕；审批绑定参数摘要与过期时间；复用批准失败 |
| P4 | 无证据不结论：事实性输出必须可追溯到本轮工具结果 | 无 citation 的结论被拒或标注 unverified；未知 citation id 触发 `citation_missing`；输出携带 evidence digest |
| P5 | 模拟不触达现场：SIMULATION 是显式命名空间 | variant 运行后 live digest 不变；promote 只产生 proposed actions；UI 持续显示 SIMULATION 标记 |
| P6 | 交付即“版本 + 证据”绑定 | bundle 导入在 model/contracts/evidence 任一不匹配时拒绝；回滚恢复到上一 bundle；未签名产物明确标注 |
| P7 | 断连优先，远程能力默认关闭 | 六个视图的核心流程在无网络时可用；远程推理默认关闭；遥测/诊断 opt-in 且脱敏 |
| P8 | 可解释的“不行动”与“被拒绝” | Planning/Execution 展示七项检查的逐项失败原因；Delivery 展示“为什么没有更新”；不得只显示笼统失败 |
| P9 | 现场反馈必须变成 fixture | 每个现场问题/缺陷链接到预登记用例或 eval fixture；CI 重放；没有重放证据不得宣称修复 |

**阶段门禁（建议写入各 Phase 的退出条件）：**

| 阶段 | 必须同时满足 |
| --- | --- |
| A | 外部消费者仅凭 Catalog/SDK 完成枚举与读取；动作预览不 dispatch；Catalog/SDK 漂移 CI 失败；六个视图不回归 |
| B | 越权工具 100% 被拒；引用校验生效；审批暂停/恢复可测；离线 eval 在 CI 确定性通过；不存在 dispatch 工具 |
| C | 同输入同 digest；stale snapshot 拒绝 promote；live 状态零写入；比较结果完整且可重放 |
| D | 版本/证据不匹配拒绝导入；回滚可测；离线导入可启动；诊断无密钥；通道与解析版本可追溯 |

## 6. 结论

- **采纳**：动作目录作为 schema、生成式 SDK、工具 scope 与引用、审批即状态、channels/plans/recall、现场问题→fixture。
- **改造**：Catalog 从正式模型生成；审批编译到 SAP；scenario 用显式 stale 检查替代自动 rebase；delivery 用证据门禁替代“CI 绿”。
- **拒绝**：闭源/咨询式交付、UI 自由建模、writeback 作为真源、隐式自动写、把数据分析闭环当成物理执行闭环。
- **最大风险不是“做得不像 Palantir”，而是“做得很像 Palantir 的演示，却丢掉了 Poieweave 的物理契约与执行证据”。**
- 衡量产品层的指标应是：一次决策从对象→动作→审批→证据的可追溯完成率，以及在断连、过期证据、越权尝试下仍保持该完成率。

## 来源（访问于 2026-11-04）

1. [Ontology overview](https://www.palantir.com/docs/foundry/ontology/overview/)
2. [Ontology core concepts](https://www.palantir.com/docs/foundry/ontology/core-concepts/)
3. [The Ontology system](https://www.palantir.com/docs/foundry/architecture-center/ontology-system/)
4. [Why create an Ontology?](https://www.palantir.com/docs/foundry/ontology/why-ontology/)
5. [Action types overview](https://www.palantir.com/docs/foundry/action-types/overview/)
6. [Functions overview](https://www.palantir.com/docs/foundry/functions/overview/)
7. [Ontology SDK (OSDK) overview](https://www.palantir.com/docs/foundry/ontology-sdk/overview/)
8. [Ontology scenarios (Beta)](https://www.palantir.com/docs/foundry/ontology/overview-ontology-scenario/)
9. [AIP overview](https://www.palantir.com/docs/foundry/aip/overview/)
10. [The Agent Stack](https://www.palantir.com/docs/foundry/aip/agent-stack/)
11. [Agent Builder core concepts](https://www.palantir.com/docs/foundry/agent-builder/core-concepts/)
12. [Agent Manager overview](https://www.palantir.com/docs/foundry/agent-manager/overview/)
13. [Automate overview](https://www.palantir.com/docs/foundry/automate/overview/)
14. [AIP Logic integration with Automate](https://www.palantir.com/docs/foundry/logic/aip-logic-integration-automate/)
15. [AIP Evals overview](https://www.palantir.com/docs/foundry/aip-evals/overview/)
16. [AI ethics and governance](https://www.palantir.com/docs/foundry/aip/ethics-governance/)
17. [Apollo introduction](https://www.palantir.com/docs/apollo/core/introduction/)
18. [Apollo overview](https://www.palantir.com/docs/apollo/core/overview/)
19. [Apollo release channels](https://www.palantir.com/docs/apollo/core/release-channels/)
20. [Apollo plans and constraints](https://www.palantir.com/docs/apollo/core/plans-and-constraints/)
21. [Apollo environments](https://www.palantir.com/docs/apollo/core/environments/)
22. [Apollo recalls](https://www.palantir.com/docs/apollo/recalling-releases/overview/)
23. [Apollo change management](https://www.palantir.com/docs/apollo/managing-changes/overview/)
24. [Architecture center overview (Forward Deployed Engineering)](https://www.palantir.com/docs/foundry/architecture-center/overview/)
25. [Palantir S-1/A, 2020 (SEC)](https://www.sec.gov/Archives/edgar/data/1321655/000119312520250103/d904406ds1a.htm)
26. [Apollo for the Edge (PDF, 2022)](https://www.palantir.com/assets/xrfr7uokpv1b/ECuz16kcqKOQ2P0jBBnVy/fd84e054ee75b9baf9151851d1c30bf9/ApolloForEdge.pdf)
