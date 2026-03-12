use ah::session::{Session, SessionKey};
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn session_key_parses_id() {
    let key = SessionKey::from_str("deadbeef").expect("expected valid session id");
    assert_eq!(key, SessionKey::Id("deadbeef".to_string()));
}

#[test]
fn session_new_sets_created_at() {
    let before = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let session = Session::new("deadbeef".to_string(), Vec::new(), "test".to_string());
    let after = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    assert!(session.created_at >= before);
    assert!(session.created_at <= after);
}
