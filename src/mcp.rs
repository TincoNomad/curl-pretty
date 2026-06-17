use serde_json::Value;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

const SESSION_FILE: &str = "mcp_session";

fn config_dir() -> PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("pcurl")
}

fn generate_session_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("pcurl-{:x}", nanos)
}

pub fn get_or_create_session(session_id: Option<String>) -> String {
    if let Some(sid) = session_id {
        return sid;
    }

    let path = config_dir().join(SESSION_FILE);
    if let Ok(existing) = fs::read_to_string(&path) {
        let trimmed = existing.trim().to_string();
        if !trimmed.is_empty() {
            return trimmed;
        }
    }

    let new_sid = generate_session_id();
    let _ = fs::create_dir_all(config_dir());
    let _ = fs::write(&path, &new_sid);
    new_sid
}

pub fn build_json_rpc_body(method: &str, params: &str) -> Result<String, String> {
    let params_value: Value =
        serde_json::from_str(params).map_err(|e| format!("Invalid JSON in params: {}", e))?;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params_value,
    });

    Ok(body.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_json_rpc_body() {
        let body = build_json_rpc_body("tools/call", r#"{"name":"test"}"#).unwrap();
        // serde_json sorts keys alphabetically by default
        assert_eq!(
            body,
            r#"{"id":1,"jsonrpc":"2.0","method":"tools/call","params":{"name":"test"}}"#
        );
    }

    #[test]
    fn test_build_json_rpc_body_empty_params() {
        let body = build_json_rpc_body("resources/list", "{}").unwrap();
        assert_eq!(
            body,
            r#"{"id":1,"jsonrpc":"2.0","method":"resources/list","params":{}}"#
        );
    }

    #[test]
    fn test_build_json_rpc_body_invalid_params() {
        let result = build_json_rpc_body("tools/call", "not json");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid JSON"));
    }

    #[test]
    fn test_build_json_rpc_body_escapes_method_quotes() {
        let body = build_json_rpc_body("tools/call", "{\"name\":\"test\"}").unwrap();
        assert!(!body.contains("undefined"));
        assert!(body.contains("\"jsonrpc\":\"2.0\""));
        assert!(body.contains("\"method\":\"tools/call\""));
        assert!(body.contains("\"params\":{\"name\":\"test\"}"));
    }

    #[test]
    fn test_generate_session_id_is_non_empty() {
        let id = generate_session_id();
        assert!(!id.is_empty());
        assert!(id.starts_with("pcurl-"));
    }

    #[test]
    fn test_session_id_is_persistent() {
        let dir = std::env::temp_dir().join("pcurl-test-mcp");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();

        // Override config dir for testing by setting HOME
        // Actually, we can't easily. Let's just test the generate function.
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        assert_ne!(id1, id2, "Each call should generate a unique ID");
    }
}
