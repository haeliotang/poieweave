//! Local operator authentication.
//!
//! Poieweave has no remote account service yet. The desktop app therefore
//! authenticates a **local operator** with a passphrase verifier (PBKDF2-HMAC-
//! SHA256, 210k iterations, per-install salt). The session lives in the host
//! process only: it is not sent to the webview and it expires after eight hours.
//! This is a real device-local lock for remote inference and audit, not a claim
//! of multi-user or federated identity.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use pbkdf2::pbkdf2_hmac;
use serde::Serialize;
use sha2::Sha256;

use crate::db::{Db, OperatorRecord};

pub const PBKDF2_ITERATIONS: u32 = 210_000;
pub const SESSION_TTL_MS: i64 = 8 * 60 * 60 * 1000;

#[derive(Clone, Debug)]
pub struct Session {
    pub session_id: String,
    pub display_name: String,
    pub expires_at_ms: i64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthStatus {
    pub has_operator: bool,
    pub signed_in: bool,
    pub display_name: Option<String>,
    pub session_id: Option<String>,
    pub session_expires_at_ms: Option<i64>,
}

pub type SessionState = Mutex<Option<Session>>;

pub fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_millis() as i64).unwrap_or(0)
}

fn new_session_id() -> Result<String, String> {
    let mut bytes = [0u8; 16];
    getrandom::getrandom(&mut bytes).map_err(|error| format!("session id randomness: {error}"))?;
    Ok(hex::encode(bytes))
}

fn hash(passphrase: &str, salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut verifier = [0u8; 32];
    pbkdf2_hmac::<Sha256>(passphrase.as_bytes(), salt, iterations, &mut verifier);
    verifier
}

fn decode_hex(value: &str) -> Result<Vec<u8>, String> {
    hex::decode(value).map_err(|_| "stored credential is corrupt".to_string())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0u8;
    for (a, b) in left.iter().zip(right.iter()) {
        difference |= a ^ b;
    }
    difference == 0
}

pub fn status(db: &Db, session: &SessionState) -> Result<AuthStatus, String> {
    let has_operator = db.operator()?.is_some();
    let mut guard = session.lock().map_err(|_| "session lock poisoned".to_string())?;
    if let Some(current) = guard.as_ref() {
        if current.expires_at_ms <= now_ms() {
            *guard = None;
        }
    }
    let current = guard.clone();
    Ok(AuthStatus {
        has_operator,
        signed_in: current.is_some(),
        display_name: current.as_ref().map(|session| session.display_name.clone()),
        session_id: current.as_ref().map(|session| session.session_id.clone()),
        session_expires_at_ms: current.as_ref().map(|session| session.expires_at_ms),
    })
}

pub fn register(db: &Db, session: &SessionState, display_name: &str, passphrase: &str) -> Result<AuthStatus, String> {
    let display_name = display_name.trim();
    if display_name.len() < 2 || display_name.len() > 80 {
        return Err("display name must be 2–80 characters".into());
    }
    if passphrase.len() < 10 || passphrase.len() > 512 {
        return Err("passphrase must be at least 10 characters".into());
    }
    if db.operator()?.is_some() {
        return Err("a local operator already exists".into());
    }
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt).map_err(|error| format!("random salt: {error}"))?;
    let verifier = hash(passphrase, &salt, PBKDF2_ITERATIONS);
    let created_at_ms = now_ms();
    db.create_operator(&OperatorRecord {
        display_name: display_name.to_string(),
        salt_hex: hex::encode(salt),
        verifier_hex: hex::encode(verifier),
        iterations: PBKDF2_ITERATIONS,
        created_at_ms,
    })?;
    let session_value = Session {
        session_id: new_session_id()?,
        display_name: display_name.to_string(),
        expires_at_ms: created_at_ms + SESSION_TTL_MS,
    };
    *session.lock().map_err(|_| "session lock poisoned".to_string())? = Some(session_value);
    db.append_audit(created_at_ms, display_name, "auth.register", "local operator created")?;
    status(db, session)
}

pub fn add_operator(
    db: &Db,
    session: &SessionState,
    display_name: &str,
    passphrase: &str,
) -> Result<AuthStatus, String> {
    require(session)?;
    let display_name = display_name.trim();
    if display_name.len() < 2 || display_name.len() > 80 {
        return Err("display name must be 2–80 characters".into());
    }
    if passphrase.len() < 10 || passphrase.len() > 512 {
        return Err("passphrase must be at least 10 characters".into());
    }
    let mut salt = [0u8; 16];
    getrandom::getrandom(&mut salt).map_err(|error| format!("random salt: {error}"))?;
    let verifier = hash(passphrase, &salt, PBKDF2_ITERATIONS);
    let created_at_ms = now_ms();
    db.create_operator(&OperatorRecord {
        display_name: display_name.to_string(),
        salt_hex: hex::encode(salt),
        verifier_hex: hex::encode(verifier),
        iterations: PBKDF2_ITERATIONS,
        created_at_ms,
    })?;
    db.append_audit(created_at_ms, display_name, "auth.operator_added", "local operator added")?;
    status(db, session)
}

pub fn sign_in(
    db: &Db,
    session: &SessionState,
    passphrase: &str,
    display_name: Option<&str>,
) -> Result<AuthStatus, String> {
    let operators = db.operators()?;
    if operators.is_empty() {
        return Err("no local operator exists yet".into());
    }
    let wanted = display_name.map(str::trim).filter(|value| !value.is_empty());
    let mut matched: Option<&OperatorRecord> = None;
    for operator in &operators {
        if wanted.is_some_and(|name| name != operator.display_name) {
            continue;
        }
        let salt = decode_hex(&operator.salt_hex)?;
        let expected = decode_hex(&operator.verifier_hex)?;
        let candidate = hash(passphrase, &salt, operator.iterations);
        if constant_time_eq(&candidate, &expected) {
            if matched.is_some() && wanted.is_none() {
                return Err("multiple operators match this passphrase; select an operator".into());
            }
            matched = Some(operator);
        }
    }
    let Some(operator) = matched else {
        let actor = wanted.unwrap_or("unknown");
        db.append_audit(now_ms(), actor, "auth.sign_in_failed", "invalid passphrase")?;
        return Err("invalid passphrase".into());
    };
    let at = now_ms();
    *session.lock().map_err(|_| "session lock poisoned".to_string())? = Some(Session {
        session_id: new_session_id()?,
        display_name: operator.display_name.clone(),
        expires_at_ms: at + SESSION_TTL_MS,
    });
    db.append_audit(at, &operator.display_name, "auth.sign_in", "local operator session started")?;
    status(db, session)
}

pub fn sign_out(db: &Db, session: &SessionState) -> Result<AuthStatus, String> {
    let actor = {
        let mut guard = session.lock().map_err(|_| "session lock poisoned".to_string())?;
        let actor = guard.as_ref().map(|session| session.display_name.clone());
        *guard = None;
        actor
    };
    if let Some(actor) = actor {
        db.append_audit(now_ms(), &actor, "auth.sign_out", "local operator session ended")?;
    }
    status(db, session)
}

pub fn require(session: &SessionState) -> Result<Session, String> {
    let mut guard = session.lock().map_err(|_| "session lock poisoned".to_string())?;
    if let Some(current) = guard.as_ref() {
        if current.expires_at_ms <= now_ms() {
            *guard = None;
        }
    }
    guard.clone().ok_or_else(|| "authentication required".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn local_operator_register_sign_in_and_sign_out() {
        let db = Db::open_in_memory().unwrap();
        let session = SessionState::new(None);
        assert!(!status(&db, &session).unwrap().has_operator);

        let registered = register(&db, &session, "Ada", "correct horse battery staple").unwrap();
        assert!(registered.has_operator);
        assert!(registered.signed_in);
        assert_eq!(registered.display_name.as_deref(), Some("Ada"));

        let stored = db.operator().unwrap().unwrap();
        assert!(!stored.verifier_hex.contains("correct"));
        assert_ne!(stored.salt_hex, stored.verifier_hex);

        sign_out(&db, &session).unwrap();
        assert!(!status(&db, &session).unwrap().signed_in);
        assert!(sign_in(&db, &session, "wrong passphrase", None).is_err());
        assert!(sign_in(&db, &session, "correct horse battery staple", None).is_ok());
        assert_eq!(require(&session).unwrap().display_name, "Ada");
        assert!(add_operator(&db, &session, "Bob", "another passphrase").is_ok());
        assert_eq!(db.operators().unwrap().len(), 2);
        assert!(sign_in(&db, &session, "another passphrase", Some("Bob")).is_ok());
        assert_eq!(require(&session).unwrap().display_name, "Bob");

        let audit = db.audit(10).unwrap();
        assert!(audit.iter().any(|row| row.event == "auth.register"));
        assert!(audit.iter().any(|row| row.event == "auth.sign_in_failed"));
        assert!(audit.iter().any(|row| row.event == "auth.sign_in"));
    }
}
