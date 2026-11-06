# M9b BaSyx 非零影子动作 + 证据过期/漂移预演登记（PREREG）

- 实验 ID：`basyx-shadow-nonzero`
- 日期：2026-11-06
- 状态：**已登记，尚未运行**
- 性质：**AI 预演 · 本机单容器 · 非现场设备**；不计入 M9.2 退出条件，也不替代真实 PLC/电表闭环。
- 依据：M9.2 预演报告（影子动作 0，C4 平凡满足）；M9 方向

## 1. 问题

在真实运行的 BaSyx AAS Environment 上，只读适配器能否：

1. 把 `OperatingState=failed` 投影为 `Failed`，产生**非零** shadow plan，同时 `dispatched=false`；
2. 当 `ObservedAt` 早于 60 s 证据窗口时，把资产降为 `Unknown`，并记录过期证据；
3. 注入新的 `degraded` 观测后，恢复为 `Degraded`，shadow plan 回到零动作；
4. 全流程不派发任何动作。

## 2. 被测系统

同 M9.2：

- 镜像 `eclipsebasyx/aas-environment:2.0.0-milestone-15`
- 单容器、回环网络、无认证、InMemory 后端
- 只读 `ReadOnlyAdapter`，`MarkUnknown`，证据窗口 60 s
- 数据为合成值，`ObservedAt` 由脚本注入

## 3. 阶段与判据

| 编号 | 阶段 | 期望 |
| --- | --- | --- |
| N1 | 注入 `OperatingState=failed` + 新鲜 `ObservedAt` | 投影 `pump-p1=Failed`；shadow plan 动作数 **>0**；`dispatched=false` |
| N2 | 注入 `failed` + `ObservedAt=now-120s` | 投影 `pump-p1=Unknown`；`expired_evidence ≥1`；`healthy=false` |
| N3 | 注入 `degraded` + 新鲜 `ObservedAt` | 投影 `pump-p1=Degraded`；shadow plan 动作数 0；`dispatched=false` |
| N4 | 全流程 | 未派发任何动作；适配器只读 |

任一 FAIL → 预演失败，保留原始输出。

## 4. 方法

由 `run_nonzero.sh` 编排：

1. 启动固定镜像，等待 `/shells` 200；
2. 注入 failed 数据集；
3. runner `failed <ObservedAt>` → N1；
4. 注入 stale failed；runner `stale <ObservedAt>` → N2；
5. 注入 fresh degraded；runner `drifted <ObservedAt>` → N3；
6. `finish`，写 `REPORT.md` 与 `raw/`。

runner 复用 `experiments/basyx-shadow/runner`，仅增加阶段命令与 N1–N3 检查。

## 5. 局限

- 单容器、回环、无认证、InMemory；不是 M9.2 现场验收
- 合成数据；不是设备时钟
- 不包含 PLC/真实电表；硬件 BOM 另文规划
- 不证明 `poie-exec` 写适配器；只证明只读影子链路
