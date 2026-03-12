#[test]
fn session_module_exports_types_and_service() {
    let _ = ah::session::SessionKey::Index(1);
    let _ = ah::session::SessionService::list_sessions;
    let _ = ah::session::SessionRemoveResult {
        removed_ids: Vec::new(),
        missing_keys: Vec::new(),
    };
    let _ = ah::session::Session::new("id".to_string(), Vec::new(), "provider".to_string());
    let _ = ah::session::SessionError::NotFound("id".to_string());
    let _ = ah::session::SESSION_ID_LEN;
}
