//! Canonical JSON and SHA-256 digests for Poieweave product-layer artifacts.
//!
//! The formal PO/SAP types keep their human-readable fingerprints; this crate
//! produces byte-stable SHA-256 commitments over a canonical JSON payload.

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DigestError(pub String);

impl std::fmt::Display for DigestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for DigestError {}

pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, DigestError> {
    let value = serde_json::to_value(value).map_err(|error| DigestError(error.to_string()))?;
    let mut out = String::new();
    write_value(&value, &mut out)?;
    Ok(out)
}

fn write_value(value: &Value, out: &mut String) -> Result<(), DigestError> {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(value) => out.push_str(if *value { "true" } else { "false" }),
        Value::Number(number) => {
            let float = number.as_f64().ok_or_else(|| DigestError("number is not finite".into()))?;
            if !float.is_finite() {
                return Err(DigestError("number is not finite".into()));
            }
            if float == 0.0 {
                out.push('0');
            } else {
                out.push_str(&serde_json::to_string(number).map_err(|e| DigestError(e.to_string()))?);
            }
        }
        Value::String(text) => out.push_str(&serde_json::to_string(text).map_err(|e| DigestError(e.to_string()))?),
        Value::Array(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(item, out)?;
            }
            out.push(']');
        }
        Value::Object(object) => {
            out.push('{');
            write_object(object, out)?;
            out.push('}');
        }
    }
    Ok(())
}

fn write_object(object: &Map<String, Value>, out: &mut String) -> Result<(), DigestError> {
    let mut keys: Vec<&String> = object.keys().collect();
    keys.sort();
    for (index, key) in keys.into_iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&serde_json::to_string(key).map_err(|e| DigestError(e.to_string()))?);
        out.push(':');
        write_value(&object[key], out)?;
    }
    Ok(())
}

pub fn sha256_of<T: Serialize>(domain: &str, value: &T) -> Result<String, DigestError> {
    let canonical = canonical_json(value)?;
    let mut hasher = Sha256::new();
    hasher.update(domain.as_bytes());
    hasher.update(b"\n");
    hasher.update(canonical.as_bytes());
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn hex_sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn key_order_does_not_change_the_digest() {
        let first = sha256_of("poie.test/0.1", &json!({"b": 2, "a": 1})).unwrap();
        let second = sha256_of("poie.test/0.1", &json!({"a": 1, "b": 2})).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn changed_values_change_the_digest() {
        let first = sha256_of("poie.test/0.1", &json!({"a": 1})).unwrap();
        let second = sha256_of("poie.test/0.1", &json!({"a": 2})).unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn negative_zero_is_normalized() {
        let canonical = canonical_json(&json!({"value": -0.0})).unwrap();
        assert_eq!(canonical, "{\"value\":0}");
    }

    #[test]
    fn domain_separation_changes_the_digest() {
        let first = sha256_of("poie.a/0.1", &json!({"a": 1})).unwrap();
        let second = sha256_of("poie.b/0.1", &json!({"a": 1})).unwrap();
        assert_ne!(first, second);
    }
}
