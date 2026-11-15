// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! A separate process leaves a partial record; reopen must truncate it.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use poie_state::log::AppendLog;

fn temp_path() -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("poie-crash-tail-{}-{nanos}.jsonl", std::process::id()))
}

#[test]
fn another_process_crash_tail_is_truncated() {
    let path = temp_path();
    fs::write(&path, b"").unwrap();
    let status = Command::new(env!("CARGO_BIN_EXE_poie-partial-writer")).arg(&path).status().unwrap();
    assert!(status.success());
    let log = AppendLog::open(&path).unwrap();
    assert!(log.records().unwrap().is_empty());
    assert_eq!(fs::metadata(&path).unwrap().len(), 0);
    let _ = fs::remove_file(&path);
}
