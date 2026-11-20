# S5-V N11 — Native External Event Evidence

基线：`3f9fff1b618ebade815f10a6e321e2cc3a169788`。

## 状态

```text
N11-0 PRECONDITION EVIDENCE HARDENING
  IN PROGRESS

N11-A EXTERNAL NATIVE EVENT JOURNAL
  ESTABLISHED (SCOPED)

N11-B EXTERNAL TRUE BEFORE/AFTER
  ESTABLISHED (SCOPED)

N11-C BIDIRECTIONAL NATIVE STATE-TRANSITION
  ESTABLISHED (SCOPED: 14 CASES; canonical before/after NOT_OBSERVED)

N11-D ADVERSARIAL NATIVE EVENT VALIDITY
  NOT ESTABLISHED

N11-E EVIDENCE AND CERTIFICATE INTEGRITY
  NOT ESTABLISHED
```

## 本轮完成

- N10 报告与 CLAIMS 对齐；N10-E 限定 scope 说明。
- N10 OR 策略分类去掉 case-specific 分支。
- N10 trajectory 明确 `external_event_order = NOT_OBSERVED_FROM_ENDPOINT_TRACE`。
- 新 N11 C99 FMU 在真实状态转换处记录原生事件 journal。
- journal 容量 32，overflow 显式；同刻事件独立 seq。
- 每个事件保存真实 before/after 七维状态。
- fmpy 与 C harness 读取同一 journal 并逐事件核对一致。
- 14 个案例：canonical 共同事件时间/顺序与 external native 事件对齐。
- N10-OR2 归类为 `VALID_POLICY_DEPENDENT_DIVERGENCE`，由选择/依赖规则解释，不依赖 case 名。

## 限制

```text
canonical before/after 未在 N11 证据中导出 => N11-C NOT ESTABLISHED
journal 未做 overflow 运行实例
N11 adversarial controls 尚未完成
N11 CI/clean check-all 尚未完成
```
