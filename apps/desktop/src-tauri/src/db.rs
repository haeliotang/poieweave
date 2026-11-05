//! Small SQLite state layer owned by the host process.
//!
//! The webview never gets SQL. It can only call the narrow commands in
//! `lib.rs`, which go through this module and return JSON values.

use std::path::Path;

use poie_governance::ApprovalToken;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct OperatorRecord {
    pub display_name: String,
    pub salt_hex: String,
    pub verifier_hex: String,
    pub iterations: u32,
    pub created_at_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditRow {
    pub id: i64,
    pub at_ms: i64,
    pub actor: String,
    pub event: String,
    pub detail: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InferenceConfigRow {
    pub endpoint: String,
    pub model: String,
    pub has_key: bool,
    pub updated_at_ms: i64,
}

pub type TimedScenarioRow = (String, String, String, String, i64);

pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn open(path: &Path) -> Result<Db, String> {
        let connection = Connection::open(path).map_err(|error| format!("open {}: {error}", path.display()))?;
        let db = Db { connection };
        db.migrate()?;
        Ok(db)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Db, String> {
        let db = Db { connection: Connection::open_in_memory().map_err(|error| error.to_string())? };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), String> {
        self.connection
            .execute_batch(
                "PRAGMA journal_mode = WAL;
                 CREATE TABLE IF NOT EXISTS operators (
                     id INTEGER PRIMARY KEY CHECK (id = 1),
                     display_name TEXT NOT NULL,
                     salt_hex TEXT NOT NULL,
                     verifier_hex TEXT NOT NULL,
                     iterations INTEGER NOT NULL,
                     created_at_ms INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS audit (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     at_ms INTEGER NOT NULL,
                     actor TEXT NOT NULL,
                     event TEXT NOT NULL,
                     detail TEXT NOT NULL,
                     session_id TEXT,
                     turn_id INTEGER,
                     args_sha256 TEXT,
                     result_sha256 TEXT,
                     sap_record_id TEXT,
                     prev_hash TEXT,
                     hash TEXT,
                     legacy INTEGER NOT NULL DEFAULT 0
                 );
                 CREATE INDEX IF NOT EXISTS audit_by_time ON audit(at_ms DESC, id DESC);
                 CREATE TABLE IF NOT EXISTS inference_config (
                     id INTEGER PRIMARY KEY CHECK (id = 1),
                     endpoint TEXT NOT NULL,
                     model TEXT NOT NULL,
                     has_key INTEGER NOT NULL,
                     updated_at_ms INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS approval_tokens (
                     id TEXT PRIMARY KEY,
                     session_id TEXT NOT NULL,
                     run_id TEXT NOT NULL,
                     call_id TEXT NOT NULL,
                     tool TEXT NOT NULL,
                     tool_version TEXT NOT NULL,
                     subject_digest TEXT NOT NULL,
                     policy_version TEXT NOT NULL,
                     args_json_redacted TEXT NOT NULL,
                     impact_json TEXT NOT NULL,
                     citations_json TEXT NOT NULL,
                     state TEXT NOT NULL,
                     created_at_ms INTEGER NOT NULL,
                     decided_at_ms INTEGER,
                     started_at_ms INTEGER,
                     expires_at_ms INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS agent_runs (
                     run_id TEXT PRIMARY KEY,
                     schema_version TEXT NOT NULL,
                     session_id TEXT NOT NULL,
                     state_json TEXT NOT NULL,
                     created_at_ms INTEGER NOT NULL,
                     updated_at_ms INTEGER NOT NULL,
                     expires_at_ms INTEGER NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS approval_by_run ON approval_tokens(run_id, state);
                 CREATE TABLE IF NOT EXISTS whatif_snapshots (
                     snapshot_id TEXT PRIMARY KEY, schema_version TEXT NOT NULL, session_id TEXT NOT NULL,
                     snapshot_json TEXT NOT NULL, snapshot_sha256 TEXT NOT NULL, created_at INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS whatif_variants (
                     variant_id TEXT PRIMARY KEY, snapshot_id TEXT NOT NULL, schema_version TEXT NOT NULL,
                     variant_json TEXT NOT NULL, created_at INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS whatif_results (
                     result_id TEXT PRIMARY KEY, variant_id TEXT NOT NULL, snapshot_id TEXT NOT NULL,
                     schema_version TEXT NOT NULL, mode TEXT NOT NULL, settings_json TEXT NOT NULL,
                     result_json TEXT NOT NULL, result_sha256 TEXT NOT NULL, created_at INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS whatif_timed_scenarios (
                     scenario_sha256 TEXT PRIMARY KEY, scenario_path TEXT NOT NULL, schema_version TEXT NOT NULL,
                     model_sha256 TEXT NOT NULL, description TEXT NOT NULL, imported_at INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS whatif_timed_snapshots (
                     snapshot_id TEXT PRIMARY KEY, schema_version TEXT NOT NULL, snapshot_json TEXT NOT NULL,
                     snapshot_sha256 TEXT NOT NULL, created_at INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS whatif_timed_variants (
                     variant_id TEXT PRIMARY KEY, snapshot_id TEXT NOT NULL, schema_version TEXT NOT NULL,
                     variant_json TEXT NOT NULL, created_at INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS whatif_timed_results (
                     result_id TEXT PRIMARY KEY, variant_id TEXT NOT NULL, snapshot_id TEXT NOT NULL,
                     schema_version TEXT NOT NULL, mode TEXT NOT NULL, settings_json TEXT NOT NULL,
                     result_json TEXT NOT NULL, result_sha256 TEXT NOT NULL, created_at INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS delivery_manifests (
                     manifest_sha256 TEXT PRIMARY KEY, manifest_json TEXT NOT NULL, imported_at INTEGER NOT NULL
                 );
                 CREATE TABLE IF NOT EXISTS delivery_state (
                     id INTEGER PRIMARY KEY CHECK (id = 1), state_json TEXT NOT NULL, updated_at INTEGER NOT NULL
                 );
                 PRAGMA user_version = 5;",
            )
            .map_err(|error| format!("migrate: {error}"))?;
        // Existing databases created before B0 have an audit table without the
        // new nullable columns; add them without touching historical rows.
        for column in ["session_id", "turn_id", "args_sha256", "result_sha256", "sap_record_id", "prev_hash", "hash"] {
            if !self.has_column("audit", column)? {
                let sql = format!("ALTER TABLE audit ADD COLUMN {column} TEXT");
                self.connection.execute_batch(&sql).map_err(|error| format!("migrate audit {column}: {error}"))?;
            }
        }
        if !self.has_column("audit", "legacy")? {
            self.connection
                .execute_batch("ALTER TABLE audit ADD COLUMN legacy INTEGER NOT NULL DEFAULT 0")
                .map_err(|error| format!("migrate audit legacy: {error}"))?;
        }
        Ok(())
    }

    fn has_column(&self, table: &str, column: &str) -> Result<bool, String> {
        let mut statement =
            self.connection.prepare(&format!("PRAGMA table_info({table})")).map_err(|error| error.to_string())?;
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .map_err(|error| error.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;
        Ok(columns.iter().any(|name| name == column))
    }

    pub fn operator(&self) -> Result<Option<OperatorRecord>, String> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT display_name, salt_hex, verifier_hex, iterations, created_at_ms FROM operators WHERE id = 1",
            )
            .map_err(|error| error.to_string())?;
        let mut rows = statement.query([]).map_err(|error| error.to_string())?;
        let Some(row) = rows.next().map_err(|error| error.to_string())? else {
            return Ok(None);
        };
        Ok(Some(OperatorRecord {
            display_name: row.get(0).map_err(|error| error.to_string())?,
            salt_hex: row.get(1).map_err(|error| error.to_string())?,
            verifier_hex: row.get(2).map_err(|error| error.to_string())?,
            iterations: row.get(3).map_err(|error| error.to_string())?,
            created_at_ms: row.get(4).map_err(|error| error.to_string())?,
        }))
    }

    pub fn create_operator(&self, operator: &OperatorRecord) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO operators (id, display_name, salt_hex, verifier_hex, iterations, created_at_ms)
                 VALUES (1, ?1, ?2, ?3, ?4, ?5)",
                params![
                    operator.display_name,
                    operator.salt_hex,
                    operator.verifier_hex,
                    operator.iterations,
                    operator.created_at_ms
                ],
            )
            .map_err(|error| format!("create operator: {error}"))?;
        Ok(())
    }

    pub fn append_audit(&self, at_ms: i64, actor: &str, event: &str, detail: &str) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO audit (at_ms, actor, event, detail) VALUES (?1, ?2, ?3, ?4)",
                params![at_ms, actor, event, detail],
            )
            .map_err(|error| format!("append audit: {error}"))?;
        Ok(())
    }

    pub fn audit(&self, limit: usize) -> Result<Vec<AuditRow>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT id, at_ms, actor, event, detail FROM audit ORDER BY id DESC LIMIT ?1")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([limit as i64], |row| {
                Ok(AuditRow {
                    id: row.get(0)?,
                    at_ms: row.get(1)?,
                    actor: row.get(2)?,
                    event: row.get(3)?,
                    detail: row.get(4)?,
                })
            })
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    #[allow(clippy::too_many_arguments)]
    #[allow(dead_code)]
    pub fn save_agent_run(
        &self,
        run_id: &str,
        schema: &str,
        session_id: &str,
        state_json: &str,
        created_at_ms: i64,
        updated_at_ms: i64,
        expires_at_ms: i64,
    ) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO agent_runs (run_id, schema_version, session_id, state_json, created_at_ms, updated_at_ms, expires_at_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(run_id) DO UPDATE SET state_json=?4, updated_at_ms=?6, expires_at_ms=?7",
                params![run_id, schema, session_id, state_json, created_at_ms, updated_at_ms, expires_at_ms],
            )
            .map_err(|error| format!("save agent run: {error}"))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn agent_run(&self, run_id: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT state_json FROM agent_runs WHERE run_id=?1", [run_id], |row| row.get(0))
            .optional()
            .map_err(|error| format!("read agent run: {error}"))
    }

    #[allow(dead_code)]
    pub fn save_timed_scenario(
        &self,
        scenario_sha256: &str,
        scenario_path: &str,
        schema_version: &str,
        model_sha256: &str,
        description: &str,
        imported_at: i64,
    ) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT OR IGNORE INTO whatif_timed_scenarios (scenario_sha256,scenario_path,schema_version,model_sha256,description,imported_at) VALUES (?1,?2,?3,?4,?5,?6)",
                params![scenario_sha256, scenario_path, schema_version, model_sha256, description, imported_at],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn timed_scenario(&self, sha: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row(
                "SELECT scenario_path FROM whatif_timed_scenarios WHERE scenario_sha256=?1",
                [sha],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn timed_scenarios(&self, limit: u32) -> Result<Vec<TimedScenarioRow>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT scenario_sha256,scenario_path,model_sha256,description,imported_at FROM whatif_timed_scenarios ORDER BY imported_at DESC LIMIT ?1")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map([limit], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)))
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn save_timed_snapshot(&self, id: &str, schema_version: &str, json: &str, sha: &str, created_at: i64) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO whatif_timed_snapshots (snapshot_id,schema_version,snapshot_json,snapshot_sha256,created_at) VALUES (?1,?2,?3,?4,?5)",
                params![id, schema_version, json, sha, created_at],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn timed_snapshot(&self, id: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT snapshot_json FROM whatif_timed_snapshots WHERE snapshot_id=?1", [id], |row| row.get(0))
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn save_timed_variant(&self, id: &str, snapshot_id: &str, schema_version: &str, json: &str, created_at: i64) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO whatif_timed_variants (variant_id,snapshot_id,schema_version,variant_json,created_at) VALUES (?1,?2,?3,?4,?5)",
                params![id, snapshot_id, schema_version, json, created_at],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn whatif_timed_variants(&self, snapshot_id: &str) -> Result<Vec<String>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT variant_json FROM whatif_timed_variants WHERE snapshot_id=?1 ORDER BY created_at ASC")
            .map_err(|error| error.to_string())?;
        let rows = statement.query_map([snapshot_id], |row| row.get(0)).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn timed_variant(&self, id: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT variant_json FROM whatif_timed_variants WHERE variant_id=?1", [id], |row| row.get(0))
            .optional()
            .map_err(|error| error.to_string())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn save_timed_result(
        &self,
        id: &str,
        variant_id: &str,
        snapshot_id: &str,
        schema_version: &str,
        mode: &str,
        settings_json: &str,
        json: &str,
        sha: &str,
        created_at: i64,
    ) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO whatif_timed_results (result_id,variant_id,snapshot_id,schema_version,mode,settings_json,result_json,result_sha256,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![id, variant_id, snapshot_id, schema_version, mode, settings_json, json, sha, created_at],
            )
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn timed_result(&self, id: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT result_json FROM whatif_timed_results WHERE result_id=?1", [id], |row| row.get(0))
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn timed_results(&self, snapshot_id: &str, variant_id: Option<&str>, limit: u32) -> Result<Vec<String>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT result_json FROM whatif_timed_results WHERE snapshot_id=?1 AND (?2 IS NULL OR variant_id=?2) ORDER BY created_at DESC LIMIT ?3")
            .map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![snapshot_id, variant_id, limit], |row| row.get(0))
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }

    pub fn save_approval_token(&self, token: &ApprovalToken) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO approval_tokens (id, session_id, run_id, call_id, tool, tool_version, subject_digest, policy_version, args_json_redacted, impact_json, citations_json, state, created_at_ms, decided_at_ms, started_at_ms, expires_at_ms)
                 VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16)",
                params![token.id, token.session_id, token.run_id, token.call_id, token.tool, token.tool_version,
                    token.subject_digest, token.policy_version, token.args_json_redacted, token.impact_json,
                    token.citations_json, token.state, token.created_at_ms, token.decided_at_ms,
                    token.started_at_ms, token.expires_at_ms],
            )
            .map_err(|error| format!("save approval token: {error}"))?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn approval_token(&self, id: &str) -> Result<Option<ApprovalToken>, String> {
        self.connection.query_row(
            "SELECT id, session_id, run_id, call_id, tool, tool_version, subject_digest, policy_version, args_json_redacted, impact_json, citations_json, state, created_at_ms, decided_at_ms, started_at_ms, expires_at_ms FROM approval_tokens WHERE id=?1",
            [id],
            |row| Ok(ApprovalToken {
                id: row.get(0)?, session_id: row.get(1)?, run_id: row.get(2)?, call_id: row.get(3)?,
                tool: row.get(4)?, tool_version: row.get(5)?, subject_digest: row.get(6)?, policy_version: row.get(7)?,
                args_json_redacted: row.get(8)?, impact_json: row.get(9)?, citations_json: row.get(10)?, state: row.get(11)?,
                created_at_ms: row.get(12)?, decided_at_ms: row.get(13)?, started_at_ms: row.get(14)?, expires_at_ms: row.get(15)?,
            }),
        ).optional().map_err(|error| format!("read approval token: {error}"))
    }

    #[allow(dead_code)]
    pub fn transition_approval(&self, id: &str, from: &str, to: &str, at_ms: i64) -> Result<bool, String> {
        let changed = self
            .connection
            .execute(
                "UPDATE approval_tokens SET state=?3, decided_at_ms=?4 WHERE id=?1 AND state=?2 AND expires_at_ms>?4",
                params![id, from, to, at_ms],
            )
            .map_err(|error| format!("transition approval: {error}"))?;
        Ok(changed == 1)
    }

    #[allow(dead_code)]
    pub fn delivery_manifest(&self, sha: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT manifest_json FROM delivery_manifests WHERE manifest_sha256=?1", [sha], |row| row.get(0))
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn delivery_state(&self) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT state_json FROM delivery_state WHERE id=1", [], |row| row.get(0))
            .optional()
            .map_err(|error| error.to_string())
    }

    pub fn save_delivery_import(
        &self,
        manifest_json: &str,
        sha: &str,
        state_json: &str,
        at: i64,
    ) -> Result<(), String> {
        self.connection.execute_batch("BEGIN IMMEDIATE").map_err(|error| format!("delivery begin: {error}"))?;
        let result = (|| {
            self.connection
                .execute(
                    "INSERT OR IGNORE INTO delivery_manifests (manifest_sha256,manifest_json,imported_at) VALUES (?1,?2,?3)",
                    params![sha, manifest_json, at],
                )
                .map_err(|error| format!("delivery manifest: {error}"))?;
            self.connection
                .execute(
                    "INSERT INTO delivery_state (id,state_json,updated_at) VALUES (1,?1,?2)
                     ON CONFLICT(id) DO UPDATE SET state_json=?1, updated_at=?2",
                    params![state_json, at],
                )
                .map_err(|error| format!("delivery state: {error}"))?;
            Ok(())
        })();
        match result {
            Ok(()) => self.connection.execute_batch("COMMIT").map_err(|error| format!("delivery commit: {error}")),
            Err(error) => {
                let _ = self.connection.execute_batch("ROLLBACK");
                Err(error)
            }
        }
    }

    pub fn save_delivery_state(&self, state_json: &str, at: i64) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO delivery_state (id,state_json,updated_at) VALUES (1,?1,?2)
                 ON CONFLICT(id) DO UPDATE SET state_json=?1, updated_at=?2",
                params![state_json, at],
            )
            .map_err(|error| format!("delivery state: {error}"))?;
        Ok(())
    }

    pub fn save_whatif_snapshot(
        &self,
        id: &str,
        schema: &str,
        session_id: &str,
        json: &str,
        sha: &str,
        at: i64,
    ) -> Result<(), String> {
        self.connection.execute(
            "INSERT INTO whatif_snapshots (snapshot_id,schema_version,session_id,snapshot_json,snapshot_sha256,created_at) VALUES (?1,?2,?3,?4,?5,?6)",
            params![id, schema, session_id, json, sha, at],
        ).map_err(|error| format!("save whatif snapshot: {error}"))?;
        Ok(())
    }
    pub fn whatif_snapshot(&self, id: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT snapshot_json FROM whatif_snapshots WHERE snapshot_id=?1", [id], |row| row.get(0))
            .optional()
            .map_err(|error| error.to_string())
    }
    pub fn whatif_snapshots(&self, limit: u32) -> Result<Vec<String>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT snapshot_json FROM whatif_snapshots ORDER BY created_at DESC LIMIT ?1")
            .map_err(|error| error.to_string())?;
        let rows = statement.query_map([limit], |row| row.get::<_, String>(0)).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }
    pub fn save_whatif_variant(
        &self,
        id: &str,
        snapshot_id: &str,
        schema: &str,
        json: &str,
        at: i64,
    ) -> Result<(), String> {
        self.connection.execute(
            "INSERT INTO whatif_variants (variant_id,snapshot_id,schema_version,variant_json,created_at) VALUES (?1,?2,?3,?4,?5)",
            params![id, snapshot_id, schema, json, at],
        ).map_err(|error| format!("save whatif variant: {error}"))?;
        Ok(())
    }
    pub fn whatif_variant(&self, id: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT variant_json FROM whatif_variants WHERE variant_id=?1", [id], |row| row.get(0))
            .optional()
            .map_err(|error| error.to_string())
    }
    pub fn whatif_variants(&self, snapshot_id: &str) -> Result<Vec<String>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT variant_json FROM whatif_variants WHERE snapshot_id=?1 ORDER BY created_at ASC")
            .map_err(|error| error.to_string())?;
        let rows =
            statement.query_map([snapshot_id], |row| row.get::<_, String>(0)).map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }
    #[allow(clippy::too_many_arguments)]
    pub fn save_whatif_result(
        &self,
        result_id: &str,
        variant_id: &str,
        snapshot_id: &str,
        schema: &str,
        mode: &str,
        settings: &str,
        json: &str,
        sha: &str,
        at: i64,
    ) -> Result<(), String> {
        self.connection.execute(
            "INSERT INTO whatif_results (result_id,variant_id,snapshot_id,schema_version,mode,settings_json,result_json,result_sha256,created_at) VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9)",
            params![result_id, variant_id, snapshot_id, schema, mode, settings, json, sha, at],
        ).map_err(|error| format!("save whatif result: {error}"))?;
        Ok(())
    }
    pub fn whatif_result(&self, id: &str) -> Result<Option<String>, String> {
        self.connection
            .query_row("SELECT result_json FROM whatif_results WHERE result_id=?1", [id], |row| row.get(0))
            .optional()
            .map_err(|error| error.to_string())
    }
    pub fn whatif_results(
        &self,
        snapshot_id: &str,
        variant_id: Option<&str>,
        limit: u32,
    ) -> Result<Vec<String>, String> {
        let mut statement = self.connection.prepare(
            "SELECT result_json FROM whatif_results WHERE snapshot_id=?1 AND (?2 IS NULL OR variant_id=?2) ORDER BY created_at DESC LIMIT ?3",
        ).map_err(|error| error.to_string())?;
        let rows = statement
            .query_map(params![snapshot_id, variant_id, limit], |row| row.get::<_, String>(0))
            .map_err(|error| error.to_string())?;
        rows.collect::<Result<Vec<_>, _>>().map_err(|error| error.to_string())
    }
    pub fn inference_config(&self) -> Result<Option<InferenceConfigRow>, String> {
        let mut statement = self
            .connection
            .prepare("SELECT endpoint, model, has_key, updated_at_ms FROM inference_config WHERE id = 1")
            .map_err(|error| error.to_string())?;
        let mut rows = statement.query([]).map_err(|error| error.to_string())?;
        let Some(row) = rows.next().map_err(|error| error.to_string())? else {
            return Ok(None);
        };
        Ok(Some(InferenceConfigRow {
            endpoint: row.get(0).map_err(|error| error.to_string())?,
            model: row.get(1).map_err(|error| error.to_string())?,
            has_key: row.get::<_, i64>(2).map_err(|error| error.to_string())? != 0,
            updated_at_ms: row.get(3).map_err(|error| error.to_string())?,
        }))
    }

    pub fn save_inference_config(
        &self,
        endpoint: &str,
        model: &str,
        has_key: bool,
        updated_at_ms: i64,
    ) -> Result<(), String> {
        self.connection
            .execute(
                "INSERT INTO inference_config (id, endpoint, model, has_key, updated_at_ms)
                 VALUES (1, ?1, ?2, ?3, ?4)
                 ON CONFLICT(id) DO UPDATE SET
                     endpoint = excluded.endpoint,
                     model = excluded.model,
                     has_key = excluded.has_key,
                     updated_at_ms = excluded.updated_at_ms",
                params![endpoint, model, if has_key { 1 } else { 0 }, updated_at_ms],
            )
            .map_err(|error| format!("save inference config: {error}"))?;
        Ok(())
    }

    pub fn clear_inference_config(&self) -> Result<(), String> {
        self.connection
            .execute("DELETE FROM inference_config WHERE id = 1", [])
            .map_err(|error| format!("clear inference config: {error}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operator_and_inference_config_round_trip() {
        let db = Db::open_in_memory().unwrap();
        assert!(db.operator().unwrap().is_none());
        db.create_operator(&OperatorRecord {
            display_name: "operator".into(),
            salt_hex: "00".into(),
            verifier_hex: "ff".into(),
            iterations: 1,
            created_at_ms: 7,
        })
        .unwrap();
        assert_eq!(db.operator().unwrap().unwrap().display_name, "operator");
        db.save_inference_config("https://example.test/v1", "model", true, 9).unwrap();
        let config = db.inference_config().unwrap().unwrap();
        assert_eq!(config.endpoint, "https://example.test/v1");
        assert!(config.has_key);
        db.clear_inference_config().unwrap();
        assert!(db.inference_config().unwrap().is_none());
        db.append_audit(11, "operator", "auth.sign_in", "ok").unwrap();
        assert_eq!(db.audit(10).unwrap().len(), 1);
    }

    #[test]
    fn real_file_migration_reaches_v4_with_product_tables() {
        let path = std::env::temp_dir().join(format!("poieweave-acceptance-{}.sqlite3", std::process::id()));
        let _ = std::fs::remove_file(&path);
        {
            let db = Db::open(&path).unwrap();
            let version: i64 = db.connection.query_row("PRAGMA user_version", [], |row| row.get(0)).unwrap();
            assert_eq!(version, 5);
            let tables: Vec<String> = db
                .connection
                .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")
                .unwrap()
                .query_map([], |row| row.get(0))
                .unwrap()
                .collect::<Result<_, _>>()
                .unwrap();
            for table in [
                "agent_runs",
                "approval_tokens",
                "delivery_manifests",
                "delivery_state",
                "whatif_results",
                "whatif_snapshots",
                "whatif_variants",
                "whatif_timed_results",
                "whatif_timed_snapshots",
                "whatif_timed_variants",
                "whatif_timed_scenarios",
            ] {
                assert!(tables.iter().any(|name| name == table), "missing {table}");
            }
            db.save_agent_run("run-1", "poie.agent-run/0.1", "session-1", "{}", 1, 1, 10).unwrap();
            assert_eq!(db.agent_run("run-1").unwrap().as_deref(), Some("{}"));
        }
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn approval_token_lifecycle_is_atomic() {
        use poie_governance::ApprovalToken;
        let db = Db::open_in_memory().unwrap();
        let token = ApprovalToken {
            id: "tok-1".into(),
            session_id: "s".into(),
            run_id: "r".into(),
            call_id: "c".into(),
            tool: "action.draft".into(),
            tool_version: "1.0.0".into(),
            subject_digest: "d".into(),
            policy_version: "poie-agent/0.1".into(),
            args_json_redacted: "{}".into(),
            impact_json: "{}".into(),
            citations_json: "[]".into(),
            state: "pending".into(),
            created_at_ms: 1,
            decided_at_ms: None,
            started_at_ms: None,
            expires_at_ms: 1_000,
        };
        db.save_approval_token(&token).unwrap();
        assert_eq!(db.approval_token("tok-1").unwrap().unwrap().state, "pending");
        assert!(db.transition_approval("tok-1", "pending", "approved", 2).unwrap());
        assert!(!db.transition_approval("tok-1", "pending", "approved", 3).unwrap());
        assert!(db.transition_approval("tok-1", "approved", "executing", 4).unwrap());
        assert!(db.transition_approval("tok-1", "executing", "consumed", 5).unwrap());
        assert_eq!(db.approval_token("tok-1").unwrap().unwrap().state, "consumed");
    }
}
