# Self-Sustaining Core — 复现指南

日期：2026-11-15
范围：模型级/软件级证据；不含物理验收

## 1. 环境

- Rust workspace（见根 `Cargo.toml`）
- Python 3.11+（`tomllib`、`hashlib`）
- `reuse`、`cargo`、`gh`（可选，用于发布流程）

## 2. 基础验证

```bash
cargo test --workspace --locked
cargo clippy --workspace --all-targets -- -D warnings
reuse lint
python3 scripts/check-test-matrix.py
python3 scripts/check-experiment-replays.py
python3 scripts/check-status-counts.py
python3 scripts/check-docs.py
```

预期输出包含：

- `491 tests`
- `ok status snapshot: 491 (475 + 16)`
- `OK: ... CI experiment paths ... generated artifacts`
- `ok status counts: ...`
- `OK: all relative Markdown links resolve`

## 3. 证书 bundle

```bash
cargo run --locked -q -p poie-viability --bin poie-certificate-bundle \
  > experiments/certificate-bundle/bundle.json
python3 experiments/certificate-bundle/verify.py experiments/certificate-bundle/bundle.json
python3 experiments/certificate-bundle/render.py experiments/certificate-bundle/bundle.json \
  > experiments/certificate-bundle/SUMMARY.md
python3 experiments/certificate-bundle/verify_summary.py \
  experiments/certificate-bundle/bundle.json experiments/certificate-bundle/SUMMARY.md
python3 experiments/certificate-bundle-tamper/tamper.py
python3 experiments/certificate-clean-room/clean_room.py
```

预期：

```text
bundle=ok
summary=ok
tamper=detected
clean-room=ok
```

## 4. 单项研究

每个 `experiments/*/REPORT.md` 给出对应二进制、预期输出与限制。用以下命令重放：

```bash
cargo run --locked -q -p poie-viability --bin <binary>
diff experiments/<dir>/expected.txt <(cargo run --locked -q -p poie-viability --bin <binary>)
```

## 5. 未实现/未执行

- FMI 3.0 协同仿真：仅设计说明，未加载 FMU
- M9/M10 物理验收：未执行
- 持久签名密钥/第三方 attestation：未实现
- 多节点隔离 VM 故障测试：未执行

## 6. 结论边界

所有可复现结果均为模型级/软件级结论。它们不构成物理安全性、设备寿命或工业自主能力声明。
