# BaSyx AAS 只读适配器兼容性预演报告（M9.2）

> **AI 预演 · 本机容器 · 非现场设备** —— 不计入 M9.2 退出条件。

- 预登记：[PREREG.md](PREREG.md)
- 镜像：`eclipsebasyx/aas-environment:2.0.0-milestone-15`
- 端点：`http://127.0.0.1:49536`；读取路径：`GET /submodels/dXJuOnBvaWV3ZWF2ZTpiYXNlLTAxOnB1bXAtcDE6b3BlcmF0aW9uYWwtZGF0YQ`
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

存在 FAIL：预演不通过；失败项如实保留。逐项判断是实现、模型还是预期错，并把理由写入 PREREG.md §7 变更记录；不得只改期望使结果变绿。

## 局限与注记

- 单容器、本机回环网络、无认证、镜像默认（InMemory）后端；不覆盖 TLS、权限、MongoDB 持久化后端、订阅/事件（MQTT）与规模。
- 数据为 AI 编写的合成值，不是现场设备读数；`ObservedAt` 由注入脚本写入，不是设备时钟。
- BaSyx 的 AAS v3 Property 不带服务端时间戳（C3b）：只有显式提供 `ObservedAt` 一类属性的 Submodel 才能声明 `EventTime`；否则适配器把观测降为 `uncertain` 并以接入时间计。
- C6 表明重启后数据丢失（InMemory）：HTTP 可达不等于数据可用；适配器把 404 视为错误而不是「无变化」，因此不会伪造状态。
- 断连与重连之间的间隔短于证据过期窗口，证据过期路径未在本次真实运行中被触发（由 `poie-runtime` 单元测试覆盖）。
- 仅测 BaSyx；Eclipse Ditto 与 ROS 2 未测。M9.2 退出条件仍需现场/测试实例与兼容性 ADR。
