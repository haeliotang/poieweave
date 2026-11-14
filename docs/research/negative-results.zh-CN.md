# Self-Sustaining Core — 负结果与已修正错误

日期：2026-11-15
范围：模型级/软件级研究记录

这份文档刻意收集**没有按预期成立**的结果、**被推翻的早期判断**，以及**看起来像结论但其实是建模假象**的情况。它比全部通过的模拟报告更能约束后续研究。

## 1. 已修正的实现错误

- **producer 索引陈旧**：直接 `model.processes.push` 后 `producers` 未更新；修复为统一的 `Model::add_process`。
- **容量未限制初始 SOC**：校验缺失导致初始电量可超过容量；已改为 fail-closed。
- **`process_yield` 未生效**：产出按名义量累加；修复为 `apply_process_outputs`，仅在 `yield_fraction < 1.0` 时累积。
- **S1 restorer 顺序敏感**：原实现返回首个可行 restorer，慢修复在前会得到更长计划；修复为保留最小 `total_s`。早期“顺序无关”的判断被推翻。
- **等时长 restorer 成本并列**：原实现选高消耗计划；加入 `relative_consumption` 后两种顺序都保留更多 grease。
- **S3 在途撤销缺口**：原实现只阻止未来提案；修复为检查 `revoke > now && revoke < now + plan.duration`。
- **approval 证书 denial dispatch**：`match` 仍只看 `revoke_at_s`；修复为 `(revoke_at_s, deny_at_s)` 并引入 denial 模拟。
- **`deny_at_s` 插入错误函数**：误入多服务循环；已移回单服务事件循环。

## 2. 没有成立的预期

| 预期 | 实际结果 | 解释 |
| --- | --- | --- |
| 修复时长推进会改变所有 S2 边界 | sparse 0/2.6 kW、energy 2.6/3.0/4.0 kW 完全不变 | 3 h 修复相对多天故障间隔可忽略 |
| 补货一定延长 horizon | 补货节奏 30 天对 10 天故障只从 30 天延到 40 天 | 补货频率不足，仍由消耗节奏决定 |
| 补货与时间语义会叠加 | 有补货时两种语义 horizon 相同 | 补货节奏成为 binding constraint，时间效应消失 |
| 长修复一定改变再生 horizon | 10 天故障下仍一致 | 5 天修复仍小于故障间隔 |
| 0 天 collapse 是系统固有最坏情况 | 完成时间语义下最坏为 5 天 | 0 天是“瞬时修复 + 密集故障”的建模假象 |

## 3. 方法学负结果

- **实验重放脚本只检查路径与输出存在**：`check-experiment-replays.py` 不比较数值；数值由各步骤的 `diff` 保证。
- **证书记录不等于重新计算**：证书 verifier 检查不变量与计数，不重跑完整模拟器；独立模型只覆盖窄范围。
- **临时签名不是信任**：`certificate-attestation` 只证明签名/篡改检测管线，不构成第三方 attestation。

## 4. 对研究的约束

- 任何 S2 结论都必须注明修复时间语义与 binding constraint。
- 不能把“瞬时修复”下的 0 天 collapse 直接当作系统设计缺陷。
- 补货、时间语义、冗余、能量的效应不能线性叠加，必须逐场景验证。
- 所有数值仅为模型内结果，不构成物理安全性、设备寿命或工业自主能力声明。
