// SPDX-FileCopyrightText: 2026 Inceptless <support@inceptless.com>
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Local audit anchor: HMAC the current audit chain head with a random key kept
//! in the platform secret store. This is device-local anchoring, not a
//! third-party timestamp; it detects modification of the chain after anchoring.

use sha2::{Digest, Sha256};

use crate::db::Db;
use crate::secrets::SecretStore;

pub const ANCHOR_SERVICE: &str = "com.poieweave.audit-anchor";
pub const ANCHOR_ACCOUNT: &str = "hmac-key";

fn hmac_sha256(key: &[u8], message: &[u8]) -> String {
    let mut key_block = [0u8; 64];
    if key.len() > 64 {
        let digest = Sha256::digest(key);
        key_block[..32].copy_from_slice(&digest);
    } else {
        key_block[..key.len()].copy_from_slice(key);
    }
    let mut inner = Vec::with_capacity(64 + message.len());
    let mut outer = Vec::with_capacity(64 + 32);
    for byte in key_block {
        inner.push(byte ^ 0x36);
        outer.push(byte ^ 0x5c);
    }
    inner.extend_from_slice(message);
    let inner_hash = Sha256::digest(&inner);
    outer.extend_from_slice(&inner_hash);
    format!("{:x}", Sha256::digest(&outer))
}

fn anchor_key(secrets: &dyn SecretStore) -> Result<Vec<u8>, String> {
    if let Some(stored) = secrets.get(ANCHOR_SERVICE, ANCHOR_ACCOUNT)? {
        return hex::decode(stored).map_err(|error| format!("anchor key decode: {error}"));
    }
    let mut bytes = [0u8; 32];
    getrandom::getrandom(&mut bytes).map_err(|error| format!("anchor key random: {error}"))?;
    secrets.set(ANCHOR_SERVICE, ANCHOR_ACCOUNT, &hex::encode(bytes))?;
    Ok(bytes.to_vec())
}

pub fn anchor_current(db: &Db, secrets: &dyn SecretStore) -> Result<(u64, String, String), String> {
    let (count, head) = db.audit_chain()?;
    let Some(head) = head else { return Err("audit_chain_empty".into()) };
    let key = anchor_key(secrets)?;
    let message = format!("{count}|{head}");
    let hmac = hmac_sha256(&key, message.as_bytes());
    db.save_audit_anchor(count, &head, &hmac, crate::auth::now_ms())?;
    Ok((count, head, hmac))
}

pub fn verify_current(db: &Db, secrets: &dyn SecretStore) -> Result<bool, String> {
    let Some((count, head, expected)) = db.latest_audit_anchor()? else { return Ok(false) };
    let key = anchor_key(secrets)?;
    let actual = hmac_sha256(&key, format!("{count}|{head}").as_bytes());
    if actual != expected {
        return Ok(false);
    }
    Ok(db.verify_audit_chain()?.is_none())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::secrets::SecretStore;
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    struct MemorySecrets(Mutex<BTreeMap<String, String>>);
    impl SecretStore for MemorySecrets {
        fn set(&self, service: &str, account: &str, value: &str) -> Result<(), String> {
            self.0.lock().unwrap().insert(format!("{service}:{account}"), value.into());
            Ok(())
        }
        fn get(&self, service: &str, account: &str) -> Result<Option<String>, String> {
            Ok(self.0.lock().unwrap().get(&format!("{service}:{account}")).cloned())
        }
        fn delete(&self, service: &str, account: &str) -> Result<(), String> {
            self.0.lock().unwrap().remove(&format!("{service}:{account}"));
            Ok(())
        }
    }

    #[test]
    fn hmac_known_vector() {
        assert_eq!(
            hmac_sha256(b"key", b"The quick brown fox jumps over the lazy dog"),
            "f7bc83f430538424b13298e6aa6fb143ef4d59a14946175997479dbc2d1a3cd8"
        );
    }

    #[test]
    fn anchor_roundtrip_and_tamper_detection() {
        let db = Db::open_in_memory().unwrap();
        db.append_audit(1, "Ada", "one", "first").unwrap();
        db.append_audit(2, "Ada", "two", "second").unwrap();
        let secrets = MemorySecrets(Mutex::new(BTreeMap::new()));
        let (count, head, _) = anchor_current(&db, &secrets).unwrap();
        assert!(count > 0 && head.len() == 64);
        assert!(verify_current(&db, &secrets).unwrap());
        db.connection.execute("UPDATE audit SET detail='tampered' WHERE id=1", []).unwrap();
        assert!(!verify_current(&db, &secrets).unwrap());
    }
}
