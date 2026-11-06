# M9b 诊断

M9b 首次运行（预登记判据）结果：

- N1 PASS：failed → shadow plan 1 个动作，dispatched=false。
- N2 FAIL：Unknown、expired_evidence=6，但 healthy=true。healthy 表示适配器连接状态，不是证据新鲜度；原期望错误。
- N3 FAIL：degraded 恢复，dispatched=false，但 shadow plan 仍有 1 个修复动作；原期望 0 动作错误。

M9b 报告：[REPORT.md](REPORT.md)。修正判据后的复验见 [M9c](../basyx-shadow-m9c/REPORT.md)。
