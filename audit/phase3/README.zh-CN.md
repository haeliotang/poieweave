# S5-V Native Observatory Evidence

本目录记录 N1/N2 的原生观测证据。

- `native-trajectory.csv` 由 `poie-s5v-native-observatory` 直接调用
  `run_engine_observed` 生成
- 不使用零数量补货；观测在事件内核内完成
- 保留原始 200 h horizon
- 0/1/2/3/79 h 能量样本与 neutral-probe 证据一致
- 3 h 维修完成事件同时记录 Before/After：
  - Before: 38.5 kWh，维修仍运行
  - After: 38.0 kWh，repair 0.5 kWh
- 终止保持 `blocked:energy_depleted`，survived 284400 s
- `check_native.py` 在 `check-all` 中重放并逐字节比较

N2 非干扰测试覆盖：

- 提前终止
- 能源恰好在 horizon 耗尽
- 同刻多事件
- 维修中断
- 维修完成恰好等于 horizon
- 事件前/后观测

这些测试比较无观测与有观测运行的
termination、survived_s、final_inventory、完整 trace 和累计能量。
