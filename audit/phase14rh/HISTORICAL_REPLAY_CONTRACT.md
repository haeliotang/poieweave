# Historical Replay Contract

- historical commit: `161273d4599ff07fdd90f7a4be9e3bd6e221a94f`
- detached worktree: `/tmp/n14rh-v6`
- build: `cargo build --locked --offline --bins` from historical source
- every v6 claim runs its own historical runner binary
- comparison: raw stdout bytes equal frozen `expected.txt`; exit code 0
- record: runner hash, expected/actual hashes, exit code, stderr hash
