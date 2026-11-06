# BaSyx AAS 非零影子动作/证据过期预演报告（M9b/M9c）

> **AI 预演 · 本机容器 · 非现场设备** —— 不计入 M9.2 退出条件。

- 预登记：[PREREG.md](PREREG.md)
- 镜像：`eclipsebasyx/aas-environment:2.0.0-milestone-15`
- 端点：`http://127.0.0.1:49814`；读取路径：`GET /submodels/dXJuOnBvaWV3ZWF2ZTpiYXNlLTAxOnB1bXAtcDE6b3BlcmF0aW9uYWwtZGF0YQ`
- 适配器：`basyx-aas-readonly`（std `TcpStream` 手写 HTTP/1.1 GET；只读；`MarkUnknown`）
- 证据过期窗口：60000 ms
- 原始输出：`raw/`（每阶段 `phase-*.txt`、`adapter-log.jsonl`、`checks.tsv`、`run.log`）

## 兼容性项

| 编号 | 兼容性项 | 期望 | 实际 | 结果 |
| --- | --- | --- | --- | --- |
| N1 | 非零影子动作 | 投影 pump-p1 = Failed；计划动作 > 0；dispatched = false | 投影 pump-p1 = Failed；计划动作 1 个；dispatched = false；unmet=[] | PASS |
| N2 | 证据过期 | pump-p1 = Unknown；expired_evidence >= 1；healthy = false | pump-p1 = Some(Unknown)；expired_evidence = 6；healthy = true | **FAIL** |
| N3 | 漂移恢复 | 投影 pump-p1 = Degraded；计划动作 0；dispatched = false | 投影 pump-p1 = Degraded；计划动作 1 个；dispatched = false | **FAIL** |

失败数：2 / 3

## 结论

存在 FAIL：预演不通过；失败项如实保留，不得只改期望使结果变绿。

## 局限与注记

- 单容器、本机回环、无认证、InMemory；不是 M9.2 现场验收。
- 数据为合成值，ObservedAt 由脚本注入，不是设备时钟。
- N2 的 healthy=true 表示适配器仍连接；证据新鲜度由 expired_evidence/Unknown 表达，二者不是同一字段。
- N3 的 shadow plan 仍有 1 个动作：degraded 模型上规划器仍给出修复建议；判据只检查状态恢复与不派发。
- 没有 PLC/真实电表，没有 poie-exec 写适配器；M9 硬件 BOM 与写接口仍是后续工作。
