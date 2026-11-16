# v1 Certificate Freeze

本目录冻结 S5-V Native Observatory / Certificate v2 迁移前的 v1 证据基线。

- manifest: `v1-frozen.json`
- 覆盖 36 个 canonical claim 证书目录与 5 个 canonical S2 证书目录
- 同时冻结 engine、survival、clock、closure、scheduler 与 Cargo.lock 的 SHA-256
- `verify_v1.py` 在 `check-all` 中验证这些字节未被修改

v2 尚未签发。Native Observatory 需要修改 engine API 时，必须：

1. 保持本目录所有 v1 字节不变
2. 在**新目录**签发 v2 证书
3. 记录 v1 → v2 successor 关系
4. v2 绑定完整运行时源码环境，而不仅是 engine.rs
5. v1 在冻结基线上独立验证

当前状态：

```text
v1 freeze: ESTABLISHED
Native Observatory: NOT ESTABLISHED
Certificate v2: NOT ESTABLISHED
```
