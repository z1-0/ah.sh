use crate::paths::get_session_dir;
use crate::provider::get_flake_contents;

use crate::session::types::{HISTORY_LIMIT, Session, SessionKey};
use anyhow::Result;
use std::cmp::Ordering;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

fn read_session_from_path(path: &Path) -> Option<Session> {
    let meta_path = path.join("metadata.json");
    if !meta_path.exists() {
        return None;
    }
    let content = fs::read_to_string(meta_path).ok()?;
    serde_json::from_str(&content).ok()
}

fn try_iter_sessions() -> Result<Vec<(Session, SystemTime)>> {
    let session_dir = get_session_dir()?;

    if !session_dir.exists() {
        return Ok(Vec::new());
    }

    let sessions: Vec<Result<(Session, SystemTime)>> = fs::read_dir(session_dir)?
        .flatten()
        .filter_map(|entry: std::fs::DirEntry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }

            let session = read_session_from_path(&path)?;
            let mtime = path
                .metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH);

            Some(Ok((session, mtime)))
        })
        .collect();

    sessions.into_iter().collect()
}

pub fn list_sessions() -> Result<Vec<Session>> {
    let mut sessions = try_iter_sessions()?;

    sessions.sort_by(|(a, a_mtime), (b, b_mtime)| match b_mtime.cmp(a_mtime) {
        Ordering::Equal => a.id.cmp(&b.id),
        other => other,
    });

    Ok(sessions.into_iter().map(|(session, _)| session).collect())
}

pub fn save_session(session: &Session) -> Result<()> {
    let session_dir = session.get_dir()?;
    if !session_dir.exists() {
        std::fs::create_dir_all(&session_dir)?;
    }

    let flake_contents = get_flake_contents(session.provider)(&session.languages)?;
    let flake_path = session_dir.join("flake.nix");
    std::fs::write(flake_path, flake_contents)?;

    let meta_path = session_dir.join("metadata.json");
    let content = serde_json::to_string_pretty(&session)?;
    std::fs::write(&meta_path, content)?;
    Ok(())
}

pub fn remove_session(session_id: &str) -> Result<bool> {
    let session_path = get_session_dir()?.join(session_id);
    if !session_path.exists() {
        return Ok(false);
    }
    fs::remove_dir_all(session_path)?;
    Ok(true)
}

pub fn clear_sessions() -> Result<usize> {
    let session_dir = get_session_dir()?;

    if !session_dir.exists() {
        return Ok(0);
    }

    let mut removed = 0usize;

    for entry in fs::read_dir(session_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            fs::remove_dir_all(path)?;
            removed += 1;
        }
    }

    Ok(removed)
}

pub fn update_history(session: &Session, cwd: &Path) -> Result<()> {
    let session_dir = session.get_dir()?;
    let history_path = session_dir.join("history.json");

    let mut history: Vec<String> = if history_path.exists() {
        let content = fs::read_to_string(&history_path)?;
        serde_json::from_str(&content).unwrap_or_default()
    } else {
        Vec::new()
    };

    let cwd_str = cwd.to_string_lossy().into_owned();
    history.retain(|entry| *entry != cwd_str);
    history.insert(0, cwd_str);
    history.truncate(HISTORY_LIMIT);

    let content = serde_json::to_string_pretty(&history)?;
    fs::write(&history_path, content)?;

    Ok(())
}

pub(crate) fn try_session_by_id(session_id: &str) -> Result<Option<Session>> {
    let session_path = get_session_dir()?.join(session_id);
    if !session_path.exists() {
        return Ok(None);
    }
    Ok(read_session_from_path(&session_path))
}

pub(crate) fn try_session_by_index(idx: usize) -> Result<Option<Session>> {
    let sessions = list_sessions()?;
    if idx > 0 && idx <= sessions.len() {
        Ok(Some(sessions[idx - 1].clone()))
    } else {
        Ok(None)
    }
}

pub(crate) fn try_session_by_key(key: &SessionKey) -> Result<Option<Session>> {
    match key {
        SessionKey::Id(id) => try_session_by_id(id),
        SessionKey::Index(idx) => try_session_by_index(*idx),
    }
}

pub fn find_session_by_key(key: &SessionKey) -> Result<Session> {
    try_session_by_key(key)?.ok_or_else(|| anyhow::anyhow!("session '{}' not found", key))
}

pub fn find_session_by_history() -> Result<Vec<Session>> {
    let cwd = crate::paths::get_cwd()?;
    let target_path = cwd.to_string_lossy().into_owned();

    let sessions = try_iter_sessions()?;

    let matching_sessions: Vec<_> = sessions
        .into_iter()
        .filter(|(session, _)| {
            let session_dir = match session.get_dir() {
                Ok(dir) => dir,
                _ => return false,
            };
            let history_path = session_dir.join("history.json");
            if !history_path.exists() {
                return false;
            }

            let content = match fs::read_to_string(&history_path) {
                Ok(c) => c,
                _ => return false,
            };

            let history: Vec<String> = match serde_json::from_str(&content) {
                Ok(h) => h,
                _ => return false,
            };

            history.iter().any(|entry| entry == &target_path)
        })
        .collect();

    let mut sorted: Vec<_> = matching_sessions;
    sorted.sort_by(|(_, a_mtime), (_, b_mtime)| b_mtime.cmp(a_mtime));

    Ok(sorted.into_iter().map(|(s, _)| s).collect())
}
