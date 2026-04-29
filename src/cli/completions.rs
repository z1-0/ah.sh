use clap_complete::engine::{ArgValueCandidates, CompletionCandidate};

use crate::config;
use crate::provider::get_provider;
use crate::session;

pub fn language_candidates() -> Vec<CompletionCandidate> {
    let provider = config::get().provider;
    get_provider(provider)
        .get_supported_languages()
        .iter()
        .map(|l| CompletionCandidate::new(l.clone()))
        .collect()
}

pub fn session_id_candidates() -> Vec<CompletionCandidate> {
    let sessions = match session::list_sessions() {
        Ok(s) => s,
        Err(_) => return Vec::new(),
    };
    sessions
        .iter()
        .map(|s| CompletionCandidate::new(s.id.clone()))
        .collect()
}

pub fn make_language_completer() -> ArgValueCandidates {
    ArgValueCandidates::new(language_candidates)
}

pub fn make_session_key_completer() -> ArgValueCandidates {
    ArgValueCandidates::new(session_id_candidates)
}
