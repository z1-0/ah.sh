use ah::session::SessionKey;
use std::str::FromStr;

#[test]
fn session_key_parses_id() {
    let key = SessionKey::from_str("deadbeef").expect("expected valid session id");
    assert_eq!(key, SessionKey::Id("deadbeef".to_string()));
}
