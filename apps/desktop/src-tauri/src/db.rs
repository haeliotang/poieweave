//! Small SQLite state layer owned by the host process.
//!
//! The webview never gets SQL. It can only call the narrow commands in
//! `lib.rs`, which go through this module and return JSON values.

use std::path::Path;

use rusqlite::{params, Connection};
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
                 DROP TABLE IF EXISTS events;
                 DROP TABLE IF EXISTS notes;
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
                     detail TEXT NOT NULL
                 );
                 CREATE INDEX IF NOT EXISTS audit_by_time ON audit(at_ms DESC, id DESC);
                 CREATE TABLE IF NOT EXISTS inference_config (
                     id INTEGER PRIMARY KEY CHECK (id = 1),
                     endpoint TEXT NOT NULL,
                     model TEXT NOT NULL,
                     has_key INTEGER NOT NULL,
                     updated_at_ms INTEGER NOT NULL
                 );",
            )
            .map_err(|error| format!("migrate: {error}"))
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
}
