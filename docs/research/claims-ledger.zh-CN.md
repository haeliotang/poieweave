# 研究声明与证据对应表

日期：2026-11-15
范围：模型级/软件级证据；不含物理验收

每条声明都标明证据与边界。没有证据的声明不应出现在对外材料中。

| ID | 声明 | 证据 | 边界 |
| --- | --- | --- | --- |
| C1 | 无备件时车床维修构成自指循环 | `experiments/s1-tool-cycle`、S1 证书 `tool_cycle` | Base-08 修改场景 |
| C2 | 1 个 spare 可解锁 2 h 修复 | `experiments/s1-tool-cycle`、S1 证书 | 单资产 |
| C3 | 替代 restorer 可旁路循环（4 h） | `experiments/s1-alt-restorer`、S1 证书 | 合成替代过程 |
| C4 | 两条路径都存在时选更便宜的 2 h | `experiments/s1-alt-choice`、S1 证书 `path_choice` | 单资产 |
| C5 | Base-05 故障间隔 10/15/20/30 天 → 40/60/80/≥90 天 | `experiments/s2-failure-envelope` 等 | 模型内 |
| C6 | Base-06 独立覆盖 25/56、联合 24/56 | `experiments/s2-joint-coverage` | 双故障、约定假设 |
| C7 | 再生能力使 horizon 40→60 天 | `experiments/s2-regeneration` | 模型内 |
| C8 | 负载 3/4 kW 时能量成为 binding constraint | `experiments/s2-load-threshold` | 模型内 |
| C9 | 冗余泵 60→90 天 | `experiments/s2-redundancy-load` | 单冗余布局 |
| C10 | 补货间隔 10/20/30/60 天 → 60/50/40/30 天 | `experiments/s2-resupply-frequency` | 单一 consumable |
| C11 | 补货不能修复能力缺口（filter-1 仍 0 天） | `experiments/s2-adversarial-resupply` | 31 子集 |
| C12 | 时间语义改变 grease/insert 边界与 deadline | `experiments/s2-time-boundary`、`s2-time-insert`、`s2-time-horizon`、`s2-time-repair-sweep`、`certificate-s2-time` | 长修复/密集故障 |
| C13 | 时间语义在能量/sparse/标称再生下不变 | `experiments/s2-time-energy*`、`s2-regen-event` | 指定负载/修复时长 |
| C14 | 冗余效应大于时间语义效应 | `experiments/s2-redundancy-time`、S2 指标证书 | 单一布局 |
| C15 | 审批/租约/撤销/拒绝/并发 fail closed | `experiments/s3-recovery-loop`、`s3-approval-matrix`、`s3-lease-boundary`、`s3-revocation-inflight`、`s3-concurrency-deadline`、S3 证书 | 模型内执行 |
| C16 | 独立模型 8 项 26 案例一致 | `certificate-s4-independent`、`s4-*` | 窄模型 |
| C17 | bundle 可在 clean-room 中验证 | `certificate-clean-room` | 同一 verifier 代码 |
| C18 | 随机对抗中 0 天 collapse 是瞬时语义假象 | `experiments/s2-adversarial-random-time`、time 证书 | 200 子集、单一长修复 |
| C19 | 延迟故障只减少、不消除瞬时 0 天质量 | `experiments/s2-adversarial-random-window`、time 证书 | day0–10 窗口 |
| C20 | 显式替代资源组可按计划并发调度 | `experiments/s3-scheduler`、scheduler 测试 | 贪心、无抢占；`occupies` 为 AND |
| C21 | 生存循环应用被调度计划并按 makespan 推进 | `experiments/s5-scheduled-survival`、survival 测试 | 批次级库存耦合 |
| C22 | 最小 FMI 3.0 描述与容器通过检查 | `experiments/fmi-conformance` | 不执行 FMU |
| C23 | 另一进程的 crash tail 可在重开时截断 | `experiments/s5-multiprocess-crash` | 单写者、本地文件 |
| C24 | 签名路径可使用操作员提供的私钥 | `experiments/certificate-attestation`（`--key`） | 非信任锚 |
| C25 | 三个预登记场景中 advanced horizon 不低于 instant | `experiments/s5-semantics-parity` | 观察，不是普遍不变量 |
| C26 | `occupies` 的所有资源必须同时可用 | `experiments/s3-scheduler`、scheduler 测试 | 严格模式 |
| C27 | 调度与执行使用同一计划，且资源区间不重叠 | `experiments/s5-scheduled-survival`、scheduler 测试 | 批次级库存 |
| C28 | 替代策略不会删除未标记为可替代的必需资源 | `experiments/s3-scheduler`、scheduler 测试 | 显式 replaceable 集合 |
| C29 | 库存按规划快照结算，状态原子提交 | `experiments/s5-scheduled-survival`、survival 测试 | 批次级、无回滚日志 |
| C30 | 初始维修不覆盖已结算的能源/负载 | `experiments/s5-scheduled-survival`、survival 测试 | 模型级 |
| C31 | horizon 处未完成维修不被报告为完成 | `experiments/s5-scheduled-survival`、`SurvivalTermination` 测试 | 批次级 |
| C32 | horizon 时刻的阻塞不会被归类为完成 | `experiments/s5-scheduled-survival`、survival 测试 | 结构化判定 |
| C33 | PV 修复完成前不产生该 PV 电力 | `experiments/s5-scheduled-survival`、survival 测试 | 最小合成 PV |

## 不应出现的声明

- “真实设备可连续运行 X 天”
- “已通过工业安全验收”
- “已支持 FMI 3.0 协同仿真”
- “发布签名已被第三方信任”
- “多节点分区安全已验证”

这些均无当前证据，必须在对外材料中避免。
